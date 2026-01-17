use chrono::{DateTime, Utc};
use domain_shared::authentication::{
    AccessToken, AuthenticationLink, ClientCallbackToken, CsrfToken, RefreshToken, UserGroup,
};
use std::fmt::Debug;
use std::future::Future;
use thiserror::Error;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait OAuthPort {
    fn create_authentication_link(
        &self,
    ) -> impl Future<Output = (AuthenticationLink, CsrfToken)> + Send;
    fn exchange_code_after_callback(
        &self,
        client_callback_token: ClientCallbackToken,
    ) -> impl Future<Output = Result<OAuthToken, OAuthError>> + Send;
    fn refresh_token(
        &self,
        oauth_token: &OAuthToken,
    ) -> impl Future<Output = Result<OAuthToken, OAuthError>> + Send;
    fn get_user_info(
        &self,
        access_token: &AccessToken,
    ) -> impl Future<Output = Result<UserInfoDto, OAuthError>> + Send;
}

#[derive(Clone, Debug, PartialEq)]
pub struct OAuthToken {
    pub access_token: AccessToken,
    pub expires_at: DateTime<Utc>,
    pub refresh_token: RefreshToken,
}

#[derive(Debug)]
pub struct UserInfoDto {
    pub name: String,
    pub email: String,
    pub groups: Vec<UserGroup>,
}

#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("OAuth is unavailable")]
    OAuthUnavailable,
    #[error("Refresh or access token has expired or was revoked")]
    TokenExpired,
}
