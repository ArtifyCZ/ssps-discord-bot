use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain_shared::authentication::{
    AccessToken, AuthenticationLink, ClientCallbackToken, CsrfToken, RefreshToken, UserGroup,
};
use std::fmt::Debug;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait OAuthPort {
    async fn create_authentication_link(&self) -> Result<(AuthenticationLink, CsrfToken)>;
    async fn exchange_code_after_callback(
        &self,
        client_callback_token: ClientCallbackToken,
    ) -> Result<OAuthToken>;
    async fn refresh_token(&self, oauth_token: &OAuthToken) -> Result<OAuthToken>;
    async fn get_user_info(&self, access_token: &AccessToken) -> Result<UserInfoDto>;
    async fn get_user_groups(&self, access_token: &AccessToken) -> Result<Vec<UserGroup>>;
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
}
