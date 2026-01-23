use application_ports::user::{AuthenticatedUserInfoDto, UserError, UserPort};
use chrono::Duration;
use domain::authentication::authenticated_user::{
    AuthenticatedUserRepository, AuthenticatedUserRepositoryError,
};
use domain::jobs::role_sync_job::{
    request_role_sync, RoleSyncRequestedRepository, RoleSyncRequestedRepositoryError,
};
use domain::jobs::user_info_sync_job::{
    request_user_info_sync, UserInfoSyncRequestedRepository, UserInfoSyncRequestedRepositoryError,
};
use domain_shared::discord::UserId;
use tracing::{info, instrument, warn};

pub struct UserService<
    TAuthenticatedUserRepository,
    TRoleSyncRequestedRepository,
    TUserInfoSyncRequestedRepository,
> {
    authenticated_user_repository: TAuthenticatedUserRepository,
    role_sync_requested_repository: TRoleSyncRequestedRepository,
    user_info_sync_requested_repository: TUserInfoSyncRequestedRepository,
}

impl<
        TAuthenticatedUserRepository,
        TRoleSyncRequestedRepository,
        TUserInfoSyncRequestedRepository,
    >
    UserService<
        TAuthenticatedUserRepository,
        TRoleSyncRequestedRepository,
        TUserInfoSyncRequestedRepository,
    >
where
    TAuthenticatedUserRepository: AuthenticatedUserRepository + Send + Sync,
    TRoleSyncRequestedRepository: RoleSyncRequestedRepository + Send + Sync,
    TUserInfoSyncRequestedRepository: UserInfoSyncRequestedRepository + Send + Sync,
{
    #[instrument(level = "trace", skip_all)]
    pub fn new(
        authenticated_user_repository: TAuthenticatedUserRepository,
        role_sync_requested_repository: TRoleSyncRequestedRepository,
        user_info_sync_requested_repository: TUserInfoSyncRequestedRepository,
    ) -> Self {
        Self {
            authenticated_user_repository,
            role_sync_requested_repository,
            user_info_sync_requested_repository,
        }
    }
}

impl<
        TAuthenticatedUserRepository,
        TRoleSyncRequestedRepository,
        TUserInfoSyncRequestedRepository,
    > UserPort
    for UserService<
        TAuthenticatedUserRepository,
        TRoleSyncRequestedRepository,
        TUserInfoSyncRequestedRepository,
    >
where
    TAuthenticatedUserRepository: AuthenticatedUserRepository + Send + Sync,
    TRoleSyncRequestedRepository: RoleSyncRequestedRepository + Send + Sync,
    TUserInfoSyncRequestedRepository: UserInfoSyncRequestedRepository + Send + Sync,
{
    #[instrument(level = "info", skip(self))]
    async fn get_user_info(
        &mut self,
        user_id: UserId,
    ) -> Result<Option<AuthenticatedUserInfoDto>, UserError> {
        let user = match self
            .authenticated_user_repository
            .find_by_user_id(user_id)
            .await
            .map_err(map_user_repo_err)?
        {
            None => return Ok(None),
            Some(user) => user,
        };

        Ok(Some(AuthenticatedUserInfoDto {
            user_id: user.user_id(),
            name: user.name().to_string(),
            email: user.email().to_string(),
            class_id: user.class_id().map(|s| s.to_string()),
            authenticated_at: user.authenticated_at(),
        }))
    }

    #[instrument(level = "info", skip(self))]
    async fn refresh_user_roles(&mut self, user_id: UserId) -> Result<(), UserError> {
        let request = request_role_sync(user_id);
        self.role_sync_requested_repository
            .save(&request)
            .await
            .map_err(map_role_sync_req_repo_err)?;
        info!(user_id = user_id.0, "Role sync requested successfully");
        Ok(())
    }

    #[instrument(level = "info", skip(self))]
    async fn refresh_user_info(&mut self, user_id: UserId) -> Result<Duration, UserError> {
        let request = request_user_info_sync(user_id);
        self.user_info_sync_requested_repository
            .save(&request)
            .await
            .map_err(map_user_info_sync_req_repo_err)?;

        info!(user_id = user_id.0, "User info sync requested successfully");

        Ok(Duration::milliseconds(750))
    }
}

#[instrument(level = "trace", skip_all)]
fn map_user_repo_err(err: AuthenticatedUserRepositoryError) -> UserError {
    match err {
        AuthenticatedUserRepositoryError::ServiceUnavailable => UserError::TemporaryUnavailable,
    }
}

#[instrument(level = "trace", skip_all)]
fn map_role_sync_req_repo_err(err: RoleSyncRequestedRepositoryError) -> UserError {
    match err {
        RoleSyncRequestedRepositoryError::ServiceUnavailable => UserError::TemporaryUnavailable,
    }
}

#[instrument(level = "trace", skip_all)]
fn map_user_info_sync_req_repo_err(err: UserInfoSyncRequestedRepositoryError) -> UserError {
    match err {
        UserInfoSyncRequestedRepositoryError::ServiceUnavailable => UserError::TemporaryUnavailable,
    }
}
