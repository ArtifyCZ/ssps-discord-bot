use application::information_channel::InformationChannelService;
use application_ports::information_channel::InformationChannelPort;
use infrastructure::discord::DiscordAdapter;
use presentation::application_ports::Locator;
use serenity::all::Http;
use std::sync::Arc;

pub struct ApplicationPortLocator {
    information_channel_adapter: Arc<InformationChannelService>,
}

impl ApplicationPortLocator {
    pub fn new(database_connection: sqlx::PgPool, serenity_client: Arc<Http>) -> Self {
        let _ = database_connection; // Suppress unused variable warning
        let discord_adapter = Arc::new(DiscordAdapter::new(serenity_client.clone()));
        let information_channel_adapter =
            Arc::new(InformationChannelService::new(discord_adapter.clone()));

        Self {
            information_channel_adapter,
        }
    }
}

impl Locator for ApplicationPortLocator {
    fn get_information_channel_port(&self) -> Arc<dyn InformationChannelPort + Send + Sync> {
        self.information_channel_adapter.clone()
    }
}
