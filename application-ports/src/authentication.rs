use async_trait::async_trait;
use domain_shared::authentication::{AuthenticationLink, ClientCallbackToken, CsrfToken};
use domain_shared::discord::{InviteLink, RoleId, UserId};
use thiserror::Error;
use tracing::error;

#[async_trait]
pub trait AuthenticationPort {
    async fn get_user_info(
        &self,
        user_id: UserId,
        force_refresh: bool,
    ) -> Result<Option<AuthenticatedUserInfoDto>, AuthenticationError>;
    async fn create_authentication_link(
        &self,
        user_id: UserId,
    ) -> Result<AuthenticationLink, AuthenticationError>;
    async fn confirm_authentication(
        &self,
        csrf_token: CsrfToken,
        client_callback_token: ClientCallbackToken,
    ) -> Result<InviteLink, AuthenticationError>;
    async fn get_main_student_role(&self) -> RoleId;
    async fn remove_roles_from_non_authenticated_user(
        &self,
        user_id: UserId,
    ) -> Result<(), AuthenticationError>;
}

#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error(transparent)]
    Error(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("User has already been authenticated")]
    AlreadyAuthenticated,
    #[error("User authentication request was not found")]
    AuthenticationRequestNotFound,
    #[error("Email is already in use by another user")]
    EmailAlreadyInUse,
}

pub struct AuthenticatedUserInfoDto {
    pub user_id: UserId,
    pub name: String,
    pub email: String,
    pub class_id: String,
    pub authenticated_at: chrono::DateTime<chrono::Utc>,
}

pub struct VerifiedUserStatsDto {
    pub total_verified_users: u32,
    pub total_verified_users_with_user_info: u32,
}
