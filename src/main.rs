use serenity::all::{ClientBuilder, GuildId};
use std::env;

use poise::serenity_prelude as serenity;

struct Data {}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

mod commands;
mod resources;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let token = env::var("DISCORD_BOT_TOKEN")?;
    let guild = env::var("DISCORD_GUILD_ID")?
        .parse::<u64>()
        .map(|id| GuildId::new(id))?;
    let intents = serenity::GatewayIntents::non_privileged();

    let bot = tokio::spawn(run_bot(token, intents, guild));

    bot.await??;

    Ok(())
}

async fn run_bot(
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
