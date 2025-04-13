use application_ports::authentication::{
    AuthenticatedUserInfoDto, AuthenticationError, AuthenticationPort,
};
use application_ports::discord::InviteLink;
use async_trait::async_trait;
use chrono::Utc;
use domain::authentication::authenticated_user::{AuthenticatedUser, AuthenticatedUserRepository};
use domain::authentication::create_class_user_group_id_mails;
use domain::authentication::user_authentication_request::{
    UserAuthenticationRequest, UserAuthenticationRequestRepository,
};
use domain::ports::discord::DiscordPort;
use domain::ports::oauth::OAuthPort;
use domain_shared::authentication::{
    AuthenticationLink, ClientCallbackToken, CsrfToken, UserGroup,
};
use domain_shared::discord::{RoleId, UserId};
use std::sync::Arc;
use tracing::{info, instrument, warn, Span};

pub struct AuthenticationService {
    discord_port: Arc<dyn DiscordPort + Send + Sync>,
    oauth_port: Arc<dyn OAuthPort + Send + Sync>,
    authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
    user_authentication_request_repository:
        Arc<dyn UserAuthenticationRequestRepository + Send + Sync>,
    invite_link: InviteLink,
    additional_student_roles: Vec<RoleId>,
}

impl AuthenticationService {
    #[instrument(level = "trace", skip_all)]
    pub fn new(
        discord_port: Arc<dyn DiscordPort + Send + Sync>,
        oauth_port: Arc<dyn OAuthPort + Send + Sync>,
        authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
        user_authentication_request_repository: Arc<
            dyn UserAuthenticationRequestRepository + Send + Sync,
        >,
        invite_link: InviteLink,
        additional_student_roles: Vec<RoleId>,
    ) -> Self {
        Self {
            discord_port,
            oauth_port,
            authenticated_user_repository,
            user_authentication_request_repository,
            invite_link,
            additional_student_roles,
        }
    }
}

#[async_trait]
impl AuthenticationPort for AuthenticationService {
    #[instrument(level = "info", skip(self))]
    async fn get_user_info(
        &self,
        user_id: UserId,
    ) -> Result<Option<AuthenticatedUserInfoDto>, AuthenticationError> {
        let mut user = match self
            .authenticated_user_repository
            .find_by_user_id(user_id)
            .await?
        {
            Some(user) => user,
            None => {
                info!(
                    user_id = user_id.0,
                    "Tried to get user info of an unauthenticated user"
                );
                return Ok(None);
            }
        };

        if user.oauth_token.expires_at < Utc::now() {
            info!(
                user_id = user_id.0,
                "User's OAuth token is expired, refreshing it",
            );
            user.oauth_token = self.oauth_port.refresh_token(user.oauth_token).await?;
        }

        let user_info = self
            .oauth_port
            .get_user_info(&user.oauth_token.access_token)
            .await?;

        let name = user_info.name;
        let email = user_info.email;
        let class_id = user.class_id.clone();
        let authenticated_at = user.authenticated_at;
        user.name = Some(name.clone());
        user.email = Some(email.clone());
        self.authenticated_user_repository.save(user).await?;

        info!(user_id = user_id.0, "User info retrieved successfully");

        Ok(Some(AuthenticatedUserInfoDto {
            user_id,
            name,
            email,
            class_id,
            authenticated_at,
        }))
    }

    #[instrument(level = "info", skip(self))]
    async fn create_authentication_link(
        &self,
        user_id: UserId,
    ) -> Result<AuthenticationLink, AuthenticationError> {
        if let Some(_user) = self
            .authenticated_user_repository
            .find_by_user_id(user_id)
            .await?
        {
            info!(user_id = user_id.0, "The user tried to authenticate again");
            return Err(AuthenticationError::AlreadyAuthenticated);
        }

        let (link, csrf_token) = self.oauth_port.create_authentication_link().await?;

        let request = UserAuthenticationRequest {
            csrf_token,
            user_id,
            requested_at: Utc::now(),
        };

        self.user_authentication_request_repository
            .save(request)
            .await?;

        info!(user_id = user_id.0, "Authentication link created");

        Ok(link)
    }

    #[instrument(level = "info", skip(self, csrf_token, client_callback_token))]
    async fn confirm_authentication(
        &self,
        csrf_token: CsrfToken,
        client_callback_token: ClientCallbackToken,
    ) -> Result<InviteLink, AuthenticationError> {
        let request = match self
            .user_authentication_request_repository
            .find_by_csrf_token(csrf_token.clone())
            .await?
        {
            Some(request) => request,
            None => {
                warn!(
                    csrf_token = csrf_token.0,
                    "The user tried to authenticate with an invalid CSRF token",
                );
                return Err(AuthenticationError::AuthenticationRequestNotFound);
            }
        };
        Span::current().record("user_id", request.user_id.0);

        let oauth_token = self
            .oauth_port
            .exchange_code_after_callback(client_callback_token)
            .await?;
        let groups = self
            .oauth_port
            .get_user_groups(&oauth_token.access_token)
            .await?;
        let class_group = find_class_group(&groups)
            .ok_or_else(|| AuthenticationError::Error("User is not in the Class group".into()))?;
        let class_id = get_class_id(class_group)
            .ok_or_else(|| AuthenticationError::Error("User's class group ID not found".into()))?;
        let user_info = self
            .oauth_port
            .get_user_info(&oauth_token.access_token)
            .await?;

        let user = AuthenticatedUser {
            user_id: request.user_id,
            name: Some(user_info.name),
            email: Some(user_info.email),
            oauth_token,
            class_id: class_id.clone(),
            authenticated_at: Utc::now(),
        };

        let user_id = request.user_id;

        self.authenticated_user_repository.save(user).await?;
        self.user_authentication_request_repository
            .remove(request)
            .await?;

        let audit_log_reason = "Assigned student roles by OAuth2 Azure AD authentication";

        self.discord_port
            .remove_user_from_class_roles(user_id, Some(audit_log_reason))
            .await?;
        for role in &self.additional_student_roles {
            self.discord_port
                .remove_user_from_role(user_id, *role, Some(audit_log_reason))
                .await?;
        }

        self.discord_port
            .assign_user_to_class_role(user_id, class_id, Some(audit_log_reason))
            .await?;
        for role in &self.additional_student_roles {
            self.discord_port
                .assign_user_to_role(user_id, *role, Some(audit_log_reason))
                .await?;
        }

        info!(user_id = user_id.0, "User successfully authenticated");

        Ok(self.invite_link.clone())
    }
}

#[instrument(level = "trace")]
fn get_class_id(group: &UserGroup) -> Option<String> {
    if let Some(mail) = &group.mail {
        let class_group_id_mails = create_class_user_group_id_mails();
        class_group_id_mails
            .into_iter()
            .find(|(_, m)| m.eq(mail))
            .map(|(id, _)| id)
    } else {
        None
    }
}

#[instrument(level = "trace")]
fn find_class_group(groups: &[UserGroup]) -> Option<&UserGroup> {
    let class_group_id_mails = create_class_user_group_id_mails();
    let class_group_mails = class_group_id_mails
        .iter()
        .map(|(_, mail)| mail)
        .collect::<Vec<_>>();
    groups.iter().find(|group| {
        group
            .mail
            .as_ref()
            .map(|mail| class_group_mails.contains(&mail))
            .unwrap_or(false)
    })
}
