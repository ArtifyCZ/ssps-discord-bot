use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait PeriodicSchedulingHandlerPort {
    async fn tick(&mut self) -> Result<(), PeriodicSchedulingHandlerError>;
}

#[derive(Debug, Error)]
pub enum PeriodicSchedulingHandlerError {
    #[error("Service temporarily unavailable")]
    TemporarilyUnavailable,
}
