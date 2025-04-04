use async_trait::async_trait;
use domain_shared::discord::ChannelId;

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
    fn from(e: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        InformationChannelError::Error(e)
    }
}
