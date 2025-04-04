use application_ports::information_channel::InformationChannelPort;
use std::sync::Arc;

pub trait Locator {
    fn get_information_channel_port(&self) -> Arc<dyn InformationChannelPort + Send + Sync>;
}
