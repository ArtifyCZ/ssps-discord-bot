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
use poise::serenity_prelude::{GuildId, Member};
use serenity::all::{Builder, Http};
use serenity::futures::StreamExt;
use std::ops::Not;
use std::sync::Arc;
use tracing::{error, instrument};

pub struct DiscordAdapter {
    client: Arc<Http>,
    guild_id: GuildId,
    class_ids: Vec<String>,
}

impl DiscordAdapter {
    #[instrument(level = "trace", skip_all)]
    pub fn new(client: Arc<Http>, guild_id: GuildId) -> Self {
        let class_ids = create_class_user_group_id_mails()
            .into_iter()
            .map(|(class_id, _)| class_id)
            .collect();

        Self {
            client,
            guild_id,
            class_ids,
        }
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

    #[instrument(level = "debug", err, skip_all)]
    async fn assign_roles_to_user_if_not_assigned(
        &self,
        user_id: UserId,
        role_ids: &[RoleId],
        reason: &str,
    ) -> Result<()> {
        let user_id = domain_to_serenity_user_id(user_id);

        let member = self.client.get_member(self.guild_id, user_id).await?;

        for role_id in role_ids {
            let role_id = domain_to_serenity_role_id(*role_id);
            assign_user_to_role_if_not_already_assigned(&self.client, &member, role_id, reason)
                .await?;
        }

        Ok(())
    }

    #[instrument(level = "debug", err, skip_all)]
    async fn remove_user_from_roles(
        &self,
        user_id: UserId,
        role_id: &[RoleId],
        reason: &str,
    ) -> Result<()> {
        let user_id = domain_to_serenity_user_id(user_id);

        let member = self.client.get_member(self.guild_id, user_id).await?;

        for role_id in role_id {
            let role_id = domain_to_serenity_role_id(*role_id);
            if member.roles.contains(&role_id) {
                self.client
                    .remove_member_role(self.guild_id, user_id, role_id, Some(reason))
                    .await?;
            }
        }

        Ok(())
    }

    #[instrument(level = "debug", err, skip(self, user_id, class_id, reason))]
    async fn set_user_class_role<'a>(
        &self,
        user_id: UserId,
        class_id: Option<&'a str>,
        reason: &str,
    ) -> Result<()> {
        let user_id = domain_to_serenity_user_id(user_id);
        let member = self.client.get_member(self.guild_id, user_id).await?;
        let class_id_role_id_pairs =
            fetch_class_id_role_id_pairs(&self.client, self.guild_id, self.class_ids.as_slice())
                .await?;
        let desired_class_id = class_id;

        let assigned_class_id_roles: Vec<(&str, serenity::RoleId)> = class_id_role_id_pairs
            .iter()
            .filter(|(_class_id, role_id)| member.roles.contains(role_id))
            .map(|(class_id, role_id)| (*class_id, *role_id))
            .collect();

        for (class_id, role_id) in assigned_class_id_roles.iter() {
            if let Some(desired_class_id) = desired_class_id {
                if *class_id == desired_class_id {
                    continue;
                }
            }

            self.client
                .remove_member_role(self.guild_id, user_id, *role_id, Some(reason))
                .await?;
        }

        if let Some(desired_class_id) = desired_class_id {
            let (_class_id, desired_role_id) = class_id_role_id_pairs
                .iter()
                .find(|(class_id, _role_id)| *class_id == desired_class_id)
                .ok_or(format!("Class role {} not found", desired_class_id))?;

            assign_user_to_role_if_not_already_assigned(
                &self.client,
                &member,
                *desired_role_id,
                reason,
            )
            .await?;
        }

        Ok(())
    }
}

#[instrument(level = "debug", err, skip_all)]
async fn assign_user_to_role_if_not_already_assigned(
    client: &Http,
    member: &Member,
    role_id: serenity::RoleId,
    reason: &str,
) -> Result<()> {
    if member.roles.contains(&role_id).not() {
        client
            .add_member_role(member.guild_id, member.user.id, role_id, Some(reason))
            .await?;
    }

    Ok(())
}

#[instrument(level = "debug", err, skip_all)]
async fn fetch_class_id_role_id_pairs<'a>(
    client: &Http,
    guild_id: GuildId,
    class_ids: &'a [String],
) -> Result<Vec<(&'a str, serenity::RoleId)>> {
    let role_id_name_pairs = fetch_role_id_name_pairs(client, guild_id).await?;
    let mut class_id_role_id_pairs: Vec<(&str, poise::serenity_prelude::RoleId)> =
        Vec::with_capacity(class_ids.len());

    for class_id in class_ids {
        if let Some((role_id, _)) = role_id_name_pairs
            .iter()
            .find(|(_, name)| name.eq_ignore_ascii_case(class_id))
        {
            class_id_role_id_pairs.push((class_id.as_str(), *role_id));
        } else {
            error!("Role with name {} not found", class_id);
        }
    }

    Ok(class_id_role_id_pairs)
}

#[instrument(level = "debug", err, skip_all)]
async fn fetch_role_id_name_pairs(
    client: &Http,
    guild_id: GuildId,
) -> Result<Vec<(serenity::RoleId, String)>> {
    let roles = client.get_guild_roles(guild_id).await?;
    Ok(roles.into_iter().map(|role| (role.id, role.name)).collect())
}
