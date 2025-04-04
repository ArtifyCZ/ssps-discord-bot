mod locator;

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use poise::serenity_prelude as serenity;
use presentation::discord::run_bot;
use serenity::all::{ClientBuilder, GuildId};

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(name = "run")]
    Run {
        /// The token for the Discord bot
        #[arg(long, env = "DISCORD_BOT_TOKEN")]
        discord_bot_token: String,
        /// The ID of the Discord guild (server) to run the bot in
        #[arg(long, env = "DISCORD_GUILD_ID")]
        guild: u64,
    },
    #[command(name = "migrate")]
    Migrate,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let Cli {
        database_url,
        command,
    } = Cli::parse();

    match command {
        Command::Run {
            discord_bot_token,
            guild,
        } => run(database_url, discord_bot_token, GuildId::new(guild)).await?,
        Command::Migrate => migrate(database_url).await?,
    }

    Ok(())
}

async fn run(
    database_url: String,
    discord_bot_token: String,
    guild: GuildId,
) -> anyhow::Result<()> {
    let intents = serenity::GatewayIntents::non_privileged();

    let database_connection = sqlx::PgPool::connect(&database_url).await?;
    let serenity_client = ClientBuilder::new(&discord_bot_token, intents).await?.http;

    let locator = locator::ApplicationPortLocator::new(database_connection, serenity_client);

    let bot = tokio::spawn(run_bot(locator, discord_bot_token, intents, guild));

    bot.await?.map_err(|e| anyhow!(e))?;

    Ok(())
}

async fn migrate(database_url: String) -> anyhow::Result<()> {
    let connection = sqlx::PgPool::connect(&database_url).await?;
    infrastructure::database::MIGRATOR.run(&connection).await?;
    Ok(())
}
