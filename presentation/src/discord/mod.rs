use crate::application_ports::Locator;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ClientBuilder, ComponentInteractionDataKind, GuildId, Interaction};

mod buttons;
pub mod commands;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a, D> = poise::Context<'a, D, Error>;

pub async fn run_bot<L: Locator + Send + Sync + 'static>(
    locator: L,
    token: String,
    intents: serenity::GatewayIntents,
    guild: GuildId,
) -> Result<(), Error> {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::enabled_commands(),
            event_handler: |ctx, event, framework, locator| {
                Box::pin(event_handler(ctx, event, framework, locator))
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(ctx, &framework.options().commands, guild)
                    .await?;
                Ok(locator)
            })
        })
        .build();

    let client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await?;

    Ok(())
}

async fn event_handler<L: Locator>(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    framework: poise::FrameworkContext<'_, L, Error>,
    locator: &L,
) -> Result<(), Error> {
    if let serenity::FullEvent::InteractionCreate {
        interaction: Interaction::Component(component_interaction),
    } = event
    {
        if let ComponentInteractionDataKind::Button = component_interaction.data.kind {
            buttons::handle_button_click(ctx, component_interaction, framework, locator).await?
        }
    }

    Ok(())
}
