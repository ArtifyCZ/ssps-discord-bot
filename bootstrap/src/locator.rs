use application::authentication::AuthenticationService;
use application::information_channel::InformationChannelService;
use application_ports::authentication::AuthenticationPort;
use application_ports::information_channel::InformationChannelPort;
use presentation::application_ports::Locator;
use std::sync::Arc;
use tracing::instrument;

#[derive(Clone)]
pub struct ApplicationPortLocator {
    authentication_adapter: Arc<AuthenticationService>,
    information_channel_adapter: Arc<InformationChannelService>,
}

impl ApplicationPortLocator {
    #[instrument(level = "trace", skip_all)]
    pub fn new(
        authentication_adapter: Arc<AuthenticationService>,
        information_channel_adapter: Arc<InformationChannelService>,
    ) -> Self {
        Self {
            authentication_adapter,
            information_channel_adapter,
        }
    }
}

impl Locator for ApplicationPortLocator {
    #[instrument(level = "trace", skip(self))]
    fn get_authentication_port(&self) -> Arc<dyn AuthenticationPort + Send + Sync> {
        self.authentication_adapter.clone()
    }

    #[instrument(level = "trace", skip(self))]
    fn get_information_channel_port(&self) -> Arc<dyn InformationChannelPort + Send + Sync> {
        self.information_channel_adapter.clone()
    }
}
