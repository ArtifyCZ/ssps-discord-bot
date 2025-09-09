use application_ports::user::{AuthenticatedUserInfoDto, UserError, UserPort};
use async_trait::async_trait;
use chrono::Utc;
use domain::authentication::authenticated_user::AuthenticatedUserRepository;
use domain::class::class_group::find_class_group;
use domain::class::class_id::get_class_id;
use domain::ports::discord::DiscordPort;
use domain::ports::oauth::OAuthPort;
use domain_shared::discord::UserId;
use std::sync::Arc;
use tracing::{info, instrument};

pub struct UserService {
    discord_port: Arc<dyn DiscordPort + Send + Sync>,
    oauth_port: Arc<dyn OAuthPort + Send + Sync>,
    authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
}

impl UserService {
    #[instrument(level = "trace", skip_all)]
    pub fn new(
        discord_port: Arc<dyn DiscordPort + Send + Sync>,
        oauth_port: Arc<dyn OAuthPort + Send + Sync>,
        authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
    ) -> Self {
        Self {
            discord_port,
            oauth_port,
            authenticated_user_repository,
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
            .await?
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
            .await?
            .ok_or(UserError::AuthenticatedUserNotFound)?;

        if user.oauth_token().expires_at < Utc::now() {
            info!(
                user_id = user.user_id().0,
                "User's OAuth token is expired, refreshing it",
            );
            user.update_oauth_token(self.oauth_port.refresh_token(user.oauth_token()).await?);
        }

        let user_info = self
            .oauth_port
            .get_user_info(&user.oauth_token().access_token)
            .await?;

        let groups = self
            .oauth_port
            .get_user_groups(&user.oauth_token().access_token)
            .await?;
        let class_group = find_class_group(&groups)
            .ok_or_else(|| UserError::Error("User is not in the Class group".into()))?;
        let class_id = get_class_id(class_group)
            .ok_or_else(|| UserError::Error("User's class group ID not found".into()))?;

        user.set_user_info(user_info.name, user_info.email, class_id);
        self.authenticated_user_repository.save(&user).await?;
        info!(
            user_id = user.user_id().0,
            "User info refreshed successfully",
        );

        let audit_log_reason = "Assigned student roles by OAuth2 Azure AD authentication";

        self.discord_port
            .set_user_class_role(user_id, Some(user.class_id()), audit_log_reason)
            .await?;

        info!(
            user_id = user.user_id().0,
            "User data refreshed successfully",
        );

        Ok(())
    }
}
