use async_trait::async_trait;
use thiserror::Error;
use tracing::error;

#[async_trait]
pub trait UserInfoSyncJobHandlerPort {
    async fn tick(&self) -> Result<(), UserInfoSyncJobHandlerError>;
}

#[derive(Debug, Error)]
pub enum UserInfoSyncJobHandlerError {
    #[error("No request to handle")]
    NoRequestToHandle,
    #[error("Service is temporarily unavailable")]
    TemporaryUnavailable,
}
