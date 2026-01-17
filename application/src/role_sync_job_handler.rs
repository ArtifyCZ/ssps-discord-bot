use application_ports::role_sync_job_handler::{RoleSyncJobHandlerError, RoleSyncJobHandlerPort};
use async_trait::async_trait;
use chrono::{Duration, TimeDelta};
use domain::authentication::authenticated_user::{
    AuthenticatedUserRepository, AuthenticatedUserRepositoryError,
};
use domain::authentication::create_class_ids;
use domain::jobs::role_sync_job::{
    RoleSyncRequested, RoleSyncRequestedRepository, RoleSyncRequestedRepositoryError,
};
use domain::ports::discord::{DiscordError, DiscordPort};
use domain::roles::RolesDiffService;
use domain_shared::discord::RoleId;
use tracing::{error, info, instrument};

pub struct RoleSyncJobHandler<
    TDiscordPort,
    TAuthenticatedUserRepository,
    TRoleSyncRequestedRepository,
> {
    discord_port: TDiscordPort,
    authenticated_user_repository: TAuthenticatedUserRepository,
    role_sync_requested_repository: TRoleSyncRequestedRepository,
    everyone_roles: Vec<RoleId>,
    additional_student_roles: Vec<RoleId>,
    class_ids: Vec<String>,
    roles_diff_service: Option<RolesDiffService>,
    unknown_class_role_id: RoleId,
}

impl<TDiscordPort, TAuthenticatedUserRepository, TRoleSyncRequestedRepository>
    RoleSyncJobHandler<TDiscordPort, TAuthenticatedUserRepository, TRoleSyncRequestedRepository>
where
    TDiscordPort: DiscordPort + Send + Sync,
    TAuthenticatedUserRepository: AuthenticatedUserRepository + Send + Sync,
    TRoleSyncRequestedRepository: RoleSyncRequestedRepository + Send + Sync,
{
    #[instrument(level = "trace", skip_all)]
    pub fn new(
        discord_port: TDiscordPort,
        authenticated_user_repository: TAuthenticatedUserRepository,
        role_sync_requested_repository: TRoleSyncRequestedRepository,
        everyone_roles: Vec<RoleId>,
        additional_student_roles: Vec<RoleId>,
        unknown_class_role_id: RoleId,
    ) -> Self {
        let class_ids = create_class_ids();

        Self {
            discord_port,
            authenticated_user_repository,
            role_sync_requested_repository,
            everyone_roles,
            additional_student_roles,
            class_ids,
            roles_diff_service: None,
            unknown_class_role_id,
        }
    }

    #[instrument(level = "info", skip(self))]
    async fn handle(&mut self, request: RoleSyncRequested) -> Result<(), RoleSyncJobHandlerError> {
        const MIN_DURATION_SINCE_QUEUED: TimeDelta = Duration::milliseconds(400);
        const WAIT_TICK_DURATION: TimeDelta = Duration::milliseconds(100);
        let can_sync_since = request.queued_at + MIN_DURATION_SINCE_QUEUED;
        loop {
            if can_sync_since <= chrono::Utc::now() {
                break;
            }

            tokio::time::sleep(WAIT_TICK_DURATION.to_std().unwrap()).await;
        }

        if self.roles_diff_service.is_none() {
            let roles_diff_service = self.create_roles_diff_service().await?;
            self.roles_diff_service = Some(roles_diff_service);
        }
        let roles_diff_service = self.roles_diff_service.as_ref().ok_or_else(|| {
            error!("Unreachable: roles_diff_service is None, but it should be Some at this point");
            RoleSyncJobHandlerError::TemporaryUnavailable
        })?;

        let (assigned_roles, user) = tokio::try_join!(
            async {
                self.discord_port
                    .find_user_roles(request.user_id)
                    .await
                    .map_err(map_discord_err)
            },
            async {
                self.authenticated_user_repository
                    .find_by_user_id(request.user_id)
                    .await
                    .map_err(map_user_repo_err)
            }
        )?;
        let assigned_roles = match assigned_roles {
            None => return Ok(()), // User not found on the guild, nothing to do
            Some(assigned_roles) => assigned_roles,
        };

        let assigned_roles = assigned_roles.iter().map(|r| r.role_id).collect::<Vec<_>>();

        let mut role_diff = roles_diff_service.diff_roles(user.as_ref());
        role_diff.optimize_by_already_assigned_roles(&assigned_roles);

        self.discord_port
            .apply_role_diff(request.user_id, &role_diff, "Role sync job handler")
            .await
            .map_err(map_discord_err)?;

        info!(
            "Successfully synced roles for user {:?} with diff {:?}",
            request.user_id, role_diff,
        );

        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    async fn create_roles_diff_service(&self) -> Result<RolesDiffService, RoleSyncJobHandlerError> {
        let mut class_id_to_role_id = Vec::new();

        for class_id in &self.class_ids {
            let role = self
                .discord_port
                .find_or_create_role_by_name(&class_id.to_uppercase(), "Role for students of class")
                .await
                .map_err(map_discord_err)?;
            class_id_to_role_id.push((class_id.to_string(), role.role_id));
        }

        Ok(RolesDiffService {
            everyone_roles: self.everyone_roles.clone(),
            additional_student_roles: self.additional_student_roles.clone(),
            unknown_class_role_id: self.unknown_class_role_id,
            class_ids: self.class_ids.clone(),
            class_id_to_role_id,
        })
    }
}

#[async_trait]
impl<TDiscordPort, TAuthenticatedUserRepository, TRoleSyncRequestedRepository>
    RoleSyncJobHandlerPort
    for RoleSyncJobHandler<TDiscordPort, TAuthenticatedUserRepository, TRoleSyncRequestedRepository>
where
    TDiscordPort: DiscordPort + Send + Sync,
    TAuthenticatedUserRepository: AuthenticatedUserRepository + Send + Sync,
    TRoleSyncRequestedRepository: RoleSyncRequestedRepository + Send + Sync,
{
    #[instrument(level = "debug", skip_all)]
    async fn tick(&mut self) -> Result<(), RoleSyncJobHandlerError> {
        let high_priority = self
            .role_sync_requested_repository
            .pop_oldest(false)
            .await
            .map_err(map_sync_req_repo_err)?;

        if let Some(request) = high_priority {
            self.handle(request).await?;
            return Ok(());
        }

        let low_priority = self
            .role_sync_requested_repository
            .pop_oldest(true)
            .await
            .map_err(map_sync_req_repo_err)?;

        if let Some(request) = low_priority {
            self.handle(request).await?;
            return Ok(());
        }

        Err(RoleSyncJobHandlerError::NoRequestToHandle)
    }
}

#[instrument(level = "trace", skip_all)]
fn map_discord_err(err: DiscordError) -> RoleSyncJobHandlerError {
    match err {
        DiscordError::DiscordUnavailable => {
            error!("DiscordError::DiscordUnavailable");
            RoleSyncJobHandlerError::TemporaryUnavailable
        }
    }
}

#[instrument(level = "trace", skip_all)]
fn map_user_repo_err(err: AuthenticatedUserRepositoryError) -> RoleSyncJobHandlerError {
    match err {
        AuthenticatedUserRepositoryError::ServiceUnavailable => {
            error!("AuthenticatedUserRepositoryError::ServiceUnavailable");
            RoleSyncJobHandlerError::TemporaryUnavailable
        }
    }
}

#[instrument(level = "trace", skip_all)]
fn map_sync_req_repo_err(err: RoleSyncRequestedRepositoryError) -> RoleSyncJobHandlerError {
    match err {
        RoleSyncRequestedRepositoryError::ServiceUnavailable => {
            error!("RoleSyncRequestedRepositoryError::ServiceUnavailable");
            RoleSyncJobHandlerError::TemporaryUnavailable
        }
    }
}
