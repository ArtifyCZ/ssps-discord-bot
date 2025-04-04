mod channel_id;
mod create_attachment;
mod create_message;

use crate::discord::channel_id::domain_to_serenity_channel_id;
use crate::discord::create_message::domain_to_serenity_create_message;
use async_trait::async_trait;
use domain::ports::discord::Result;
use domain::ports::discord::{ChannelId, CreateMessage, DiscordPort};
use poise::serenity_prelude as serenity;
use serenity::all::{Builder, Http};
use serenity::futures::StreamExt;
use std::sync::Arc;

pub struct DiscordAdapter {
    client: Arc<Http>,
}

impl DiscordAdapter {
    pub fn new(client: Arc<Http>) -> Self {
        Self { client }
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
}
