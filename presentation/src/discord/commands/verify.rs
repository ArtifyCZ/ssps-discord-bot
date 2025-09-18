use crate::application_ports::Locator;
use crate::discord::{response, Context, Error};
use application_ports::authentication::AuthenticationError;
use domain_shared::discord::UserId;
use tracing::{error, info, instrument};

#[poise::command(
    slash_command,
    rename = "verify",
    required_permissions = "ADMINISTRATOR"
)]
#[instrument(level = "info", skip(ctx))]
pub async fn command<D: Sync + Locator>(ctx: Context<'_, D>) -> Result<(), Error> {
    info!(
        guild_id = ctx.guild_id().map(|id| id.get()),
        channel_id = ctx.channel_id().get(),
        user_id = ctx.author().id.get(),
        "Creating authentication link",
    );

    let authentication_port = ctx.data().get_authentication_port();

    let user = ctx.author();

    let response = match authentication_port
        .create_authentication_link(UserId(user.id.get()))
        .await
    {
        Ok(link) => response::authentication_link(link, user),
        Err(AuthenticationError::TemporaryUnavailable) => response::temporary_unavailable(),
        Err(AuthenticationError::Error(error)) => {
            error!(
                error = ?error,
                "An unknown error occurred while creating authentication link",
            );
            response::temporary_unavailable()
        }
        Err(AuthenticationError::AuthenticationRequestNotFound) => {
            error!(
                "Unreachable: Got authentication request not found error when creating an authentication request",
            );
            response::temporary_unavailable()
        }
    };

    ctx.send(response).await?;

    Ok(())
}
