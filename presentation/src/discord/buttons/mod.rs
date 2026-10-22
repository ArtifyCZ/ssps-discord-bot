use crate::application_ports::Locator;
use crate::discord::Error;
use poise::serenity_prelude as serenity;
use tracing::instrument;

pub mod verify;

#[instrument(level = "trace", skip(ctx, interaction))]
pub async fn handle_button_click<L: Locator>(
    ctx: poise::FrameworkContext<'_, L, Error>,
    interaction: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    match interaction.data.custom_id.as_str() {
        verify::BUTTON_ID => verify::handle_button_click(ctx, interaction).await,
        _ => Ok(()),
    }
}
