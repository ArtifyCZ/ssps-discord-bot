use crate::application_ports::Locator;
use crate::discord::{response, Error};
use application_ports::authentication::AuthenticationError;
use domain_shared::discord::UserId;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{CacheHttp, CreateInteractionResponse};
use tracing::{error, info, instrument};

pub const BUTTON_ID: &str = domain::information_channel::VERIFY_ME_BUTTON_ID;

#[instrument(level = "info", skip(ctx, interaction, _framework, locator))]
pub async fn handle_button_click<L: Locator>(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    _framework: poise::FrameworkContext<'_, L, Error>,
    locator: &L,
) -> Result<(), Error> {
    info!(
        user_id = interaction.user.id.get(),
        "User clicked on the verify button",
    );

    let authentication_port = locator.get_authentication_port();
    interaction.defer(ctx.http()).await?;

    let response = match authentication_port
        .create_authentication_link(UserId(interaction.user.id.get()))
        .await
    {
        Ok(link) => response::authentication_link(link, &interaction.user),
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

    interaction
        .create_response(
            ctx,
            CreateInteractionResponse::Message(
                response
                    .to_slash_initial_response(serenity::CreateInteractionResponseMessage::new()),
            ),
        )
        .await?;

    Ok(())
}
