use async_trait::async_trait;
use thiserror::Error;
use tracing::error;

#[async_trait]
pub trait RoleSyncJobHandlerPort {
    async fn tick(&mut self) -> Result<(), RoleSyncJobHandlerError>;
}

#[derive(Debug, Error)]
pub enum RoleSyncJobHandlerError {
    #[error("No request to handle")]
    NoRequestToHandle,
    #[error("Service is temporarily unavailable")]
    TemporaryUnavailable,
}
