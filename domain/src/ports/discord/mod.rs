mod create_action_row;
mod create_attachment;
mod create_button;
mod create_message;
mod role;
mod role_diff;

pub use create_action_row::CreateActionRow;
pub use create_attachment::CreateAttachment;
pub use create_button::{ButtonId, ButtonKind, CreateButton};
pub use create_message::CreateMessage;
pub use domain_shared::discord::ChannelId;
use domain_shared::discord::{RoleId, UserId};
pub use role::Role;
pub use role_diff::RoleDiff;
use std::future::Future;
use thiserror::Error;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait DiscordPort {
    fn send_message(
        &self,
        channel_id: ChannelId,
        message: CreateMessage,
    ) -> impl Future<Output = Result<()>> + Send;

    fn purge_messages(&self, channel_id: ChannelId) -> impl Future<Output = Result<()>> + Send;

    fn find_or_create_role_by_name(
        &self,
        role_name: &str,
        reason: &str,
    ) -> impl Future<Output = Result<Role, DiscordError>> + Send;

    fn apply_role_diff(
        &self,
        user_id: UserId,
        role_diff: &RoleDiff,
        reason: &str,
    ) -> impl Future<Output = Result<(), DiscordError>> + Send;

    fn find_user_roles(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = Result<Option<Vec<Role>>, DiscordError>> + Send;

    fn find_role_name(
        &self,
        role_id: RoleId,
    ) -> impl Future<Output = Result<Option<String>, DiscordError>> + Send;

    fn find_class_role(
        &self,
        class_id: &str,
    ) -> impl Future<Output = Result<Option<RoleId>, DiscordError>> + Send;

    fn find_all_members(
        &self,
        offset: Option<UserId>,
    ) -> impl Future<Output = Result<Option<Vec<UserId>>, DiscordError>> + Send;
}

#[derive(Debug, Error)]
pub enum DiscordError {
    #[error("Discord is unavailable")]
    DiscordUnavailable,
}
