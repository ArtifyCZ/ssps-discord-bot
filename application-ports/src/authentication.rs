use async_trait::async_trait;
use domain_shared::authentication::{AuthenticationLink, ClientCallbackToken, CsrfToken};
use domain_shared::discord::{InviteLink, UserId};
use thiserror::Error;
use tracing::error;

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

#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error(transparent)]
    Error(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("User authentication request was not found")]
    AuthenticationRequestNotFound,
    #[error("Service is temporarily unavailable")]
    TemporaryUnavailable,
}
