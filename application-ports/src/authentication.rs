use core::future::Future;
use domain_shared::authentication::{AuthenticationLink, ClientCallbackToken, CsrfToken};
use domain_shared::discord::{InviteLink, UserId};
use thiserror::Error;

pub trait AuthenticationPort {
    fn create_authentication_link<'a>(
        &'a mut self,
        user_id: UserId,
    ) -> impl Future<Output = Result<AuthenticationLink, AuthenticationError>> + Send + 'a;
    fn confirm_authentication<'a>(
        &'a mut self,
        csrf_token: CsrfToken,
        client_callback_token: ClientCallbackToken,
    ) -> impl Future<Output = Result<(UserId, &'a InviteLink), AuthenticationError>> + Send + 'a;
}

#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("User authentication request was not found")]
    AuthenticationRequestNotFound,
    #[error("Service is temporarily unavailable")]
    TemporaryUnavailable,
    #[error("User authentication request was already confirmed")]
    AuthenticationRequestAlreadyConfirmed,
}
