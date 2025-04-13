mod channel_id;
mod create_attachment;
mod create_button;
mod create_message;
mod role_id;
mod user_id;

use crate::discord::channel_id::domain_to_serenity_channel_id;
use crate::discord::create_message::domain_to_serenity_create_message;
use crate::discord::role_id::domain_to_serenity_role_id;
use crate::discord::user_id::domain_to_serenity_user_id;
use async_trait::async_trait;
use domain::authentication::create_class_user_group_id_mails;
use domain::ports::discord::Result;
use domain::ports::discord::{ChannelId, CreateMessage, DiscordPort};
use domain_shared::discord::{RoleId, UserId};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::GuildId;
use serenity::all::{Builder, Http};
use serenity::futures::StreamExt;
use std::sync::Arc;
use tracing::instrument;

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

    #[instrument(level = "debug", err, skip(self, user_id, role_id, reason))]
    async fn assign_user_to_role(
        &self,
        user_id: UserId,
        role_id: RoleId,
        reason: Option<&str>,
    ) -> Result<()> {
        let user_id = domain_to_serenity_user_id(user_id);
        let role_id = domain_to_serenity_role_id(role_id);

        self.client
            .add_member_role(self.guild_id, user_id, role_id, reason)
            .await?;

        Ok(())
    }

    #[instrument(level = "debug", err, skip(self, user_id, role_id, reason))]
    async fn remove_user_from_role(
        &self,
        user_id: UserId,
        role_id: RoleId,
        reason: Option<&str>,
    ) -> Result<()> {
        let user_id = domain_to_serenity_user_id(user_id);
        let role_id = domain_to_serenity_role_id(role_id);

        self.client
            .remove_member_role(self.guild_id, user_id, role_id, reason)
            .await?;

        Ok(())
    }

    #[instrument(level = "debug", err, skip(self, user_id, class_id, reason))]
    async fn assign_user_to_class_role(
        &self,
        user_id: UserId,
        class_id: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        let user_id = domain_to_serenity_user_id(user_id);
        let class_id = class_id.to_uppercase();

        let role = self
            .client
            .get_guild_roles(self.guild_id)
            .await?
            .into_iter()
            .find(|role| role.name == class_id)
            .ok_or_else(|| format!("Role with name {} not found", class_id))?;

        self.client
            .add_member_role(self.guild_id, user_id, role.id, reason)
            .await?;

        Ok(())
    }

    #[instrument(level = "debug", err, skip(self, user_id, reason))]
    async fn remove_user_from_class_roles(
        &self,
        user_id: UserId,
        reason: Option<&str>,
    ) -> Result<()> {
        let user_id = domain_to_serenity_user_id(user_id);

        let class_ids: Vec<String> = create_class_user_group_id_mails()
            .into_iter()
            .map(|(class_id, _)| class_id)
            .collect();

        let roles = self
            .client
            .get_guild_roles(self.guild_id)
            .await?
            .into_iter()
            .filter(|role| class_ids.contains(&role.name))
            .map(|role| role.id)
            .collect::<Vec<_>>();

        for role in roles {
            self.client
                .remove_member_role(self.guild_id, user_id, role, reason)
                .await?;
        }

        Ok(())
    }
}
