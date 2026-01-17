use std::future::Future;
use thiserror::Error;

pub trait RoleSyncJobHandlerPort {
    fn tick(&mut self) -> impl Future<Output = Result<(), RoleSyncJobHandlerError>> + Send;
}

#[derive(Debug, Error)]
pub enum RoleSyncJobHandlerError {
    #[error("No request to handle")]
    NoRequestToHandle,
    #[error("Service is temporarily unavailable")]
    TemporaryUnavailable,
}
