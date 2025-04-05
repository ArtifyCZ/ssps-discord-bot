use async_trait::async_trait;
use domain_shared::authentication::{AuthenticationLink, ClientCallbackToken, CsrfToken};
use domain_shared::discord::{InviteLink, UserId};

#[async_trait]
pub trait AuthenticationPort {
    async fn create_authentication_link(
        &self,
        user_id: UserId,
    ) -> Result<AuthenticationLink, AuthenticationError>;
    async fn confirm_authentication(
        &self,
        csrf_token: CsrfToken,
        client_callback_token: ClientCallbackToken,
    ) -> Result<InviteLink, AuthenticationError>;
}

#[derive(Debug)]
pub enum AuthenticationError {
    Error(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for AuthenticationError {
    fn from(e: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        AuthenticationError::Error(e)
    }
}
