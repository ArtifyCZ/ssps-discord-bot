use application_ports::discord::ChannelId;
use application_ports::information_channel::{InformationChannelError, InformationChannelPort};
use domain::ports::discord::DiscordPort;
use tracing::{info, instrument};

pub struct InformationChannelService<TDiscordPort> {
    discord_port: TDiscordPort,
}

impl<TDiscordPort> InformationChannelService<TDiscordPort>
where
    TDiscordPort: DiscordPort + Sync + Send,
{
    #[instrument(level = "debug", skip_all)]
    pub fn new(discord_port: TDiscordPort) -> Self {
        Self { discord_port }
    }
}

impl<TDiscordPort> InformationChannelPort for InformationChannelService<TDiscordPort>
where
    TDiscordPort: DiscordPort + Sync + Send,
{
    #[instrument(level = "info", skip(self))]
    async fn update_information(
        &self,
        channel_id: ChannelId,
    ) -> Result<(), InformationChannelError> {
        self.discord_port.purge_messages(channel_id).await?;

        let messages = domain::information_channel::create_messages();

        for message in messages {
            self.discord_port.send_message(channel_id, message).await?;
        }

        info!(channel_id = channel_id.0, "Information channel updated");

        Ok(())
    }
}
