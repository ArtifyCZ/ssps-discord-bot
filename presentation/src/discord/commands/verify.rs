use crate::application_ports::Locator;
use crate::discord::{Context, Error};
use application_ports::authentication::AuthenticationError;
use domain_shared::discord::UserId;
use poise::serenity_prelude::{ButtonStyle, CreateActionRow, CreateButton, Mentionable};
use poise::CreateReply;

#[poise::command(
    slash_command,
    rename = "verify",
    required_permissions = "ADMINISTRATOR"
)]
pub async fn command<D: Sync + Locator>(ctx: Context<'_, D>) -> Result<(), Error> {
    let authentication_port = ctx.data().get_authentication_port();

    let user = ctx.author();

    let authentication_link = match authentication_port
        .create_authentication_link(UserId(user.id.get()))
        .await
    {
        Ok(link) => link,
        Err(AuthenticationError::Error(error)) => return Err(error),
        Err(AuthenticationError::AlreadyAuthenticated) => {
            let response = "You have already been verified. It is not possible to create another verification link.".to_string();

            let response = CreateReply::default()
                .content(response)
                .ephemeral(true)
                .reply(true);
            ctx.send(response).await?;

            return Ok(());
        }
    };

    let response = format!(
        "Hello, {}! Please verify your account by clicking the button.",
        user.mention(),
    );

    let button = CreateButton::new_link(authentication_link.0)
        .style(ButtonStyle::Primary)
        .label("Verify");

    let components = vec![CreateActionRow::Buttons(vec![button])];

    let response = CreateReply::default()
        .content(response)
        .components(components)
        .ephemeral(true)
        .reply(true);
    ctx.send(response).await?;

    Ok(())
}
