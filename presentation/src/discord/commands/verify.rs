use crate::application_ports::Locator;
use crate::discord::{response, Context, Error};
use application_ports::authentication::AuthenticationError;
use application_ports::authentication::AuthenticationPort;
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

    let mut authentication_port = ctx.data().create_authentication_port();

    let user = ctx.author();

    let response = match authentication_port
        .create_authentication_link(UserId(user.id.get()))
        .await
    {
        Ok(link) => {
            let msg = response::authentication_link::authentication_link(link, user);
            let msg = user.direct_message(ctx.http(), msg).await?;
            response::authentication_link::tell_user_direct_message_sent(user, &msg.link())
        }
        Err(AuthenticationError::TemporaryUnavailable) => {
            response::unavailable::temporary_unavailable()
        }
        Err(AuthenticationError::AuthenticationRequestNotFound) => {
            error!(
                "Unreachable: Got authentication request not found error when creating an authentication request",
            );
            response::unavailable::temporary_unavailable()
        }
        Err(AuthenticationError::AuthenticationRequestAlreadyConfirmed) => {
            error!(
                user_id = user.id.get(),
                "Unreachable: Got authentication request already confirmed error when creating an authentication request",
            );
            response::unavailable::temporary_unavailable()
        }
    };

    ctx.send(response).await?;

    Ok(())
}
