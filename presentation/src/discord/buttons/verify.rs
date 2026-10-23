use crate::application_ports::Locator;
use crate::discord::{Error, response};
use application_ports::authentication::AuthenticationError;
use application_ports::authentication::AuthenticationPort;
use domain_shared::discord::UserId;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::CacheHttp;
use tracing::{error, info, instrument};

pub const BUTTON_ID: &str = domain::information_channel::VERIFY_ME_BUTTON_ID;

#[instrument(level = "info", skip(ctx, interaction))]
pub async fn handle_button_click<L: Locator>(
    ctx: poise::FrameworkContext<'_, L, Error>,
    interaction: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    info!(
        user_id = interaction.user.id.get(),
        "User clicked on the verify button",
    );

    let mut authentication_port = ctx.user_data.create_authentication_port();
    interaction
        .defer_ephemeral(ctx.serenity_context.http())
        .await?;

    let response = match authentication_port
        .create_authentication_link(UserId(interaction.user.id.get()))
        .await
    {
        Ok(link) => {
            let msg = response::authentication_link::authentication_link(link, &interaction.user);
            let msg = interaction
                .user
                .direct_message(ctx.serenity_context, msg)
                .await?;
            response::authentication_link::tell_user_direct_message_sent(
                &interaction.user,
                &msg.link(),
            )
        }
        Err(AuthenticationError::TemporaryUnavailable) => {
            response::unavailable::temporary_unavailable()
        }
        Err(AuthenticationError::AuthenticationRequestAlreadyConfirmed)
        | Err(AuthenticationError::AuthenticationRequestNotFound) => {
            error!(
                "Unreachable: Got authentication request not found error when creating an authentication request",
            );
            response::unavailable::temporary_unavailable()
        }
    };

    interaction
        .edit_response(
            ctx.serenity_context,
            response.to_slash_initial_response_edit(serenity::EditInteractionResponse::new()),
        )
        .await?;

    Ok(())
}
