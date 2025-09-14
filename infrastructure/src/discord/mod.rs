mod channel_id;
mod create_attachment;
mod create_button;
mod create_message;
mod role_id;
mod user_id;

use crate::discord::channel_id::domain_to_serenity_channel_id;
use crate::discord::create_message::domain_to_serenity_create_message;
use crate::discord::role_id::{domain_to_serenity_role_id, serenity_to_domain_role_id};
use crate::discord::user_id::domain_to_serenity_user_id;
use async_trait::async_trait;
use domain::ports::discord::{ChannelId, CreateMessage, DiscordPort};
use domain::ports::discord::{DiscordError, Result};
use domain_shared::discord::{RoleId, UserId};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::GuildId;
use serenity::all::{Builder, Http};
use serenity::futures::StreamExt;
use std::sync::Arc;
use tracing::{instrument, warn};

pub struct DiscordAdapter {
    client: Arc<Http>,
    guild_id: GuildId,
}

impl DiscordAdapter {
    #[instrument(level = "trace", skip_all)]
    pub fn new(client: Arc<Http>, guild_id: GuildId) -> Self {
        Self { client, guild_id }
    }
}

#[async_trait]
impl DiscordPort for DiscordAdapter {
    #[instrument(level = "debug", err, skip(self, channel_id, message))]
    async fn send_message(&self, channel_id: ChannelId, message: CreateMessage) -> Result<()> {
        let message = domain_to_serenity_create_message(message);
        let channel_id = domain_to_serenity_channel_id(channel_id);

        message.execute(&self.client, (channel_id, None)).await?;

        Ok(())
    }

    #[instrument(level = "debug", err, skip(self, channel_id))]
    async fn purge_messages(&self, channel_id: ChannelId) -> Result<()> {
        let channel_id = domain_to_serenity_channel_id(channel_id);

        let mut messages = channel_id.messages_iter(&self.client).boxed();

        while let Some(message) = messages.next().await {
            let message = message?;
            message.delete(&self.client).await?;
        }

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    async fn assign_user_role(
        &self,
        user_id: UserId,
        role_id: RoleId,
        reason: &str,
    ) -> Result<(), DiscordError> {
        let user_id = domain_to_serenity_user_id(user_id);
        let role_id = domain_to_serenity_role_id(role_id);

        self.client
            .add_member_role(self.guild_id, user_id, role_id, Some(reason))
            .await
            .map_err(|err| {
                warn!(
                    "Failed to assign role {} to user {}: {}",
                    role_id, user_id, err,
                );
                DiscordError::DiscordUnavailable
            })?;

        Ok(())
    }

    #[instrument(level = "debug", err, skip_all)]
    async fn remove_user_role(
        &self,
        user_id: UserId,
        role_id: RoleId,
        reason: &str,
    ) -> Result<(), DiscordError> {
        let user_id = domain_to_serenity_user_id(user_id);
        let role_id = domain_to_serenity_role_id(role_id);

        self.client
            .remove_member_role(self.guild_id, user_id, role_id, Some(reason))
            .await
            .map_err(|err| {
                warn!(
                    "Failed to remove role {} from user {}: {}",
                    role_id, user_id, err,
                );
                DiscordError::DiscordUnavailable
            })?;

        Ok(())
    }

    #[instrument(level = "debug", err, skip_all)]
    async fn find_user_roles(&self, user_id: UserId) -> Result<Vec<RoleId>, DiscordError> {
        let user_id = domain_to_serenity_user_id(user_id);

        let member = self
            .client
            .get_member(self.guild_id, user_id)
            .await
            .map_err(|err| {
                warn!(
                    "Failed to fetch member {} from guild {}: {}",
                    user_id, self.guild_id, err,
                );
                DiscordError::DiscordUnavailable
            })?;

        Ok(member
            .roles
            .into_iter()
            .map(serenity_to_domain_role_id)
            .collect())
    }

    #[instrument(level = "debug", err, skip_all)]
    async fn find_role_name(&self, role_id: RoleId) -> Result<Option<String>, DiscordError> {
        let role_id = domain_to_serenity_role_id(role_id);

        let role = self
            .client
            .get_guild_role(self.guild_id, role_id)
            .await
            .map_err(|err| {
                warn!(
                    "Failed to fetch role {} from guild {}: {}",
                    role_id, self.guild_id, err,
                );
                DiscordError::DiscordUnavailable
            })?;

        Ok(Some(role.name))
    }

    #[instrument(level = "debug", err, skip_all)]
    async fn find_class_role(&self, class_id: &str) -> Result<Option<RoleId>, DiscordError> {
        let roles = self
            .client
            .get_guild_roles(self.guild_id)
            .await
            .map_err(|err| {
                warn!(
                    "Failed to fetch roles from guild {}: {}",
                    self.guild_id, err,
                );
                DiscordError::DiscordUnavailable
            })?;

        let class_role = roles
            .iter()
            .find(|role| role.name.eq_ignore_ascii_case(class_id));
        let class_role_id = class_role.map(|role| serenity_to_domain_role_id(role.id));

        Ok(class_role_id)
    }
}
