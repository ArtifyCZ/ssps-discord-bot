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
use domain::ports::discord::DiscordPort;
use domain::ports::oauth::{OAuthError, OAuthPort};
use domain_shared::authentication::{AuthenticationLink, ClientCallbackToken, CsrfToken};
use domain_shared::discord::{RoleId, UserId};
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
    invite_link: InviteLink,
    additional_student_roles: Vec<RoleId>,
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
        invite_link: InviteLink,
        additional_student_roles: Vec<RoleId>,
    ) -> Self {
        Self {
            discord_port,
            oauth_port,
            archived_authenticated_user_repository,
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
    async fn create_authentication_link(
        &self,
        user_id: UserId,
    ) -> Result<AuthenticationLink, AuthenticationError> {
        let (link, csrf_token) = self.oauth_port.create_authentication_link().await?;

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
            .await?;
        let user_info = self
            .oauth_port
            .get_user_info(&oauth_token.access_token)
            .await
            .map_err(|err| match err {
                OAuthError::OAuthUnavailable => {
                    AuthenticationError::Error("OAuth is unavailable".into())
                }
                OAuthError::TokenExpired => {
                    error!(
                        user_id = user_id.0,
                        "User's OAuth token expired during authentication process",
                    );
                    AuthenticationError::Error("OAuth token expired".into())
                }
            })?;
        let class_group = find_class_group(&user_info.groups)
            .ok_or_else(|| AuthenticationError::Error("User is not in the Class group".into()))?;
        let class_id = get_class_id(class_group)
            .ok_or_else(|| AuthenticationError::Error("User's class group ID not found".into()))?;

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

            self.discord_port
                .set_user_class_role(user.user_id(), None, audit_log_reason)
                .await?;

            self.discord_port
                .remove_user_from_roles(
                    user.user_id(),
                    &self.additional_student_roles,
                    audit_log_reason,
                )
                .await?;
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

        self.discord_port
            .set_user_class_role(user_id, Some(user.class_id()), audit_log_reason)
            .await?;

        self.discord_port
            .assign_roles_to_user_if_not_assigned(
                user_id,
                &self.additional_student_roles,
                audit_log_reason,
            )
            .await?;

        info!(user_id = user_id.0, "User successfully authenticated");

        Ok(self.invite_link.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::authentication::archived_authenticated_user::MockArchivedAuthenticatedUserRepository;
    use domain::authentication::authenticated_user::MockAuthenticatedUserRepository;
    use domain::authentication::user_authentication_request::MockUserAuthenticationRequestRepository;
    use domain::ports::discord::MockDiscordPort;
    use domain::ports::oauth::MockOAuthPort;
    use std::sync::Arc;
    use tokio;

    #[instrument(level = "trace", skip_all)]
    fn setup_mocks() -> (
        MockDiscordPort,
        MockOAuthPort,
        MockArchivedAuthenticatedUserRepository,
        MockAuthenticatedUserRepository,
        MockUserAuthenticationRequestRepository,
    ) {
        let discord_port = MockDiscordPort::new();
        let oauth_port = MockOAuthPort::new();
        let archived_authenticated_user_repository = MockArchivedAuthenticatedUserRepository::new();
        let authenticated_user_repository = MockAuthenticatedUserRepository::new();
        let user_authentication_request_repository = MockUserAuthenticationRequestRepository::new();

        (
            discord_port,
            oauth_port,
            archived_authenticated_user_repository,
            authenticated_user_repository,
            user_authentication_request_repository,
        )
    }

    #[instrument(level = "trace", skip_all)]
    fn create_service(
        discord_port: MockDiscordPort,
        oauth_port: MockOAuthPort,
        archived_authenticated_user_repository: MockArchivedAuthenticatedUserRepository,
        authenticated_user_repository: MockAuthenticatedUserRepository,
        user_authentication_request_repository: MockUserAuthenticationRequestRepository,
    ) -> AuthenticationService {
        let invite_link = InviteLink("http://discord.gg/invite".into());
        let additional_student_roles = vec![RoleId(123), RoleId(456)];
        AuthenticationService::new(
            Arc::new(discord_port),
            Arc::new(oauth_port),
            Arc::new(archived_authenticated_user_repository),
            Arc::new(authenticated_user_repository),
            Arc::new(user_authentication_request_repository),
            invite_link,
            additional_student_roles,
        )
    }

    #[tokio::test]
    async fn create_authentication_link_success() {
        let (
            discord_port,
            mut oauth_port,
            archived_authenticated_user_repo,
            authenticated_user_repo,
            mut user_auth_request_repo,
        ) = setup_mocks();
        let user_id = UserId(98765);
        let expected_link = AuthenticationLink("http://oauth.com/auth?csrf=...&...".to_string());
        let expected_csrf_token = CsrfToken("random_csrf_token".to_string());

        let link_clone = expected_link.0.clone();
        let csrf_clone = expected_csrf_token.clone();
        oauth_port
            .expect_create_authentication_link()
            .times(1)
            .returning(move || Ok((AuthenticationLink(link_clone.clone()), csrf_clone.clone())));

        let csrf_clone_for_save = expected_csrf_token.clone();
        user_auth_request_repo
            .expect_save()
            .withf(move |req| req.user_id() == user_id && req.csrf_token() == &csrf_clone_for_save)
            .times(1)
            .returning(|_| Ok(()));

        let service = create_service(
            discord_port,
            oauth_port,
            archived_authenticated_user_repo,
            authenticated_user_repo,
            user_auth_request_repo,
        );

        let result = service.create_authentication_link(user_id).await;

        assert!(result.is_ok());
        let returned_link = result.unwrap();
        assert_eq!(returned_link.0, expected_link.0);
    }

    #[tokio::test]
    async fn confirm_authentication_request_not_found() {
        let (
            discord_port,
            oauth_port,
            archived_authenticated_user_repo,
            authenticated_user_repo,
            mut user_auth_request_repo, // mut because expect_find_by_csrf_token is called
        ) = setup_mocks();

        let csrf_token = CsrfToken("non_existent_token".to_string());
        let client_callback_token = ClientCallbackToken("callback_code".to_string());

        user_auth_request_repo
            .expect_find_by_csrf_token()
            .with(mockall::predicate::eq(csrf_token.clone()))
            .times(1)
            .returning(|_| Ok(None)); // Request not found

        let service = create_service(
            discord_port,
            oauth_port,
            archived_authenticated_user_repo,
            authenticated_user_repo,
            user_auth_request_repo,
        );

        let result = service
            .confirm_authentication(csrf_token, client_callback_token)
            .await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AuthenticationError::AuthenticationRequestNotFound => {}
            other => panic!("Expected AuthenticationRequestNotFound, got {:?}", other),
        }
    }
}
