use application_ports::authentication::AuthenticationPort;
use application_ports::information_channel::InformationChannelPort;
use application_ports::periodic_scheduling_handler::PeriodicSchedulingHandlerPort;
use application_ports::role_sync_job_handler::RoleSyncJobHandlerPort;
use application_ports::user::UserPort;
use application_ports::user_info_sync_job_handler::UserInfoSyncJobHandlerPort;
use poise::serenity_prelude as serenity;

pub trait Locator {
    fn create_authentication_port(&self) -> impl AuthenticationPort + Send + Sync;
    fn create_periodic_scheduling_handler_port(
        &self,
    ) -> impl PeriodicSchedulingHandlerPort + Send + Sync;
    fn create_role_sync_job_handler_port(&self) -> impl RoleSyncJobHandlerPort + Send + Sync;
    fn create_information_channel_port(&self) -> impl InformationChannelPort + Send + Sync;
    fn create_user_port(&self) -> impl UserPort + Send + Sync;

    fn get_role_sync_job_handler_port(&self) -> &(dyn RoleSyncJobHandlerPort + Send + Sync);
    fn get_user_info_sync_job_handler_port(
        &self,
    ) -> &(dyn UserInfoSyncJobHandlerPort + Send + Sync);

    fn get_discord_client(&self) -> &serenity::http::Http;
}
