use async_trait::async_trait;
use domain_shared::discord::UserId;
use thiserror::Error;
use tracing::error;

#[async_trait]
pub trait UserPort {
    async fn get_user_info(
        &self,
        user_id: UserId,
    ) -> Result<Option<AuthenticatedUserInfoDto>, UserError>;
    async fn refresh_user_data(&self, user_id: UserId) -> Result<(), UserError>;
}

#[derive(Debug, Error)]
pub enum UserError {
    #[error(transparent)]
    Error(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("Authenticated user not found")]
    AuthenticatedUserNotFound,
}

pub struct AuthenticatedUserInfoDto {
    pub user_id: UserId,
    pub name: String,
    pub email: String,
    pub class_id: String,
    pub authenticated_at: chrono::DateTime<chrono::Utc>,
}
