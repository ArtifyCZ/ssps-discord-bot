use domain_shared::authentication::AuthenticationLink;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ButtonStyle, CreateActionRow, CreateButton, Mentionable};
use poise::CreateReply;
use tracing::instrument;

#[instrument(level = "debug", skip_all)]
pub fn authentication_link(link: AuthenticationLink, user: &serenity::User) -> CreateReply {
    let response = format!(
        "Ahoj, {}! Ověř svůj účet kliknutím na tlačítko.",
        user.mention(),
    );

    let button = CreateButton::new_link(link.0)
        .style(ButtonStyle::Primary)
        .label("Ověřit se");

    let components = vec![CreateActionRow::Buttons(vec![button])];

    CreateReply::default()
        .content(response)
        .components(components)
        .ephemeral(true)
        .reply(true)
}
