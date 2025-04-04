mod create_attachment;
mod create_message;

use async_trait::async_trait;
pub use create_attachment::CreateAttachment;
pub use create_message::CreateMessage;
pub use domain_shared::discord::ChannelId;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait DiscordPort {
    async fn send_message(&self, channel_id: ChannelId, message: CreateMessage) -> Result<()>;
    async fn purge_messages(&self, channel_id: ChannelId) -> Result<()>;
}
