use domain_shared::authentication::AuthenticationLink;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ButtonStyle, CreateActionRow, CreateButton, Mentionable};
use poise::CreateReply;
use serenity::CreateMessage;
use tracing::instrument;

#[instrument(level = "debug", skip_all)]
pub fn authentication_link(link: AuthenticationLink, user: &serenity::User) -> CreateMessage {
    let response = format!(
        "Ahoj, {}! Ověř svůj účet kliknutím na tlačítko níže. Řekneme ti, až budeš ověřený.\
        \n\
        Tato žádost o ověření po nějakém čase přestane být platná.",
        user.mention(),
    );

    let button = CreateButton::new_link(link.0)
        .style(ButtonStyle::Primary)
        .label("Ověřit se");

    let components = vec![CreateActionRow::Buttons(vec![button])];

    CreateMessage::default()
        .content(response)
        .components(components)
}

#[instrument(level = "debug", skip_all)]
pub fn tell_user_direct_message_sent(user: &serenity::User, message_link: &str) -> CreateReply {
    let response = format!(
        "Ahoj, {}! K ověření jsem ti poslal zprávu do soukromých zpráv. {}",
        user.mention(),
        message_link,
    );

    CreateReply::default()
        .content(response)
        .reply(true)
        .ephemeral(true)
}
