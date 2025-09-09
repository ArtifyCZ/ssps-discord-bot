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
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait DiscordPort {
    async fn send_message(&self, channel_id: ChannelId, message: CreateMessage) -> Result<()>;

    async fn purge_messages(&self, channel_id: ChannelId) -> Result<()>;

    async fn assign_roles_to_user_if_not_assigned(
        &self,
        user_id: UserId,
        role_ids: &[RoleId],
        reason: &str,
    ) -> Result<()>;

    async fn remove_user_from_roles(
        &self,
        user_id: UserId,
        role_id: &[RoleId],
        reason: &str,
    ) -> Result<()>;

    /// This function should assign the provided classes' role to the user and remove the user from
    /// all other class roles if the user has any other assigned. This function should NOT make any
    /// changes if the user is already assigned to the provided class role only. Please note that
    /// if the provided class id is None, the user will be removed from all the class roles.
    async fn set_user_class_role<'a>(
        &self,
        user_id: UserId,
        class_id: Option<&'a str>,
        reason: &str,
    ) -> Result<()>;
}
