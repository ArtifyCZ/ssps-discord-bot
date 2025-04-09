use crate::application_ports::Locator;
use crate::discord::Error;
use application_ports::authentication::AuthenticationError;
use domain_shared::discord::UserId;
use poise::serenity_prelude::{
    ButtonStyle, CreateActionRow, CreateButton, CreateInteractionResponse, Mentionable,
};
use poise::{serenity_prelude as serenity, CreateReply};
use tracing::{info, instrument, warn};

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
        "User clicked on the verify button.",
    );

    let authentication_port = locator.get_authentication_port();

    let authentication_link = match authentication_port
        .create_authentication_link(UserId(interaction.user.id.get()))
        .await
    {
        Ok(link) => link,
        Err(AuthenticationError::AlreadyAuthenticated) => {
            warn!(
                user_id = interaction.user.id.get(),
                "User tried to create new authentication link, but is already authenticated."
            );

            let response =
                "Již jsi byl ověřen. Není možné vytvořit další ověřovací odkaz.".to_string();

            let response = CreateReply::default()
                .content(response)
                .ephemeral(true)
                .reply(true);
            let response = CreateInteractionResponse::Message(
                response
                    .to_slash_initial_response(serenity::CreateInteractionResponseMessage::new()),
            );
            interaction.create_response(ctx, response).await?;

            return Ok(());
        }
        Err(AuthenticationError::Error(error)) => return Err(error),
        Err(AuthenticationError::AuthenticationRequestNotFound) => unreachable!(),
    };

    let response = format!(
        "Ahoj, {}! Ověř svůj účet kliknutím na tlačítko.",
        interaction.user.mention(),
    );

    let button = CreateButton::new_link(authentication_link.0)
        .style(ButtonStyle::Primary)
        .label("Ověřit se");

    let components = vec![CreateActionRow::Buttons(vec![button])];

    let response = CreateReply::default()
        .content(response)
        .components(components)
        .ephemeral(true)
        .reply(true);
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
