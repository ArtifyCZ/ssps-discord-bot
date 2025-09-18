use application_ports::user::{AuthenticatedUserInfoDto, UserError, UserPort};
use async_trait::async_trait;
use chrono::Utc;
use domain::authentication::authenticated_user::{
    AuthenticatedUserRepository, AuthenticatedUserRepositoryError,
};
use domain::class::class_group::find_class_group;
use domain::class::class_id::get_class_id;
use domain::jobs::role_sync_job::{
    request_role_sync, RoleSyncRequestedRepository, RoleSyncRequestedRepositoryError,
};
use domain::ports::oauth::{OAuthError, OAuthPort};
use domain_shared::discord::UserId;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

pub struct UserService {
    oauth_port: Arc<dyn OAuthPort + Send + Sync>,
    authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
    role_sync_requested_repository: Arc<dyn RoleSyncRequestedRepository + Send + Sync>,
}

impl UserService {
    #[instrument(level = "trace", skip_all)]
    pub fn new(
        oauth_port: Arc<dyn OAuthPort + Send + Sync>,
        authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
        role_sync_requested_repository: Arc<dyn RoleSyncRequestedRepository + Send + Sync>,
    ) -> Self {
        Self {
            oauth_port,
            authenticated_user_repository,
            role_sync_requested_repository,
        }
    }
}

#[async_trait]
impl UserPort for UserService {
    #[instrument(level = "info", skip(self))]
    async fn get_user_info(
        &self,
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
            class_id: user.class_id().to_string(),
            authenticated_at: user.authenticated_at(),
        }))
    }

    #[instrument(level = "info", skip(self))]
    async fn refresh_user_data(&self, user_id: UserId) -> Result<(), UserError> {
        let mut user = self
            .authenticated_user_repository
            .find_by_user_id(user_id)
            .await
            .map_err(map_user_repo_err)?
            .ok_or(UserError::AuthenticatedUserNotFound)?;

        if user.oauth_token().expires_at < Utc::now() {
            info!(
                user_id = user.user_id().0,
                "User's OAuth token is expired, refreshing it",
            );
            match self.oauth_port.refresh_token(user.oauth_token()).await {
                Ok(new_token) => user.update_oauth_token(new_token),
                Err(err) => {
                    return Err(match err {
                        OAuthError::OAuthUnavailable => UserError::TemporaryUnavailable,
                        OAuthError::TokenExpired => {
                            warn!(
                                user_id = user.user_id().0,
                                "User's OAuth refresh token is expired, requesting reauthentication",
                            );
                            // @TODO: request the user to reauthenticate
                            UserError::TemporaryUnavailable
                        }
                    });
                }
            };
        }

        let user_info = self
            .oauth_port
            .get_user_info(&user.oauth_token().access_token)
            .await
            .map_err(map_oauth_err)?;

        let class_group = find_class_group(&user_info.groups).ok_or_else(|| {
            error!(
                user_id = user.user_id().0,
                groups = ?&user_info.groups,
                "Could not find class group in user's groups",
            );
            UserError::TemporaryUnavailable
        })?;
        let class_id = get_class_id(class_group).ok_or_else(|| {
            error!(
                user_id = user.user_id().0,
                class_group = ?&class_group,
                "Could not find class ID for class group",
            );
            UserError::TemporaryUnavailable
        })?;

        user.set_user_info(user_info.name, user_info.email, class_id);
        self.authenticated_user_repository
            .save(&user)
            .await
            .map_err(map_user_repo_err)?;
        info!(
            user_id = user.user_id().0,
            "User info refreshed successfully",
        );

        let role_sync_request = request_role_sync(user.user_id());
        self.role_sync_requested_repository
            .save(&role_sync_request)
            .await
            .map_err(map_role_sync_req_repo_err)?;

        info!(
            user_id = user.user_id().0,
            "User data refreshed successfully",
        );

        Ok(())
    }
}

#[instrument(level = "trace", skip_all)]
fn map_user_repo_err(err: AuthenticatedUserRepositoryError) -> UserError {
    match err {
        AuthenticatedUserRepositoryError::ServiceUnavailable => UserError::TemporaryUnavailable,
    }
}

#[instrument(level = "trace", skip_all)]
fn map_oauth_err(err: OAuthError) -> UserError {
    match err {
        OAuthError::OAuthUnavailable => UserError::TemporaryUnavailable,
        OAuthError::TokenExpired => {
            error!("User's OAuth access token or refresh token is expired, and this case should be covered");
            UserError::TemporaryUnavailable
        }
    }
}

#[instrument(level = "trace", skip_all)]
fn map_role_sync_req_repo_err(err: RoleSyncRequestedRepositoryError) -> UserError {
    match err {
        RoleSyncRequestedRepositoryError::ServiceUnavailable => UserError::TemporaryUnavailable,
    }
}
