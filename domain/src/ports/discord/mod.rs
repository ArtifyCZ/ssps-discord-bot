mod create_action_row;
mod create_attachment;
mod create_button;
mod create_message;

use async_trait::async_trait;
pub use create_action_row::CreateActionRow;
pub use create_attachment::CreateAttachment;
pub use create_button::{ButtonId, ButtonKind, CreateButton};
pub use create_message::CreateMessage;
pub use domain_shared::discord::ChannelId;
use domain_shared::discord::{RoleId, UserId};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait DiscordPort {
    async fn send_message(&self, channel_id: ChannelId, message: CreateMessage) -> Result<()>;
    async fn purge_messages(&self, channel_id: ChannelId) -> Result<()>;
    async fn assign_user_to_role(
        &self,
        user_id: UserId,
        role_id: RoleId,
        reason: Option<&str>,
    ) -> Result<()>;
    async fn remove_user_from_role(
        &self,
        user_id: UserId,
        role_id: RoleId,
        reason: Option<&str>,
    ) -> Result<()>;
    async fn assign_user_to_class_role(
        &self,
        user_id: UserId,
        class_id: &str,
        reason: Option<&str>,
    ) -> Result<()>;
    async fn remove_user_from_class_roles(
        &self,
        user_id: UserId,
        reason: Option<&str>,
    ) -> Result<()>;
}
