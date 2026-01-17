use chrono::Duration;
use domain_shared::discord::UserId;
use std::future::Future;
use thiserror::Error;
use tracing::error;

pub trait UserPort {
    fn get_user_info(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = Result<Option<AuthenticatedUserInfoDto>, UserError>> + Send;
    fn refresh_user_roles(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = Result<(), UserError>> + Send;
    fn refresh_user_info(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = Result<Duration, UserError>> + Send;
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
