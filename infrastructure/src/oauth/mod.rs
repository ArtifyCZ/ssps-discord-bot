mod authentication_link;
mod csrf_token;

use crate::oauth::authentication_link::oauth_to_domain_authentication_link;
use crate::oauth::csrf_token::oauth_to_domain_csrf_token;
use async_trait::async_trait;
use chrono::Utc;
use domain::ports::oauth::{OAuthError, OAuthPort, OAuthToken, UserInfoDto};
use domain_shared::authentication::{
    AccessToken, AuthenticationLink, ClientCallbackToken, CsrfToken, RefreshToken, UserGroup,
};
use oauth2::basic::{
    BasicClient, BasicErrorResponse, BasicErrorResponseType, BasicRevocationErrorResponse,
    BasicTokenIntrospectionResponse, BasicTokenResponse,
};
use oauth2::url::Url;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, EndpointNotSet, EndpointSet, RedirectUrl,
    RequestTokenError, Scope, StandardRevocableToken, TokenResponse, TokenUrl,
};
use reqwest::Client as HttpClient;
use serde::Deserialize;
use std::time::Duration;
use tracing::{error, instrument, warn};

#[derive(Clone, Debug)]
pub struct OAuthAdapterConfig {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub tenant_id: TenantId,
    pub authentication_callback_url: Url,
}

pub struct OAuthAdapter {
    oauth_client: OAuthClient,
    http_client: HttpClient,
}

#[derive(Clone, Debug)]
pub struct TenantId(pub String);

pub type OAuthClient = oauth2::Client<
    BasicErrorResponse,
    BasicTokenResponse,
    BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    BasicRevocationErrorResponse,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

#[derive(Deserialize, Debug)]
struct UserInfoResponse {
    #[serde(rename = "displayName")]
    pub name: String,
    #[serde(rename = "mail")]
    pub email: String,
}

#[derive(Deserialize, Debug)]
struct UserGroupResponse {
    pub value: Vec<UserGroup>,
}

impl OAuthAdapter {
    #[instrument(level = "trace", skip_all)]
    pub fn new(config: OAuthAdapterConfig) -> Self {
        let oauth_client = create_oauth_client(config);
        let http_client = HttpClient::new();

        Self {
            oauth_client,
            http_client,
        }
    }
}

#[async_trait]
impl OAuthPort for OAuthAdapter {
    #[instrument(level = "debug", skip(self))]
    async fn create_authentication_link(&self) -> (AuthenticationLink, CsrfToken) {
        let (link, csrf_token) = self
            .oauth_client
            .authorize_url(oauth2::CsrfToken::new_random)
            .add_scope(Scope::new("User.read".to_string()))
            .add_scope(Scope::new("GroupMember.Read.All".to_string()))
            .add_scope(Scope::new("offline_access".to_string()))
            .url();

        (
            oauth_to_domain_authentication_link(link),
            oauth_to_domain_csrf_token(csrf_token),
        )
    }

    #[instrument(level = "debug", skip(self, client_callback_token))]
    async fn exchange_code_after_callback(
        &self,
        client_callback_token: ClientCallbackToken,
    ) -> domain::ports::oauth::Result<OAuthToken, OAuthError> {
        let token_result = self
            .oauth_client
            .exchange_code(AuthorizationCode::new(client_callback_token.0))
            .request_async(&self.http_client)
            .await
            .map_err(|err| match err {
                RequestTokenError::ServerResponse(_) => {
                    error!("OAuth exchange code after callback failed: {:?}", err,);
                    OAuthError::OAuthUnavailable
                }
                RequestTokenError::Request(err) => {
                    warn!("OAuth request failed with error: {:?}", err);
                    OAuthError::OAuthUnavailable
                }
                RequestTokenError::Parse(err, _) => {
                    warn!("OAuth request failed to parse response: {:?}", err);
                    OAuthError::OAuthUnavailable
                }
                RequestTokenError::Other(err) => {
                    warn!("Request failed with error: {:?}", err);
                    OAuthError::OAuthUnavailable
                }
            })?;

        let expires_at = Utc::now()
            + token_result
                .expires_in()
                .map(|d| d - Duration::from_secs(30))
                .unwrap_or(Duration::from_secs(300));

        let access_token = token_result.access_token().secret().clone();
        let refresh_token = token_result
            .refresh_token()
            .expect("Refresh token should be present")
            .secret()
            .clone();

        Ok(OAuthToken {
            access_token: AccessToken(access_token),
            expires_at,
            refresh_token: RefreshToken(refresh_token),
        })
    }

    #[instrument(level = "debug", skip(self, oauth_token))]
    async fn refresh_token(
        &self,
        oauth_token: &OAuthToken,
    ) -> domain::ports::oauth::Result<OAuthToken, OAuthError> {
        let refresh_token = oauth2::RefreshToken::new(oauth_token.refresh_token.0.clone());
        let token_result = self
            .oauth_client
            .exchange_refresh_token(&refresh_token)
            .request_async(&self.http_client)
            .await
            .map_err(|e| match e {
                RequestTokenError::ServerResponse(err) => {
                    match err.error() {
                        BasicErrorResponseType::InvalidGrant => {
                            // OAuth refresh token expired or revoked
                            OAuthError::TokenExpired
                        }
                        BasicErrorResponseType::InvalidClient
                        | BasicErrorResponseType::InvalidRequest
                        | BasicErrorResponseType::InvalidScope
                        | BasicErrorResponseType::UnauthorizedClient
                        | BasicErrorResponseType::UnsupportedGrantType
                        | BasicErrorResponseType::Extension(_) => {
                            warn!("OAuth request failed with error: {:?}", err);
                            OAuthError::OAuthUnavailable
                        }
                    }
                }
                RequestTokenError::Request(err) => {
                    warn!("OAuth request failed with error: {:?}", err);
                    OAuthError::OAuthUnavailable
                }
                RequestTokenError::Parse(err, _) => {
                    warn!("OAuth request failed to parse response: {:?}", err);
                    OAuthError::OAuthUnavailable
                }
                RequestTokenError::Other(err) => {
                    warn!("Request failed with error: {:?}", err);
                    OAuthError::OAuthUnavailable
                }
            })?;

        let access_token = AccessToken(token_result.access_token().secret().clone());
        let expires_at = Utc::now()
            + token_result
                .expires_in()
                .map(|d| d - Duration::from_secs(30))
                .unwrap_or(Duration::from_secs(300));
        let refresh_token = RefreshToken(
            token_result
                .refresh_token()
                .expect("New refresh token should be present")
                .secret()
                .clone(),
        );

        Ok(OAuthToken {
            access_token,
            expires_at,
            refresh_token,
        })
    }

    #[instrument(level = "debug", err, skip(self, access_token))]
    async fn get_user_info(
        &self,
        access_token: &AccessToken,
    ) -> domain::ports::oauth::Result<UserInfoDto, OAuthError> {
        let user_info = self
            .http_client
            .get("https://graph.microsoft.com/v1.0/me")
            .bearer_auth(access_token.0.clone())
            .send()
            .await
            .map_err(|err| {
                warn!("Failed to get user info: {:?}", err);
                OAuthError::OAuthUnavailable
            })?
            .text()
            .await
            .map_err(|err| {
                warn!("Failed to get user info: {:?}", err);
                OAuthError::OAuthUnavailable
            })?;
        let UserInfoResponse { name, email } = serde_json::from_str(&user_info).map_err(|err| {
            warn!("Failed to parse user info: {:?}", err);
            OAuthError::OAuthUnavailable
        })?;

        let user_groups = self
            .http_client
            .get("https://graph.microsoft.com/v1.0/me/memberOf")
            .bearer_auth(access_token.0.clone())
            .send()
            .await
            .map_err(|err| {
                warn!("Failed to get user groups: {:?}", err);
                OAuthError::OAuthUnavailable
            })?
            .text()
            .await
            .map_err(|err| {
                warn!("Failed to get user groups: {:?}", err);
                OAuthError::OAuthUnavailable
            })?;
        let groups = serde_json::from_str::<UserGroupResponse>(&user_groups)
            .map_err(|err| {
                warn!("Failed to parse user groups: {:?}", err);
                OAuthError::OAuthUnavailable
            })?
            .value;

        Ok(UserInfoDto {
            name,
            email,
            groups,
        })
    }
}

#[instrument(level = "trace", skip(config))]
fn create_oauth_client(config: OAuthAdapterConfig) -> OAuthClient {
    let auth_url = AuthUrl::new(format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
        config.tenant_id.0
    ))
    .unwrap();
    let token_url = TokenUrl::new(format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        config.tenant_id.0
    ))
    .unwrap();
    BasicClient::new(config.client_id)
        .set_client_secret(config.client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(RedirectUrl::from_url(config.authentication_callback_url))
}
