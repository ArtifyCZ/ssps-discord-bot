mod channel_id;
mod create_attachment;
mod create_message;
mod user_id;

use crate::discord::channel_id::domain_to_serenity_channel_id;
use crate::discord::create_message::domain_to_serenity_create_message;
use crate::discord::user_id::domain_to_serenity_user_id;
use async_trait::async_trait;
use domain::ports::discord::Result;
use domain::ports::discord::{ChannelId, CreateMessage, DiscordPort};
use domain_shared::discord::UserId;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::GuildId;
use serenity::all::{Builder, Http};
use serenity::futures::StreamExt;
use std::sync::Arc;

pub struct DiscordAdapter {
    client: Arc<Http>,
    guild_id: GuildId,
}

impl DiscordAdapter {
    pub fn new(client: Arc<Http>, guild_id: GuildId) -> Self {
        Self { client, guild_id }
    }
}

#[async_trait]
impl DiscordPort for DiscordAdapter {
    async fn send_message(&self, channel_id: ChannelId, message: CreateMessage) -> Result<()> {
        let message = domain_to_serenity_create_message(message);
        let channel_id = domain_to_serenity_channel_id(channel_id);

        message.execute(&self.client, (channel_id, None)).await?;

        Ok(())
    }

    async fn purge_messages(&self, channel_id: ChannelId) -> Result<()> {
        let channel_id = domain_to_serenity_channel_id(channel_id);

        let mut messages = channel_id.messages_iter(&self.client).boxed();

        while let Some(message) = messages.next().await {
            let message = message?;
            message.delete(&self.client).await?;
        }

        Ok(())
    }

    async fn assign_user_to_class_role(&self, user_id: UserId, class_id: String) -> Result<()> {
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
            .add_member_role(
                self.guild_id,
                user_id,
                role.id,
                Some("Assigned class role per authentication using Azure AD OAuth2"),
            )
            .await?;

        Ok(())
    }
}
