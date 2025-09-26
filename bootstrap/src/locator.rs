use application::authentication::AuthenticationService;
use application::information_channel::InformationChannelService;
use application::periodic_scheduling_handler::PeriodicSchedulingHandler;
use application::role_sync_job_handler::RoleSyncJobHandler;
use application::user::UserService;
use application::user_info_sync_job_handler::UserInfoSyncJobHandler;
use application_ports::authentication::AuthenticationPort;
use application_ports::information_channel::InformationChannelPort;
use application_ports::periodic_scheduling_handler::PeriodicSchedulingHandlerPort;
use application_ports::role_sync_job_handler::RoleSyncJobHandlerPort;
use application_ports::user::UserPort;
use application_ports::user_info_sync_job_handler::UserInfoSyncJobHandlerPort;
use infrastructure::authentication::authenticated_user::PostgresAuthenticatedUserRepository;
use infrastructure::discord::DiscordAdapter;
use infrastructure::jobs::role_sync_job_repository::PostgresRoleSyncRequestedRepository;
use infrastructure::jobs::user_info_sync_job_repository::PostgresUserInfoSyncRequestedRepository;
use presentation::application_ports::Locator;
use std::sync::Arc;
use tracing::instrument;

#[derive(Clone)]
pub struct ApplicationPortLocator {
    pub(crate) discord_adapter: Arc<DiscordAdapter>,
    pub(crate) authenticated_user_repository: Arc<PostgresAuthenticatedUserRepository>,
    pub(crate) role_sync_requested_repository: Arc<PostgresRoleSyncRequestedRepository>,
    pub(crate) user_info_sync_requested_repository: Arc<PostgresUserInfoSyncRequestedRepository>,

    pub(crate) authentication_adapter: Arc<AuthenticationService>,
    pub(crate) information_channel_adapter: Arc<InformationChannelService>,
    pub(crate) user_adapter: Arc<UserService>,
    pub(crate) role_sync_job_handler_adapter: Arc<RoleSyncJobHandler>,
    pub(crate) user_info_sync_job_handler_adapter: Arc<UserInfoSyncJobHandler>,
    pub(crate) serenity_client: Arc<serenity::http::Http>,
}

impl Locator for ApplicationPortLocator {
    #[instrument(level = "trace", skip(self))]
    fn create_periodic_scheduling_handler_port(&self) -> impl PeriodicSchedulingHandlerPort {
        PeriodicSchedulingHandler::new(
            self.discord_adapter.clone(),
            self.authenticated_user_repository.clone(),
            self.role_sync_requested_repository.clone(),
            self.user_info_sync_requested_repository.clone(),
        )
    }

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

    #[instrument(level = "trace", skip(self))]
    fn get_role_sync_job_handler_port(&self) -> &(dyn RoleSyncJobHandlerPort + Send + Sync) {
        &*self.role_sync_job_handler_adapter
    }

    #[instrument(level = "trace", skip(self))]
    fn get_user_info_sync_job_handler_port(
        &self,
    ) -> &(dyn UserInfoSyncJobHandlerPort + Send + Sync) {
        &*self.user_info_sync_job_handler_adapter
    }

    #[instrument(level = "trace", skip(self))]
    fn get_discord_client(&self) -> &serenity::http::Http {
        self.serenity_client.as_ref()
    }
}
