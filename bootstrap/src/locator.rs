use application::authentication::AuthenticationService;
use application::information_channel::InformationChannelService;
use application::user::UserService;
use application_ports::authentication::AuthenticationPort;
use application_ports::information_channel::InformationChannelPort;
use application_ports::user::UserPort;
use presentation::application_ports::Locator;
use std::sync::Arc;
use tracing::instrument;

#[derive(Clone)]
pub struct ApplicationPortLocator {
    authentication_adapter: Arc<AuthenticationService>,
    information_channel_adapter: Arc<InformationChannelService>,
    user_adapter: Arc<UserService>,
}

impl ApplicationPortLocator {
    #[instrument(level = "trace", skip_all)]
    pub fn new(
        authentication_adapter: Arc<AuthenticationService>,
        information_channel_adapter: Arc<InformationChannelService>,
        user_adapter: Arc<UserService>,
    ) -> Self {
        Self {
            authentication_adapter,
            information_channel_adapter,
            user_adapter,
        }
    }
}

impl Locator for ApplicationPortLocator {
    #[instrument(level = "trace", skip(self))]
    fn get_authentication_port(&self) -> &(dyn AuthenticationPort + Send + Sync) {
        &*self.authentication_adapter
    }

    #[instrument(level = "trace", skip(self))]
    fn get_information_channel_port(&self) -> &(dyn InformationChannelPort + Send + Sync) {
        &*self.information_channel_adapter
    }

    #[instrument(level = "trace", skip(self))]
    fn get_user_port(&self) -> &(dyn UserPort + Send + Sync) {
        &*self.user_adapter
    }
}
