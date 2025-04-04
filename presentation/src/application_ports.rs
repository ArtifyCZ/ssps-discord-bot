use application_ports::authentication::AuthenticationPort;
use application_ports::information_channel::InformationChannelPort;
use std::sync::Arc;

pub trait Locator {
    fn get_authentication_port(&self) -> Arc<dyn AuthenticationPort + Send + Sync>;
    fn get_information_channel_port(&self) -> Arc<dyn InformationChannelPort + Send + Sync>;
}
