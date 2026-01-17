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
use domain::authentication::archived_authenticated_user::ArchivedAuthenticatedUserRepository;
use domain::authentication::authenticated_user::AuthenticatedUserRepository;
use domain::authentication::user_authentication_request::UserAuthenticationRequestRepository;
use domain::jobs::role_sync_job::RoleSyncRequestedRepository;
use domain::jobs::user_info_sync_job::UserInfoSyncRequestedRepository;
use domain::ports::discord::DiscordPort;
use domain::ports::oauth::OAuthPort;
use domain_shared::discord::{InviteLink, RoleId};
use infrastructure::authentication::archived_authenticated_user::PostgresArchivedAuthenticatedUserRepository;
use infrastructure::authentication::authenticated_user::PostgresAuthenticatedUserRepository;
use infrastructure::authentication::user_authentication_request::PostgresUserAuthenticationRequestRepository;
use infrastructure::discord::DiscordAdapter;
use infrastructure::jobs::role_sync_job_repository::PostgresRoleSyncRequestedRepository;
use infrastructure::jobs::user_info_sync_job_repository::PostgresUserInfoSyncRequestedRepository;
use infrastructure::oauth::{OAuthAdapter, OAuthAdapterConfig};
use presentation::application_ports::Locator;
use serenity::all::GuildId;
use std::sync::Arc;
use tracing::instrument;

#[derive(Clone)]
pub struct ApplicationPortLocator {
    pub(crate) everyone_roles: Vec<RoleId>,
    pub(crate) additional_student_roles: Vec<RoleId>,
    pub(crate) unknown_class_role_id: RoleId,
    pub(crate) invite_link: InviteLink,
    pub(crate) guild_id: GuildId,
    pub(crate) oauth_adapter_config: OAuthAdapterConfig,

    pub(crate) postgres_pool: sqlx::PgPool,
    pub(crate) serenity_client: Arc<serenity::http::Http>,

    pub(crate) role_sync_job_wake_tx: tokio::sync::mpsc::Sender<()>,
    pub(crate) user_info_sync_job_wake_tx: tokio::sync::mpsc::Sender<()>,
}

impl ApplicationPortLocator {
    #[instrument(level = "trace", skip(self))]
    fn authenticated_user_repository(
        &self,
    ) -> impl AuthenticatedUserRepository + Send + Sync + use<'_> {
        PostgresAuthenticatedUserRepository::new(&self.postgres_pool)
    }

    #[instrument(level = "trace", skip(self))]
    fn archived_authenticated_user_repository(
        &self,
    ) -> impl ArchivedAuthenticatedUserRepository + Send + Sync + use<'_> {
        PostgresArchivedAuthenticatedUserRepository::new(&self.postgres_pool)
    }

    #[instrument(level = "trace", skip(self))]
    fn role_sync_requested_repository(&self) -> impl RoleSyncRequestedRepository + Send + Sync {
        PostgresRoleSyncRequestedRepository::new(
            self.postgres_pool.clone(),
            self.role_sync_job_wake_tx.clone(),
        )
    }

    #[instrument(level = "trace", skip(self))]
    fn user_authentication_request_repository(
        &self,
    ) -> impl UserAuthenticationRequestRepository + Send + Sync {
        PostgresUserAuthenticationRequestRepository::new(self.postgres_pool.clone())
    }

    #[instrument(level = "trace", skip(self))]
    fn user_info_sync_requested_repository(
        &self,
    ) -> impl UserInfoSyncRequestedRepository + Send + Sync {
        PostgresUserInfoSyncRequestedRepository::new(
            self.postgres_pool.clone(),
            self.user_info_sync_job_wake_tx.clone(),
        )
    }

    #[instrument(level = "trace", skip(self))]
    fn discord_adapter(&self) -> impl DiscordPort + Send + Sync {
        DiscordAdapter::new(self.serenity_client.clone(), self.guild_id)
    }

    #[instrument(level = "trace", skip(self))]
    fn oauth_adapter(&self) -> impl OAuthPort + Send + Sync {
        OAuthAdapter::new(&self.oauth_adapter_config)
    }
}

impl Locator for ApplicationPortLocator {
    #[instrument(level = "trace", skip(self))]
    fn create_authentication_port(&self) -> impl AuthenticationPort + Send + Sync {
        AuthenticationService {
            oauth_port: self.oauth_adapter(),
            archived_authenticated_user_repository: self.archived_authenticated_user_repository(),
            authenticated_user_repository: self.authenticated_user_repository(),
            user_authentication_request_repository: self.user_authentication_request_repository(),
            user_info_sync_requested_repository: self.user_info_sync_requested_repository(),
            role_sync_requested_repository: self.role_sync_requested_repository(),
            invite_link: self.invite_link.clone(),
        }
    }

    #[instrument(level = "trace", skip(self))]
    fn create_periodic_scheduling_handler_port(&self) -> impl PeriodicSchedulingHandlerPort {
        PeriodicSchedulingHandler::new(
            self.discord_adapter(),
            self.authenticated_user_repository(),
            self.role_sync_requested_repository(),
            self.user_info_sync_requested_repository(),
        )
    }

    #[instrument(level = "trace", skip(self))]
    fn create_role_sync_job_handler_port(&self) -> impl RoleSyncJobHandlerPort + Send + Sync {
        RoleSyncJobHandler::new(
            self.discord_adapter(),
            self.authenticated_user_repository(),
            self.role_sync_requested_repository(),
            self.everyone_roles.clone(),
            self.additional_student_roles.clone(),
            self.unknown_class_role_id,
        )
    }

    #[instrument(level = "trace", skip(self))]
    fn create_user_info_sync_job_handler_port(
        &self,
    ) -> impl UserInfoSyncJobHandlerPort + Send + Sync {
        UserInfoSyncJobHandler::new(
            self.authenticated_user_repository(),
            self.role_sync_requested_repository(),
            self.user_info_sync_requested_repository(),
            self.oauth_adapter(),
        )
    }

    #[instrument(level = "trace", skip(self))]
    fn create_information_channel_port(&self) -> impl InformationChannelPort + Send + Sync {
        InformationChannelService::new(self.discord_adapter())
    }

    #[instrument(level = "trace", skip(self))]
    fn create_user_port(&self) -> impl UserPort + Send + Sync {
        UserService::new(
            self.authenticated_user_repository(),
            self.role_sync_requested_repository(),
            self.user_info_sync_requested_repository(),
        )
    }

    #[instrument(level = "trace", skip(self))]
    fn get_discord_client(&self) -> &serenity::http::Http {
        self.serenity_client.as_ref()
    }
}
