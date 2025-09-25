use crate::application_ports::Locator;
use crate::discord::{Context, Error};
use application_ports::user::AuthenticatedUserInfoDto;
use domain_shared::discord::UserId;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{CreateEmbed, Mentionable};
use poise::CreateReply;
use tracing::{error, info, instrument, warn};

#[poise::command(
    slash_command,
    rename = "user-info",
    required_permissions = "ADMINISTRATOR"
)]
#[instrument(level = "info", skip(ctx))]
pub async fn command<D: Sync + Locator>(
    ctx: Context<'_, D>,
    #[description = "Selected target"] target: serenity::User,
    #[description = "Force refresh user info (default false)"] force_refresh: Option<bool>,
) -> Result<(), Error> {
    let user_port = ctx.data().get_user_port();

    info!(
        guild_id = ctx.guild_id().map(|id| id.get()),
        user_id = ctx.author().id.get(),
        "Accessing user info",
    );

    let user_id = UserId(target.id.get());
    let force_refresh = force_refresh.unwrap_or(false);

    if force_refresh {
        ctx.defer_ephemeral().await?;
        let wait_for_sync_duration = user_port.refresh_user_info(user_id).await?;
        tokio::time::sleep(wait_for_sync_duration.to_std().unwrap()).await;
    }

    let user_info = match user_port.get_user_info(user_id).await {
        Ok(user_info) => user_info,
        Err(error) => {
            let reply = CreateReply::default()
                .reply(true)
                .ephemeral(true)
                .content("An error occurred while fetching user info.");
            if let Err(error2) = ctx.send(reply).await {
                error!(error2 = %error2, "Failed to send error message");
            }
            warn!(error = ?error, "Failed to fetch user info");
            return Err(Box::new(error));
        }
    };

    let embed = match user_info {
        Some(AuthenticatedUserInfoDto {
            user_id,
            name,
            email,
            class_id,
            authenticated_at,
        }) => CreateEmbed::default()
            .title("Ověřený student".to_string())
            .thumbnail(target.face())
            .fields(vec![
                ("", target.mention().to_string(), false),
                ("User ID", user_id.0.to_string(), false),
                ("Jméno", name, false),
                ("Email", email.to_string(), false),
                (
                    "Třída",
                    class_id.unwrap_or_else(|| "N/A".to_string()),
                    false,
                ),
                ("Ověřen", authenticated_at.to_rfc2822(), false),
            ]),
        None => CreateEmbed::default()
            .title("Neověřený uživatel".to_string())
            .thumbnail(target.face())
            .fields(vec![
                ("", target.mention().to_string(), false),
                ("User ID", target.id.to_string(), false),
            ]),
    };

    let reply = CreateReply::default()
        .reply(true)
        .ephemeral(true)
        .embed(embed);
    ctx.send(reply).await?;

    Ok(())
}
