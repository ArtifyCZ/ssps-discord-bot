mod authentication_link;
mod csrf_token;

use crate::oauth::authentication_link::oauth_to_domain_authentication_link;
use crate::oauth::csrf_token::oauth_to_domain_csrf_token;
use async_trait::async_trait;
use chrono::Utc;
use domain::ports::oauth::{OAuthPort, OAuthToken, UserInfoDto};
use domain_shared::authentication::{
    AccessToken, AuthenticationLink, ClientCallbackToken, CsrfToken, RefreshToken, UserGroup,
};
use oauth2::basic::{
    BasicClient, BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse,
    BasicTokenResponse,
};
use oauth2::url::Url;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, EndpointNotSet, EndpointSet, RedirectUrl,
    Scope, StandardRevocableToken, TokenResponse, TokenUrl,
};
use reqwest::Client as HttpClient;
use serde::Deserialize;
use std::time::Duration;
use tracing::instrument;

pub struct OAuthAdapter {
    oauth_client: OAuthClient,
    pub http_client: HttpClient,
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
    pub fn new(
        authentication_callback_url: Url,
        client_id: ClientId,
        client_secret: ClientSecret,
        tenant_id: TenantId,
    ) -> Self {
        let oauth_client = create_oauth_client(
            tenant_id.clone(),
            client_id.clone(),
            client_secret.clone(),
            authentication_callback_url.clone(),
        );

        let http_client = HttpClient::new();

        Self {
            oauth_client,
            http_client,
        }
    }
}

#[async_trait]
impl OAuthPort for OAuthAdapter {
    #[instrument(level = "debug", err, skip(self))]
    async fn create_authentication_link(
        &self,
    ) -> domain::ports::oauth::Result<(AuthenticationLink, CsrfToken)> {
        let (link, csrf_token) = self
            .oauth_client
            .authorize_url(oauth2::CsrfToken::new_random)
            .add_scope(Scope::new("User.read".to_string()))
            .add_scope(Scope::new("GroupMember.Read.All".to_string()))
            .add_scope(Scope::new("offline_access".to_string()))
            .url();

        Ok((
            oauth_to_domain_authentication_link(link),
            oauth_to_domain_csrf_token(csrf_token),
        ))
    }

    #[instrument(level = "debug", err, skip(self, client_callback_token))]
    async fn exchange_code_after_callback(
        &self,
        client_callback_token: ClientCallbackToken,
    ) -> domain::ports::oauth::Result<OAuthToken> {
        let token_result = self
            .oauth_client
            .exchange_code(AuthorizationCode::new(client_callback_token.0))
            .request_async(&self.http_client)
            .await?;
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

    #[instrument(level = "debug", err, skip(self, oauth_token))]
    async fn refresh_token(
        &self,
        oauth_token: &OAuthToken,
    ) -> domain::ports::oauth::Result<OAuthToken> {
        let refresh_token = oauth2::RefreshToken::new(oauth_token.refresh_token.0.clone());
        let token_result = self
            .oauth_client
            .exchange_refresh_token(&refresh_token)
            .request_async(&self.http_client)
            .await?;

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
    ) -> domain::ports::oauth::Result<UserInfoDto> {
        let user_info = self
            .http_client
            .get("https://graph.microsoft.com/v1.0/me")
            .bearer_auth(access_token.0.clone())
            .send()
            .await?
            .text()
            .await?;
        let UserInfoResponse { name, email } = serde_json::from_str(&user_info)?;

        Ok(UserInfoDto { name, email })
    }

    #[instrument(level = "debug", err, skip(self, access_token))]
    async fn get_user_groups(
        &self,
        access_token: &AccessToken,
    ) -> domain::ports::oauth::Result<Vec<UserGroup>> {
        let user_info = self
            .http_client
            .get("https://graph.microsoft.com/v1.0/me/memberOf")
            .bearer_auth(access_token.0.clone())
            .send()
            .await?
            .text()
            .await?;

        let user_groups: UserGroupResponse = serde_json::from_str(&user_info)?;

        Ok(user_groups.value)
    }
}

#[instrument(
    level = "trace",
    skip(tenant_id, client_id, client_secret, callback_url)
)]
fn create_oauth_client(
    tenant_id: TenantId,
    client_id: ClientId,
    client_secret: ClientSecret,
    callback_url: Url,
) -> OAuthClient {
    let auth_url = AuthUrl::new(format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
        tenant_id.0
    ))
    .unwrap();
    let token_url = TokenUrl::new(format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        tenant_id.0
    ))
    .unwrap();
    BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(RedirectUrl::from_url(callback_url))
}
