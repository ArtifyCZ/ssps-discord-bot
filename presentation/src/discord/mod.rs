use crate::application_ports::Locator;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ClientBuilder, GuildId};

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
