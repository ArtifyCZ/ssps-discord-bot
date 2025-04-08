use crate::application_ports::Locator;
use crate::discord::Error;
use poise::serenity_prelude as serenity;

pub mod verify;

pub async fn handle_button_click<L: Locator>(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    framework: poise::FrameworkContext<'_, L, Error>,
    locator: &L,
) -> Result<(), Error> {
    match interaction.data.custom_id.as_str() {
        verify::BUTTON_ID => {
            verify::handle_button_click(ctx, interaction, framework, locator).await
        }
        _ => Ok(()),
    }
}
