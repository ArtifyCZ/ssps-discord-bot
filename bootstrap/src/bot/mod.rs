use crate::Data;
use poise::serenity_prelude as serenity;
use serenity::all::{ClientBuilder, GuildId};

mod commands;

pub async fn run_bot(
    token: String,
    intents: serenity::GatewayIntents,
    guild: GuildId,
) -> anyhow::Result<()> {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::enabled_commands(),
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(ctx, &framework.options().commands, guild)
                    .await?;
                Ok(Data {})
            })
        })
        .build();

    let client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await?;

    Ok(())
}
