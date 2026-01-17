mod channel_id;
mod create_attachment;
mod create_button;
mod create_message;
mod role_id;
mod user_id;

use crate::discord::channel_id::domain_to_serenity_channel_id;
use crate::discord::create_message::domain_to_serenity_create_message;
use crate::discord::role_id::{domain_to_serenity_role_id, serenity_to_domain_role_id};
use crate::discord::user_id::{domain_to_serenity_user_id, serenity_to_domain_user_id};
use domain::ports::discord::{ChannelId, CreateMessage, DiscordPort, Role, RoleDiff};
use domain::ports::discord::{DiscordError, Result};
use domain_shared::discord::{RoleId, UserId};
use poise::futures_util::stream::FuturesUnordered;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::GuildId;
use serenity::all::{Builder, Http};
use serenity::futures::StreamExt;
use std::ops::Not;
use tracing::{error, instrument, warn};

pub struct DiscordAdapter<'a> {
    client: &'a Http,
    guild_id: GuildId,
}

impl<'a> DiscordAdapter<'a> {
    #[instrument(level = "trace", skip_all)]
    pub fn new(client: &'a Http, guild_id: GuildId) -> Self {
        Self { client, guild_id }
    }
}

impl<'a> DiscordPort for DiscordAdapter<'a> {
    #[instrument(level = "debug", err, skip(self, channel_id, message))]
    async fn send_message(&self, channel_id: ChannelId, message: CreateMessage) -> Result<()> {
        let message = domain_to_serenity_create_message(message);
        let channel_id = domain_to_serenity_channel_id(channel_id);

        message.execute(self.client, (channel_id, None)).await?;

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
    async fn find_or_create_role_by_name(
        &self,
        role_name: &str,
        reason: &str,
    ) -> Result<Role, DiscordError> {
        let roles = self
            .client
            .get_guild_roles(self.guild_id)
            .await
            .map_err(map_serenity_err)?;

        for role in roles {
            if role.name.eq_ignore_ascii_case(role_name) {
                return Ok(Role {
                    role_id: serenity_to_domain_role_id(role.id),
                    name: role.name,
                });
            }
        }

        let role = self
            .client
            .create_role(
                self.guild_id,
                &serenity::EditRole::new()
                    .name(role_name)
                    .hoist(false)
                    .mentionable(false),
                Some(reason),
            )
            .await
            .map_err(map_serenity_err)?;

        Ok(Role {
            role_id: serenity_to_domain_role_id(role.id),
            name: role.name,
        })
    }

    #[instrument(level = "debug", err, skip_all)]
    async fn apply_role_diff(
        &self,
        user_id: UserId,
        role_diff: &RoleDiff,
        reason: &str,
    ) -> Result<(), DiscordError> {
        let user_id = domain_to_serenity_user_id(user_id);
        let mut assign_set = FuturesUnordered::new();
        let mut remove_set = FuturesUnordered::new();

        for role_id in role_diff.to_assign() {
            let role_id = domain_to_serenity_role_id(*role_id);

            assign_set.push(async move {
                self.client
                    .add_member_role(self.guild_id, user_id, role_id, Some(reason))
                    .await
            });
        }

        for role_id in role_diff.to_remove() {
            let role_id = domain_to_serenity_role_id(*role_id);

            remove_set.push(async move {
                self.client
                    .remove_member_role(self.guild_id, user_id, role_id, Some(reason))
                    .await
            });
        }

        let mut failed = false;
        loop {
            tokio::select! {
                Some(result) = assign_set.next() => {
                    if let Err(err) = result {
                        error!(
                    "An error occurred during role diff of user {:?}: {:?}",
                    user_id,
                    map_serenity_err(err)
                );
                        failed = true;
                    }
                }
                Some(result) = remove_set.next() => {
                    if let Err(err) = result {
                        error!(
                    "An error occurred during role diff of user {:?}: {:?}",
                    user_id,
                    map_serenity_err(err)
                );
                        failed = true;
                    }
                }
                else => {
                    break;
                }
            }
        }

        if failed.not() {
            Ok(())
        } else {
            Err(DiscordError::DiscordUnavailable)
        }
    }

    #[instrument(level = "debug", err, skip_all)]
    async fn find_user_roles(&self, user_id: UserId) -> Result<Option<Vec<Role>>, DiscordError> {
        let user_id = domain_to_serenity_user_id(user_id);
        let mut set = FuturesUnordered::new();

        let member = match self.client.get_member(self.guild_id, user_id).await {
            Ok(member) => member,
            Err(err) => {
                if let serenity::Error::Http(serenity::http::HttpError::UnsuccessfulRequest(err)) =
                    &err
                {
                    if err.status_code == 404 {
                        return Ok(None);
                    }
                }
                return Err(map_serenity_err(err));
            }
        };

        for role_id in member.roles.iter() {
            let role_id = *role_id;

            set.push(async move {
                match self.client.get_guild_role(self.guild_id, role_id).await {
                    Ok(role) => Ok(Role {
                        role_id: serenity_to_domain_role_id(role.id),
                        name: role.name,
                    }),
                    Err(error) => {
                        warn!(
                            "Failed to fetch role {} from guild {}: {:?}",
                            role_id, self.guild_id, error,
                        );
                        Err(DiscordError::DiscordUnavailable)
                    }
                }
            });
        }

        let mut roles = vec![];
        while let Some(role) = set.next().await {
            roles.push(role?);
        }

        Ok(Some(roles))
    }

    #[instrument(level = "debug", err, skip_all)]
    async fn find_role_name(&self, role_id: RoleId) -> Result<Option<String>, DiscordError> {
        let role_id = domain_to_serenity_role_id(role_id);

        let role = self
            .client
            .get_guild_role(self.guild_id, role_id)
            .await
            .map_err(map_serenity_err)?;

        Ok(Some(role.name))
    }

    #[instrument(level = "debug", err, skip_all)]
    async fn find_class_role(&self, class_id: &str) -> Result<Option<RoleId>, DiscordError> {
        let roles = self
            .client
            .get_guild_roles(self.guild_id)
            .await
            .map_err(map_serenity_err)?;

        let class_role = roles
            .iter()
            .find(|role| role.name.eq_ignore_ascii_case(class_id));
        let class_role_id = class_role.map(|role| serenity_to_domain_role_id(role.id));

        Ok(class_role_id)
    }

    #[instrument(level = "debug", err, skip_all)]
    async fn find_all_members(
        &self,
        offset: Option<UserId>,
    ) -> Result<Option<Vec<UserId>>, DiscordError> {
        let offset = offset.map(|offset| offset.0);

        let members = self
            .client
            .get_guild_members(self.guild_id, None, offset)
            .await
            .map_err(map_serenity_err)?;
        if members.is_empty() {
            return Ok(None);
        }

        let member_ids = members
            .into_iter()
            .map(|m| serenity_to_domain_user_id(m.user.id))
            .collect();
        Ok(Some(member_ids))
    }
}

#[instrument(level = "trace", skip_all)]
fn map_serenity_err(err: serenity::Error) -> DiscordError {
    error!("Serenity error occurred: {:?}", err);
    DiscordError::DiscordUnavailable
}
