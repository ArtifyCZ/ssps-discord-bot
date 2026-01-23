use application_ports::authentication::AuthenticationPort;
use application_ports::information_channel::InformationChannelPort;
use application_ports::periodic_scheduling_handler::PeriodicSchedulingHandlerPort;
use application_ports::role_sync_job_handler::RoleSyncJobHandlerPort;
use application_ports::user::UserPort;
use application_ports::user_info_sync_job_handler::UserInfoSyncJobHandlerPort;
use domain_shared::discord::InviteLink;
use poise::serenity_prelude as serenity;
use std::future::Future;

pub trait Locator {
    fn create_authentication_port(&self) -> impl AuthenticationPort + Send + Sync;
    fn create_periodic_scheduling_handler_port(
        &self,
    ) -> impl PeriodicSchedulingHandlerPort + Send + Sync;
    fn create_role_sync_job_handler_port(&self) -> impl RoleSyncJobHandlerPort + Send + Sync;
    fn create_user_info_sync_job_handler_port(
        &self,
    ) -> impl UserInfoSyncJobHandlerPort + Send + Sync;
    fn create_information_channel_port(&self) -> impl InformationChannelPort + Send + Sync;
    fn create_user_port(&self) -> impl UserPort + Send + Sync;
    fn create_scope(&self) -> impl Future<Output = impl LocatorScope + Send + Sync> + Send;

    fn get_invite_link(&self) -> &InviteLink;
    fn get_discord_client(&self) -> &serenity::http::Http;
}

pub trait LocatorScope {
    fn create_authentication_port(&mut self) -> impl AuthenticationPort + Send + Sync;
}
