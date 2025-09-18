use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain_shared::discord::UserId;
use thiserror::Error;
use tracing::instrument;

#[derive(Debug)]
pub struct RoleSyncRequested {
    pub user_id: UserId,
    pub queued_at: DateTime<Utc>,
    pub low_priority: bool,
}

#[instrument(level = "info")]
pub fn request_role_sync(user_id: UserId) -> RoleSyncRequested {
    RoleSyncRequested {
        user_id,
        queued_at: Utc::now(),
        low_priority: false,
    }
}

#[instrument(level = "debug")]
pub fn request_periodic_role_sync(user_id: UserId) -> RoleSyncRequested {
    RoleSyncRequested {
        user_id,
        queued_at: Utc::now(),
        low_priority: true,
    }
}

#[async_trait]
pub trait RoleSyncRequestedRepository {
    async fn save(
        &self,
        request: &RoleSyncRequested,
    ) -> Result<(), RoleSyncRequestedRepositoryError>;
    async fn pop_oldest(
        &self,
        low_priority: bool,
    ) -> Result<Option<RoleSyncRequested>, RoleSyncRequestedRepositoryError>;
}

#[derive(Debug, Error)]
pub enum RoleSyncRequestedRepositoryError {
    #[error("Service unavailable")]
    ServiceUnavailable,
}
