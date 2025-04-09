use async_trait::async_trait;
use domain_shared::discord::ChannelId;
use tracing::{error, instrument};

#[async_trait]
pub trait InformationChannelPort {
    async fn update_information(
        &self,
        channel_id: ChannelId,
    ) -> Result<(), InformationChannelError>;
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
