use crate::application_ports::Locator;
use crate::discord::{Context, Error};
use application_ports::discord::ChannelId;
use application_ports::information_channel::InformationChannelError;
use poise::CreateReply;
use tracing::{info, instrument};

#[poise::command(
    slash_command,
    rename = "update-information",
    required_permissions = "ADMINISTRATOR"
)]
#[instrument(level = "info", skip(ctx))]
pub async fn command<D: Sync + Locator>(ctx: Context<'_, D>) -> Result<(), Error> {
    info!(
        guild_id = ctx.guild_id().map(|id| id.get()),
        channel_id = ctx.channel_id().get(),
        user_id = ctx.author().id.get(),
        "Updating information channel",
    );

    let information_channel_port = ctx.data().get_information_channel_port();
    ctx.defer_ephemeral().await?;

    information_channel_port
        .update_information(ChannelId(ctx.channel_id().get()))
        .await
        .map_err(|e| match e {
            InformationChannelError::Error(error) => error,
        })?;

    ctx.send(
        CreateReply::default()
            .content("Information channel updated!")
            .ephemeral(true)
            .reply(true),
    )
    .await?;

    Ok(())
}
