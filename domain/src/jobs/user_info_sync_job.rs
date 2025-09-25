use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain_shared::discord::UserId;
use thiserror::Error;
use tracing::instrument;

#[derive(Debug)]
pub struct UserInfoSyncRequested {
    pub user_id: UserId,
    pub queued_at: DateTime<Utc>,
    pub low_priority: bool,
}

#[instrument(level = "info")]
pub fn request_user_info_sync(user_id: UserId) -> UserInfoSyncRequested {
    UserInfoSyncRequested {
        user_id,
        queued_at: Utc::now(),
        low_priority: false,
    }
}

#[instrument(level = "debug")]
pub fn request_periodic_user_info_sync(user_id: UserId) -> UserInfoSyncRequested {
    UserInfoSyncRequested {
        user_id,
        queued_at: Utc::now(),
        low_priority: true,
    }
}

#[async_trait]
pub trait UserInfoSyncRequestedRepository {
    async fn save(
        &self,
        request: &UserInfoSyncRequested,
    ) -> Result<(), UserInfoSyncRequestedRepositoryError>;
    async fn pop_oldest(
        &self,
        low_priority: bool,
    ) -> Result<Option<UserInfoSyncRequested>, UserInfoSyncRequestedRepositoryError>;
}

#[derive(Debug, Error)]
pub enum UserInfoSyncRequestedRepositoryError {
    #[error("Service unavailable")]
    ServiceUnavailable,
}
