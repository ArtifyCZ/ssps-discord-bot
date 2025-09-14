use application_ports::user::{AuthenticatedUserInfoDto, UserError, UserPort};
use async_trait::async_trait;
use chrono::Utc;
use domain::authentication::authenticated_user::AuthenticatedUserRepository;
use domain::class::class_group::find_class_group;
use domain::class::class_id::get_class_id;
use domain::ports::discord::{DiscordError, DiscordPort};
use domain::ports::oauth::{OAuthError, OAuthPort};
use domain::user_role_service::UserRoleService;
use domain_shared::discord::UserId;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

pub struct UserService {
    discord_port: Arc<dyn DiscordPort + Send + Sync>,
    oauth_port: Arc<dyn OAuthPort + Send + Sync>,
    authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
    user_role_service: Arc<UserRoleService>,
}

impl UserService {
    #[instrument(level = "trace", skip_all)]
    pub fn new(
        discord_port: Arc<dyn DiscordPort + Send + Sync>,
        oauth_port: Arc<dyn OAuthPort + Send + Sync>,
        authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
        user_role_service: Arc<UserRoleService>,
    ) -> Self {
        Self {
            discord_port,
            oauth_port,
            authenticated_user_repository,
            user_role_service,
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
            .map_err(|err| {
                // @TODO: implement proper error handling
                warn!(
                    user_id = user_id.0,
                    error = ?err,
                    "Failed to find user in database",
                );
                UserError::TemporaryUnavailable
            })? {
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
            .map_err(|err| {
                // @TODO: implement proper error handling
                warn!(
                    user_id = user_id.0,
                    error = ?err,
                    "Failed to find user in database",
                );
                UserError::TemporaryUnavailable
            })?
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
            .map_err(|err| match err {
                OAuthError::OAuthUnavailable => UserError::TemporaryUnavailable,
                OAuthError::TokenExpired => {
                    error!(
                    user_id = user.user_id().0,
                    "User's OAuth access token is expired after refresh, this should not happen",
                );
                    UserError::TemporaryUnavailable
                }
            })?;

        let class_group = find_class_group(&user_info.groups).ok_or_else(|| {
            warn!(
                user_id = user.user_id().0,
                groups = ?&user_info.groups,
                "Could not find class group in user's groups",
            );
            UserError::TemporaryUnavailable
        })?;
        let class_id = get_class_id(class_group).ok_or_else(|| {
            warn!(
                user_id = user.user_id().0,
                class_group = ?&class_group,
                "Could not find class ID for class group",
            );
            UserError::TemporaryUnavailable
        })?;

        user.set_user_info(user_info.name, user_info.email, class_id);
        match self.authenticated_user_repository.save(&user).await {
            Ok(()) => {}
            Err(err) => {
                // @TODO: implement proper error handling
                warn!(
                    user_id = user.user_id().0,
                    error = ?err,
                    "Failed to save user's info in database",
                );
                return Err(UserError::TemporaryUnavailable);
            }
        };
        info!(
            user_id = user.user_id().0,
            "User info refreshed successfully",
        );

        let audit_log_reason = "Assigned student roles by OAuth2 Azure AD authentication";

        let diff = self.user_role_service.assign_user_roles(&user);
        self.discord_port
            .apply_role_diff(user.user_id(), &diff, audit_log_reason)
            .await
            .map_err(|err| match err {
                DiscordError::DiscordUnavailable => UserError::TemporaryUnavailable,
            })?;

        info!(
            user_id = user.user_id().0,
            "User data refreshed successfully",
        );

        Ok(())
    }
}
