mod locator;

use anyhow::anyhow;
use poise::serenity_prelude as serenity;
use presentation::discord::run_bot;
use serenity::all::{ClientBuilder, GuildId};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let token = env::var("DISCORD_BOT_TOKEN")?;
    let guild = env::var("DISCORD_GUILD_ID")?
        .parse::<u64>()
        .map(|id| GuildId::new(id))?;
    let intents = serenity::GatewayIntents::non_privileged();

    let client = ClientBuilder::new(&token, intents).await?.http;

    let locator = locator::ApplicationPortLocator::new(client);

    let bot = tokio::spawn(run_bot(locator, token, intents, guild));

    bot.await?.map_err(|e| anyhow!(e))?;

    Ok(())
}
