use std::env;

use async_trait::async_trait;
use serenity::all::{
    Context, CreateInteractionResponse, CreateInteractionResponseMessage, GuildId, Interaction,
    Ready,
};
use serenity::prelude::GatewayIntents;
use serenity::Client;

mod commands;

struct EventHandler;

#[async_trait]
impl serenity::prelude::EventHandler for EventHandler {
    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        let guild_id = env::var("DISCORD_GUILD_ID").unwrap();
        let guild_id = GuildId::new(guild_id.parse().unwrap());
        guild_id
            .set_commands(
                &ctx.http,
                vec![commands::modal::register()],
            )
            .await
            .unwrap();
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let content = match command.data.name.as_str() {
                "modal" => {
                    commands::modal::run(&ctx, &command).await.unwrap();
                    None
                }
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let token = env::var("DISCORD_BOT_TOKEN")?;
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(EventHandler)
        .await?;
    client.start().await?;
    Ok(())
}
