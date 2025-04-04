use crate::application_ports::Locator;
use crate::discord::{Context, Error};
use application_ports::discord::ChannelId;
use application_ports::information_channel::InformationChannelError;

#[poise::command(
    slash_command,
    rename = "update-information",
    required_permissions = "ADMINISTRATOR"
)]
pub async fn command<D: Sync + Locator>(ctx: Context<'_, D>) -> Result<(), Error> {
    let information_channel_port = ctx.data().get_information_channel_port();
    ctx.defer().await?;

    information_channel_port
        .update_information(ChannelId(ctx.channel_id().get()))
        .await
        .map_err(|e| match e {
            InformationChannelError::Error(error) => error,
        })?;

    Ok(())
}
