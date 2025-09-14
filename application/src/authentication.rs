use application_ports::authentication::{AuthenticationError, AuthenticationPort};
use application_ports::discord::InviteLink;
use async_trait::async_trait;
use domain::authentication::archived_authenticated_user::{
    create_archived_authenticated_user_from_user, ArchivedAuthenticatedUserRepository,
};
use domain::authentication::authenticated_user::{
    create_user_from_successful_authentication, AuthenticatedUserRepository,
};
use domain::authentication::user_authentication_request::{
    create_user_authentication_request, UserAuthenticationRequestRepository,
};
use domain::class::class_group::find_class_group;
use domain::class::class_id::get_class_id;
use domain::ports::discord::{DiscordError, DiscordPort};
use domain::ports::oauth::{OAuthError, OAuthPort};
use domain::user_role_service::UserRoleService;
use domain_shared::authentication::{AuthenticationLink, ClientCallbackToken, CsrfToken};
use domain_shared::discord::UserId;
use std::sync::Arc;
use tracing::{error, info, instrument, warn, Span};

pub struct AuthenticationService {
    discord_port: Arc<dyn DiscordPort + Send + Sync>,
    oauth_port: Arc<dyn OAuthPort + Send + Sync>,
    archived_authenticated_user_repository:
        Arc<dyn ArchivedAuthenticatedUserRepository + Send + Sync>,
    authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
    user_authentication_request_repository:
        Arc<dyn UserAuthenticationRequestRepository + Send + Sync>,
    user_role_service: Arc<UserRoleService>,
    invite_link: InviteLink,
}

impl AuthenticationService {
    #[instrument(level = "trace", skip_all)]
    pub fn new(
        discord_port: Arc<dyn DiscordPort + Send + Sync>,
        oauth_port: Arc<dyn OAuthPort + Send + Sync>,
        archived_authenticated_user_repository: Arc<
            dyn ArchivedAuthenticatedUserRepository + Send + Sync,
        >,
        authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
        user_authentication_request_repository: Arc<
            dyn UserAuthenticationRequestRepository + Send + Sync,
        >,
        user_role_service: Arc<UserRoleService>,
        invite_link: InviteLink,
    ) -> Self {
        Self {
            discord_port,
            oauth_port,
            archived_authenticated_user_repository,
            authenticated_user_repository,
            user_authentication_request_repository,
            user_role_service,
            invite_link,
        }
    }
}

#[async_trait]
impl AuthenticationPort for AuthenticationService {
    #[instrument(level = "info", skip(self))]
    async fn create_authentication_link(
        &self,
        user_id: UserId,
    ) -> Result<AuthenticationLink, AuthenticationError> {
        let (link, csrf_token) = self.oauth_port.create_authentication_link().await;

        let request = create_user_authentication_request(csrf_token, user_id);

        self.user_authentication_request_repository
            .save(&request)
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
            .find_by_csrf_token(&csrf_token)
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
        let user_id = request.user_id();
        Span::current().record("user_id", user_id.0);

        let oauth_token = self
            .oauth_port
            .exchange_code_after_callback(client_callback_token)
            .await
            .map_err(|err| match err {
                OAuthError::OAuthUnavailable => AuthenticationError::TemporaryUnavailable,
                OAuthError::TokenExpired => {
                    error!(
                        user_id = user_id.0,
                        "User's OAuth token expired during authentication process",
                    );
                    AuthenticationError::TemporaryUnavailable
                }
            })?;
        let user_info = self
            .oauth_port
            .get_user_info(&oauth_token.access_token)
            .await
            .map_err(|err| match err {
                OAuthError::OAuthUnavailable => AuthenticationError::TemporaryUnavailable,
                OAuthError::TokenExpired => {
                    error!(
                        user_id = user_id.0,
                        "User's OAuth token expired during authentication process",
                    );
                    AuthenticationError::TemporaryUnavailable
                }
            })?;
        let class_group = find_class_group(&user_info.groups).ok_or_else(|| {
            warn!(
                user_id = user_id.0,
                groups = ?&user_info.groups,
                "Could not find class group in user's groups",
            );
            AuthenticationError::TemporaryUnavailable
        })?;
        let class_id = get_class_id(class_group).ok_or_else(|| {
            warn!(
                user_id = user_id.0,
                class_group = ?class_group,
                "Could not find class ID from class group",
            );
            AuthenticationError::TemporaryUnavailable
        })?;

        if let Some(user) = self
            .authenticated_user_repository
            .find_by_email(&user_info.email)
            .await?
        {
            warn!(
                user_id = user_id.0,
                email = user.email(),
                "User tried to authenticate with an already used email"
            );
            let archived_user = create_archived_authenticated_user_from_user(&user);
            self.archived_authenticated_user_repository
                .save(&archived_user)
                .await?;
            self.authenticated_user_repository
                .remove(user.user_id())
                .await?;

            let audit_log_reason =
                "Removed user roles due to new user authenticating with the same email";

            let diff = self.user_role_service.remove_user_roles();
            self.discord_port
                .apply_role_diff(user.user_id(), &diff, audit_log_reason)
                .await
                .map_err(|err| match err {
                    DiscordError::DiscordUnavailable => AuthenticationError::TemporaryUnavailable,
                })?;
        }

        let user = create_user_from_successful_authentication(
            &request,
            user_info.name,
            user_info.email,
            oauth_token,
            class_id,
        );

        self.authenticated_user_repository.save(&user).await?;
        self.user_authentication_request_repository
            .remove_by_csrf_token(request.csrf_token())
            .await?;

        let audit_log_reason = "Assigned student roles by OAuth2 Azure AD authentication";

        let diff = self.user_role_service.assign_user_roles(&user);

        self.discord_port
            .apply_role_diff(user.user_id(), &diff, audit_log_reason)
            .await
            .map_err(|err| match err {
                DiscordError::DiscordUnavailable => AuthenticationError::TemporaryUnavailable,
            })?;

        info!(user_id = user_id.0, "User successfully authenticated");

        Ok(self.invite_link.clone())
    }
}
