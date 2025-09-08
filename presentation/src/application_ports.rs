use application_ports::authentication::AuthenticationPort;
use application_ports::information_channel::InformationChannelPort;
use application_ports::user::UserPort;

pub trait Locator {
    fn get_authentication_port(&self) -> &(dyn AuthenticationPort + Send + Sync);
    fn get_information_channel_port(&self) -> &(dyn InformationChannelPort + Send + Sync);
    fn get_user_port(&self) -> &(dyn UserPort + Send + Sync);
}
