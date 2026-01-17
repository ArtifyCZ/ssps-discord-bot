use application_ports::user_info_sync_job_handler::{
    UserInfoSyncJobHandlerError, UserInfoSyncJobHandlerPort,
};
use async_trait::async_trait;
use chrono::{Duration, TimeDelta};
use domain::authentication::authenticated_user::{
    AuthenticatedUser, AuthenticatedUserRepository, AuthenticatedUserRepositoryError,
};
use domain::class::class_group::find_class_group;
use domain::class::class_id::get_class_id;
use domain::jobs::role_sync_job::{
    request_role_sync, RoleSyncRequestedRepository, RoleSyncRequestedRepositoryError,
};
use domain::jobs::user_info_sync_job::{
    UserInfoSyncRequested, UserInfoSyncRequestedRepository, UserInfoSyncRequestedRepositoryError,
};
use domain::ports::oauth::{OAuthError, OAuthPort, OAuthToken};
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

pub struct UserInfoSyncJobHandler<TAuthenticatedUserRepository, TOAuthAdapter> {
    authenticated_user_repository: TAuthenticatedUserRepository,
    role_sync_requested_repository: Arc<dyn RoleSyncRequestedRepository + Send + Sync>,
    user_info_sync_requested_repository: Arc<dyn UserInfoSyncRequestedRepository + Send + Sync>,
    oauth_port: TOAuthAdapter,
}

impl<TAuthenticatedUserRepository, TOAuthAdapter>
    UserInfoSyncJobHandler<TAuthenticatedUserRepository, TOAuthAdapter>
where
    TAuthenticatedUserRepository: AuthenticatedUserRepository + Send + Sync,
    TOAuthAdapter: OAuthPort + Send + Sync,
{
    #[instrument(level = "trace", skip_all)]
    pub fn new(
        authenticated_user_repository: TAuthenticatedUserRepository,
        role_sync_requested_repository: Arc<dyn RoleSyncRequestedRepository + Send + Sync>,
        user_info_sync_requested_repository: Arc<dyn UserInfoSyncRequestedRepository + Send + Sync>,
        oauth_port: TOAuthAdapter,
    ) -> Self {
        Self {
            authenticated_user_repository,
            role_sync_requested_repository,
            user_info_sync_requested_repository,
            oauth_port,
        }
    }

    #[instrument(level = "info", skip(self))]
    async fn handle(
        &self,
        request: UserInfoSyncRequested,
    ) -> Result<(), UserInfoSyncJobHandlerError> {
        const MIN_DURATION_SINCE_QUEUED: TimeDelta = Duration::milliseconds(400);
        const WAIT_TICK_DURATION: TimeDelta = Duration::milliseconds(100);
        let can_sync_since = request.queued_at + MIN_DURATION_SINCE_QUEUED;
        loop {
            if can_sync_since <= chrono::Utc::now() {
                break;
            }

            tokio::time::sleep(WAIT_TICK_DURATION.to_std().unwrap()).await;
        }

        let user = self
            .authenticated_user_repository
            .find_by_user_id(request.user_id)
            .await
            .map_err(map_user_repo_err)?;

        if let Some(mut user) = user {
            match self.handle_authenticated_user(&mut user).await {
                Ok(()) => {}
                Err(err) => {
                    // Handle the error gracefully to update the user's OAuth token and other info either way
                    error!(
                        user_id = user.user_id().0,
                        "User info sync job failed with error: {:?}", err,
                    );
                }
            };
            self.authenticated_user_repository
                .save(&user)
                .await
                .map_err(map_user_repo_err)?;
        }

        let request = request_role_sync(request.user_id);
        self.role_sync_requested_repository
            .save(&request)
            .await
            .map_err(map_role_sync_req_repo_err)?;

        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    async fn handle_authenticated_user(
        &self,
        user: &mut AuthenticatedUser,
    ) -> Result<(), UserInfoSyncJobHandlerError> {
        let token = match self.refresh_oauth_token(user).await? {
            None => {
                user.mark_class_unknown();
                return Ok(());
            }
            Some(token) => token,
        };

        user.update_oauth_token(token);

        let user_info = self
            .oauth_port
            .get_user_info(&user.oauth_token().access_token)
            .await
            .map_err(map_oauth_err)?;

        user.update_user_info(user_info.name, user_info.email);

        let class_group = find_class_group(&user_info.groups);
        let class_id = class_group.and_then(get_class_id);

        if let Some(class_id) = class_id {
            user.update_class_id(class_id);
        } else {
            user.mark_class_unknown();
        }

        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    async fn refresh_oauth_token(
        &self,
        user: &AuthenticatedUser,
    ) -> Result<Option<OAuthToken>, UserInfoSyncJobHandlerError> {
        info!(
            user_id = user.user_id().0,
            "Periodically refreshing user's OAuth token",
        );
        match self.oauth_port.refresh_token(user.oauth_token()).await {
            Ok(new_token) => Ok(Some(new_token)),
            Err(err) => match err {
                OAuthError::OAuthUnavailable => {
                    Err(UserInfoSyncJobHandlerError::TemporaryUnavailable)
                }
                OAuthError::TokenExpired => {
                    warn!(
                        user_id = user.user_id().0,
                        "User's OAuth refresh token is expired",
                    );
                    Ok(None)
                }
            },
        }
    }
}

#[async_trait]
impl<TAuthenticatedUserRepository, TOAuthAdapter> UserInfoSyncJobHandlerPort
    for UserInfoSyncJobHandler<TAuthenticatedUserRepository, TOAuthAdapter>
where
    TAuthenticatedUserRepository: AuthenticatedUserRepository + Send + Sync,
    TOAuthAdapter: OAuthPort + Send + Sync,
{
    #[instrument(level = "debug", skip_all)]
    async fn tick(&self) -> Result<(), UserInfoSyncJobHandlerError> {
        let high_priority = self
            .user_info_sync_requested_repository
            .pop_oldest(false)
            .await
            .map_err(map_sync_req_repo_err)?;

        if let Some(request) = high_priority {
            self.handle(request).await?;
            return Ok(());
        }

        let low_priority = self
            .user_info_sync_requested_repository
            .pop_oldest(true)
            .await
            .map_err(map_sync_req_repo_err)?;

        if let Some(request) = low_priority {
            self.handle(request).await?;
            return Ok(());
        }

        Err(UserInfoSyncJobHandlerError::NoRequestToHandle)
    }
}

#[instrument(level = "trace", skip_all)]
fn map_oauth_err(err: OAuthError) -> UserInfoSyncJobHandlerError {
    match err {
        OAuthError::OAuthUnavailable => {
            error!("OAuthError::OAuthUnavailable");
            UserInfoSyncJobHandlerError::TemporaryUnavailable
        }
        OAuthError::TokenExpired => {
            error!("User info sync job failed with expired token");
            UserInfoSyncJobHandlerError::TemporaryUnavailable
        }
    }
}

#[instrument(level = "trace", skip_all)]
fn map_user_repo_err(err: AuthenticatedUserRepositoryError) -> UserInfoSyncJobHandlerError {
    match err {
        AuthenticatedUserRepositoryError::ServiceUnavailable => {
            error!("AuthenticatedUserRepositoryError::ServiceUnavailable");
            UserInfoSyncJobHandlerError::TemporaryUnavailable
        }
    }
}

#[instrument(level = "trace", skip_all)]
fn map_sync_req_repo_err(err: UserInfoSyncRequestedRepositoryError) -> UserInfoSyncJobHandlerError {
    match err {
        UserInfoSyncRequestedRepositoryError::ServiceUnavailable => {
            error!("UserInfoSyncRequestedRepositoryError::ServiceUnavailable");
            UserInfoSyncJobHandlerError::TemporaryUnavailable
        }
    }
}

#[instrument(level = "trace", skip_all)]
fn map_role_sync_req_repo_err(
    err: RoleSyncRequestedRepositoryError,
) -> UserInfoSyncJobHandlerError {
    match err {
        RoleSyncRequestedRepositoryError::ServiceUnavailable => {
            error!("RoleSyncRequestedRepositoryError::ServiceUnavailable");
            UserInfoSyncJobHandlerError::TemporaryUnavailable
        }
    }
}
