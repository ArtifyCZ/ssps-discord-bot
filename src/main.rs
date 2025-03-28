use serenity::all::GuildId;
use std::env;

use crate::bot::run_bot;
use poise::serenity_prelude as serenity;

struct Data {}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

mod bot;
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
