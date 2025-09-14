mod create_action_row;
mod create_attachment;
mod create_button;
mod create_message;
mod role;

use async_trait::async_trait;
pub use create_action_row::CreateActionRow;
pub use create_attachment::CreateAttachment;
pub use create_button::{ButtonId, ButtonKind, CreateButton};
pub use create_message::CreateMessage;
pub use domain_shared::discord::ChannelId;
use domain_shared::discord::{RoleId, UserId};
pub use role::Role;
use thiserror::Error;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait DiscordPort {
    async fn send_message(&self, channel_id: ChannelId, message: CreateMessage) -> Result<()>;

    async fn purge_messages(&self, channel_id: ChannelId) -> Result<()>;

    async fn assign_user_role(
        &self,
        user_id: UserId,
        role_id: RoleId,
        reason: &str,
    ) -> Result<(), DiscordError>;

    async fn remove_user_role(
        &self,
        user_id: UserId,
        role_id: RoleId,
        reason: &str,
    ) -> Result<(), DiscordError>;

    async fn find_user_roles(&self, user_id: UserId) -> Result<Vec<Role>, DiscordError>;

    async fn find_role_name(&self, role_id: RoleId) -> Result<Option<String>, DiscordError>;

    async fn find_class_role(&self, class_id: &str) -> Result<Option<RoleId>, DiscordError>;
}

#[derive(Debug, Error)]
pub enum DiscordError {
    #[error("Discord is unavailable")]
    DiscordUnavailable,
}
