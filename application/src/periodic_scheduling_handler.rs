use application_ports::periodic_scheduling_handler::{
    PeriodicSchedulingHandlerError, PeriodicSchedulingHandlerPort,
};
use async_trait::async_trait;
use domain::authentication::authenticated_user::{
    AuthenticatedUserRepository, AuthenticatedUserRepositoryError,
};
use domain::jobs::role_sync_job::{
    request_periodic_role_sync, RoleSyncRequestedRepository, RoleSyncRequestedRepositoryError,
};
use domain::jobs::user_info_sync_job::{
    request_periodic_user_info_sync, UserInfoSyncRequestedRepository,
    UserInfoSyncRequestedRepositoryError,
};
use domain::ports::discord::{DiscordError, DiscordPort};
use domain_shared::discord::UserId;
use std::sync::Arc;
use std::vec::IntoIter;
use tracing::{error, info, instrument};

pub struct PeriodicSchedulingHandler<TDiscordPort, TAuthenticatedUserRepository> {
    discord_port: TDiscordPort,
    authenticated_user_repository: TAuthenticatedUserRepository,
    role_sync_requested_repository: Arc<dyn RoleSyncRequestedRepository + Send + Sync>,
    user_info_sync_requested_repository: Arc<dyn UserInfoSyncRequestedRepository + Send + Sync>,
    authenticated_user_ids: IntoIter<UserId>,
    discord_members_chunk: IntoIter<UserId>,
    discord_members_chunk_offset: Option<UserId>,
}

impl<TDiscordPort, TAuthenticatedUserRepository>
    PeriodicSchedulingHandler<TDiscordPort, TAuthenticatedUserRepository>
where
    TDiscordPort: DiscordPort + Send + Sync,
    TAuthenticatedUserRepository: AuthenticatedUserRepository + Send + Sync,
{
    #[instrument(level = "trace", skip_all)]
    pub fn new(
        discord_port: TDiscordPort,
        authenticated_user_repository: TAuthenticatedUserRepository,
        role_sync_requested_repository: Arc<dyn RoleSyncRequestedRepository + Send + Sync>,
        user_info_sync_requested_repository: Arc<dyn UserInfoSyncRequestedRepository + Send + Sync>,
    ) -> Self {
        Self {
            discord_port,
            authenticated_user_repository,
            role_sync_requested_repository,
            user_info_sync_requested_repository,
            authenticated_user_ids: Vec::new().into_iter(),
            discord_members_chunk: Vec::new().into_iter(),
            discord_members_chunk_offset: None,
        }
    }

    #[instrument(level = "debug", skip_all)]
    async fn produce_user_jobs(
        &self,
        user_id: UserId,
    ) -> Result<(), PeriodicSchedulingHandlerError> {
        tokio::try_join!(
            async {
                let request = request_periodic_user_info_sync(user_id);
                self.user_info_sync_requested_repository
                    .save(&request)
                    .await
                    .map_err(map_user_info_sync_job_repo_err)?;
                Ok(())
            },
            async {
                let request = request_periodic_role_sync(user_id);
                self.role_sync_requested_repository
                    .save(&request)
                    .await
                    .map_err(map_role_sync_job_repo_err)?;
                Ok(())
            }
        )?;

        info!(
            "Successfully produced periodic job requests for user {:?}",
            user_id,
        );

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    async fn retrieve_authenticated_users(
        &self,
    ) -> Result<Vec<UserId>, PeriodicSchedulingHandlerError> {
        let users = self
            .authenticated_user_repository
            .find_all()
            .await
            .map_err(map_user_repo_err)?;

        let user_ids: Vec<UserId> = users.into_iter().map(|u| u.user_id()).collect();

        let user_ids_sample = &user_ids[0..user_ids.len().min(6)];
        info!(
            "Successfully retrieved the list of authenticated users for periodic scheduling: sample(0..6): {:?}",
            user_ids_sample,
        );

        Ok(user_ids)
    }

    #[instrument(level = "debug", skip_all)]
    async fn retrieve_discord_members(
        &self,
    ) -> Result<(Vec<UserId>, Option<UserId>), PeriodicSchedulingHandlerError> {
        let chunk = self
            .discord_port
            .find_all_members(self.discord_members_chunk_offset)
            .await
            .map_err(map_discord_err)?;
        if let Some(chunk) = chunk {
            let offset = chunk.last().copied();

            let chunk_sample = &chunk[0..chunk.len().min(6)];
            info!("Successfully retrieved the list of discord members for periodic scheduling with new offset {:?}: sample(0..6): {:?}", offset, chunk_sample);

            Ok((chunk, offset))
        } else {
            Ok((Vec::new(), None))
        }
    }
}

#[async_trait]
impl<TDiscordPort, TAuthenticatedUserRepository> PeriodicSchedulingHandlerPort
    for PeriodicSchedulingHandler<TDiscordPort, TAuthenticatedUserRepository>
where
    TDiscordPort: DiscordPort + Send + Sync,
    TAuthenticatedUserRepository: AuthenticatedUserRepository + Send + Sync,
{
    #[instrument(level = "debug", skip_all)]
    async fn tick(&mut self) -> Result<(), PeriodicSchedulingHandlerError> {
        if let Some(ref user_id) = self.authenticated_user_ids.next() {
            self.produce_user_jobs(*user_id).await?;
        } else {
            self.authenticated_user_ids = self.retrieve_authenticated_users().await?.into_iter();
        }

        if let Some(ref user_id) = self.discord_members_chunk.next() {
            self.produce_user_jobs(*user_id).await?;
        } else {
            let (members_chunk, offset) = self.retrieve_discord_members().await?;
            self.discord_members_chunk = members_chunk.into_iter();
            self.discord_members_chunk_offset = offset;
        }

        Ok(())
    }
}

#[instrument(level = "debug", skip_all)]
fn map_discord_err(err: DiscordError) -> PeriodicSchedulingHandlerError {
    match err {
        DiscordError::DiscordUnavailable => {
            error!(
                "Periodic scheduling handler is temporarily unavailable: discord is unavailable"
            );
            PeriodicSchedulingHandlerError::TemporarilyUnavailable
        }
    }
}

#[instrument(level = "debug", skip_all)]
fn map_user_repo_err(err: AuthenticatedUserRepositoryError) -> PeriodicSchedulingHandlerError {
    match err {
        AuthenticatedUserRepositoryError::ServiceUnavailable => {
            error!("Periodic scheduling handler is temporarily unavailable: authenticated user repository is unavailable");
            PeriodicSchedulingHandlerError::TemporarilyUnavailable
        }
    }
}

#[instrument(level = "debug", skip_all)]
fn map_user_info_sync_job_repo_err(
    err: UserInfoSyncRequestedRepositoryError,
) -> PeriodicSchedulingHandlerError {
    match err {
        UserInfoSyncRequestedRepositoryError::ServiceUnavailable => {
            error!("Periodic scheduling handler is temporarily unavailable: user info sync job repository is unavailable");
            PeriodicSchedulingHandlerError::TemporarilyUnavailable
        }
    }
}

#[instrument(level = "debug", skip_all)]
fn map_role_sync_job_repo_err(
    err: RoleSyncRequestedRepositoryError,
) -> PeriodicSchedulingHandlerError {
    match err {
        RoleSyncRequestedRepositoryError::ServiceUnavailable => {
            error!("Periodic scheduling handler is temporarily unavailable: role sync job repository is unavailable");
            PeriodicSchedulingHandlerError::TemporarilyUnavailable
        }
    }
}
