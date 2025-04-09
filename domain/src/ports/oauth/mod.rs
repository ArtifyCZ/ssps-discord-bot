use crate::authentication::authenticated_user::AuthenticatedUser;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain_shared::authentication::{
    AccessToken, AuthenticationLink, ClientCallbackToken, CsrfToken, RefreshToken, UserGroup,
};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, crate::ports::discord::Error>;

#[async_trait]
pub trait OAuthPort {
    async fn create_authentication_link(&self) -> Result<(AuthenticationLink, CsrfToken)>;
    async fn exchange_code_after_callback(
        &self,
        client_callback_token: ClientCallbackToken,
    ) -> Result<(AccessToken, DateTime<Utc>, RefreshToken)>;
    async fn get_user_info(&self, user: &mut AuthenticatedUser) -> Result<UserInfoDto>;
    async fn get_user_groups(&self, access_token: AccessToken) -> Result<Vec<UserGroup>>;
}

#[derive(Debug)]
pub struct UserInfoDto {
    pub name: String,
    pub email: String,
}
