use async_trait::async_trait;
use chrono::Duration;
use domain_shared::discord::UserId;
use thiserror::Error;
use tracing::error;

#[async_trait]
pub trait UserPort {
    async fn get_user_info(
        &self,
        user_id: UserId,
    ) -> Result<Option<AuthenticatedUserInfoDto>, UserError>;
    async fn refresh_user_roles(&self, user_id: UserId) -> Result<(), UserError>;
    async fn refresh_user_info(&self, user_id: UserId) -> Result<Duration, UserError>;
}

#[derive(Debug, Error)]
pub enum UserError {
    #[error("Authenticated user not found")]
    AuthenticatedUserNotFound,
    #[error("Service is temporarily unavailable")]
    TemporaryUnavailable,
}

pub struct AuthenticatedUserInfoDto {
    pub user_id: UserId,
    pub name: String,
    pub email: String,
    pub class_id: Option<String>,
    pub authenticated_at: chrono::DateTime<chrono::Utc>,
}
