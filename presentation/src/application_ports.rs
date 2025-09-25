use application_ports::authentication::AuthenticationPort;
use application_ports::information_channel::InformationChannelPort;
use application_ports::role_sync_job_handler::RoleSyncJobHandlerPort;
use application_ports::user::UserPort;
use application_ports::user_info_sync_job_handler::UserInfoSyncJobHandlerPort;
use poise::serenity_prelude as serenity;

pub trait Locator {
    fn get_authentication_port(&self) -> &(dyn AuthenticationPort + Send + Sync);
    fn get_information_channel_port(&self) -> &(dyn InformationChannelPort + Send + Sync);
    fn get_user_port(&self) -> &(dyn UserPort + Send + Sync);
    fn get_role_sync_job_handler_port(&self) -> &(dyn RoleSyncJobHandlerPort + Send + Sync);
    fn get_user_info_sync_job_handler_port(
        &self,
    ) -> &(dyn UserInfoSyncJobHandlerPort + Send + Sync);

    fn get_discord_client(&self) -> &serenity::http::Http;
}
