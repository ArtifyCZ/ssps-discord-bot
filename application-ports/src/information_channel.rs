use domain_shared::discord::ChannelId;
use std::future::Future;
use tracing::{error, instrument};

pub trait InformationChannelPort {
    fn update_information(
        &self,
        channel_id: ChannelId,
    ) -> impl Future<Output = Result<(), InformationChannelError>> + Send;
}

pub enum InformationChannelError {
    Error(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for InformationChannelError {
    #[instrument(level = "trace", skip(e))]
    fn from(e: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        error!(error = e, "Information channel error");
        InformationChannelError::Error(e)
    }
}
