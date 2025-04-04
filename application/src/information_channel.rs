use application_ports::discord::ChannelId;
use application_ports::information_channel::{InformationChannelError, InformationChannelPort};
use async_trait::async_trait;
use domain::ports::discord::DiscordPort;
use std::sync::Arc;

pub struct InformationChannelService {
    discord_port: Arc<dyn DiscordPort + Sync + Send>,
}

impl InformationChannelService {
    pub fn new(discord_port: Arc<dyn DiscordPort + Sync + Send>) -> Self {
        Self { discord_port }
    }
}

#[async_trait]
impl InformationChannelPort for InformationChannelService {
    async fn update_information(
        &self,
        channel_id: ChannelId,
    ) -> Result<(), InformationChannelError> {
        self.discord_port.purge_messages(channel_id).await?;

        let messages = domain::information_channel::create_messages();

        for message in messages {
            self.discord_port.send_message(channel_id, message).await?;
        }

        Ok(())
    }
}
