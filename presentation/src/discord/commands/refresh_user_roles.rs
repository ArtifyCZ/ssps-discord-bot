use crate::application_ports::Locator;
use crate::discord::{response, Context, Error};
use application_ports::user::UserError;
use domain_shared::discord::UserId;
use poise::serenity_prelude as serenity;
use poise::CreateReply;
use tracing::{info, instrument, warn};

#[poise::command(
    slash_command,
    rename = "refresh-user-roles",
    required_permissions = "ADMINISTRATOR"
)]
#[instrument(level = "info", skip(ctx))]
pub async fn command<D: Sync + Locator>(
    ctx: Context<'_, D>,
    #[description = "Selected target"] target: serenity::User,
) -> Result<(), Error> {
    let user_port = ctx.data().get_user_port();

    info!(
        guild_id = ctx.guild_id().map(|id| id.get()),
        user_id = ctx.author().id.get(),
        "Requesting user roles refresh for user {}",
        target.id.get(),
    );

    let user_id = UserId(target.id.get());

    let reply = match user_port.refresh_user_roles(user_id).await {
        Ok(()) => CreateReply::default()
            .reply(true)
            .ephemeral(true)
            .content("User roles refresh successfully requested. In a few minutes, the roles will be updated."),
        Err(UserError::AuthenticatedUserNotFound) | Err(UserError::TemporaryUnavailable)  => {
            warn!(
                "Failed to request user roles refresh for user {}: Service is temporarily unavailable",
                target.id.get(),
            );
            response::unavailable::temporary_unavailable()
        }
    };

    ctx.send(reply).await?;

    Ok(())
}
