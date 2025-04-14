use crate::application_ports::Locator;
use crate::discord::{Context, Error};
use application_ports::authentication::AuthenticationError;
use poise::serenity_prelude::CreateEmbed;
use poise::CreateReply;
use tracing::{info, instrument};

#[poise::command(
    slash_command,
    rename = "verified-user-stats",
    required_permissions = "ADMINISTRATOR"
)]
#[instrument(level = "info", skip(ctx))]
pub async fn command<D: Sync + Locator>(
    ctx: Context<'_, D>,
    #[description = "Should the bot load names and emails?"] load_names: bool,
) -> Result<(), Error> {
    info!(
        guild_id = ctx.guild_id().map(|id| id.get()),
        channel_id = ctx.channel_id().get(),
        user_id = ctx.author().id.get(),
        "Generating verified user stats",
    );

    let authentication_port = ctx.data().get_authentication_port();
    ctx.defer().await?;

    let stats = authentication_port
        .get_verified_user_stats(load_names)
        .await
        .map_err(|e| match e {
            AuthenticationError::Error(error) => error,
            AuthenticationError::AlreadyAuthenticated
            | AuthenticationError::AuthenticationRequestNotFound => unreachable!(),
        })?;

    let embed = CreateEmbed::default()
        .title("Ověření studenti".to_string())
        .fields(vec![
            (
                "Celkový počet ověřených studentů",
                stats.total_verified_users.to_string(),
                false,
            ),
            (
                "Celkový počet ověřených studentů s osobními údaji",
                stats.total_verified_users_with_user_info.to_string(),
                false,
            ),
        ]);
    let reply = CreateReply::default()
        .embed(embed)
        .reply(true)
        .ephemeral(true);
    ctx.send(reply).await?;

    Ok(())
}
