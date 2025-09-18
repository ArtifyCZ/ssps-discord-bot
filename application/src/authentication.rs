use application_ports::authentication::{AuthenticationError, AuthenticationPort};
use application_ports::discord::InviteLink;
use async_trait::async_trait;
use domain::authentication::archived_authenticated_user::{
    create_archived_authenticated_user_from_user, ArchivedAuthenticatedUserRepository,
    ArchivedAuthenticatedUserRepositoryError,
};
use domain::authentication::authenticated_user::{
    create_user_from_successful_authentication, AuthenticatedUserRepository,
    AuthenticatedUserRepositoryError,
};
use domain::authentication::user_authentication_request::{
    create_user_authentication_request, UserAuthenticationRequestRepository,
    UserAuthenticationRequestRepositoryError,
};
use domain::class::class_group::find_class_group;
use domain::class::class_id::get_class_id;
use domain::jobs::role_sync_job::{
    request_role_sync, RoleSyncRequestedRepository, RoleSyncRequestedRepositoryError,
};
use domain::ports::oauth::{OAuthError, OAuthPort};
use domain_shared::authentication::{AuthenticationLink, ClientCallbackToken, CsrfToken};
use domain_shared::discord::UserId;
use std::sync::Arc;
use tracing::{error, info, instrument, warn, Span};

pub struct AuthenticationService {
    oauth_port: Arc<dyn OAuthPort + Send + Sync>,
    archived_authenticated_user_repository:
        Arc<dyn ArchivedAuthenticatedUserRepository + Send + Sync>,
    authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
    user_authentication_request_repository:
        Arc<dyn UserAuthenticationRequestRepository + Send + Sync>,
    role_sync_requested_repository: Arc<dyn RoleSyncRequestedRepository + Send + Sync>,
    invite_link: InviteLink,
}

impl AuthenticationService {
    #[instrument(level = "trace", skip_all)]
    pub fn new(
        oauth_port: Arc<dyn OAuthPort + Send + Sync>,
        archived_authenticated_user_repository: Arc<
            dyn ArchivedAuthenticatedUserRepository + Send + Sync,
        >,
        authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
        user_authentication_request_repository: Arc<
            dyn UserAuthenticationRequestRepository + Send + Sync,
        >,
        role_sync_requested_repository: Arc<dyn RoleSyncRequestedRepository + Send + Sync>,
        invite_link: InviteLink,
    ) -> Self {
        Self {
            oauth_port,
            archived_authenticated_user_repository,
            authenticated_user_repository,
            user_authentication_request_repository,
            role_sync_requested_repository,
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
            .await
            .map_err(map_auth_req_repo_err)?;

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
            .await
            .map_err(map_auth_req_repo_err)?
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
            error!(
                user_id = user_id.0,
                groups = ?&user_info.groups,
                "Could not find class group in user's groups",
            );
            AuthenticationError::TemporaryUnavailable
        })?;
        let class_id = get_class_id(class_group).ok_or_else(|| {
            error!(
                user_id = user_id.0,
                class_group = ?class_group,
                "Could not find class ID from class group",
            );
            AuthenticationError::TemporaryUnavailable
        })?;

        if let Some(user) = self
            .authenticated_user_repository
            .find_by_email(&user_info.email)
            .await
            .map_err(map_user_repo_err)?
        {
            warn!(
                user_id = user_id.0,
                email = user.email(),
                "User tried to authenticate with an already used email"
            );
            let archived_user = create_archived_authenticated_user_from_user(&user);
            self.archived_authenticated_user_repository
                .save(&archived_user)
                .await
                .map_err(map_archived_user_repo_err)?;
            self.authenticated_user_repository
                .remove(user.user_id())
                .await
                .map_err(map_user_repo_err)?;

            info!(
                user_id = user.user_id().0,
                "Removing user roles due to new user authenticating with the same email asynchronously",
            );

            let role_sync_request = request_role_sync(user.user_id());

            self.role_sync_requested_repository
                .save(&role_sync_request)
                .await
                .map_err(map_role_sync_req_repo_err)?;
        }

        let user = create_user_from_successful_authentication(
            &request,
            user_info.name,
            user_info.email,
            oauth_token,
            class_id,
        );

        self.authenticated_user_repository
            .save(&user)
            .await
            .map_err(map_user_repo_err)?;
        self.user_authentication_request_repository
            .remove_by_csrf_token(request.csrf_token())
            .await
            .map_err(map_auth_req_repo_err)?;

        info!(
            user_id = user.user_id().0,
            "Assigning student roles by OAuth2 Azure AD authentication asynchronously",
        );

        let role_sync_request = request_role_sync(user.user_id());
        self.role_sync_requested_repository
            .save(&role_sync_request)
            .await
            .map_err(map_role_sync_req_repo_err)?;

        info!(user_id = user_id.0, "User successfully authenticated");

        Ok(self.invite_link.clone())
    }
}

#[instrument(level = "trace", skip_all)]
fn map_user_repo_err(err: AuthenticatedUserRepositoryError) -> AuthenticationError {
    match err {
        AuthenticatedUserRepositoryError::ServiceUnavailable => {
            AuthenticationError::TemporaryUnavailable
        }
    }
}

#[instrument(level = "trace", skip_all)]
fn map_archived_user_repo_err(
    err: ArchivedAuthenticatedUserRepositoryError,
) -> AuthenticationError {
    match err {
        ArchivedAuthenticatedUserRepositoryError::ServiceUnavailable => {
            AuthenticationError::TemporaryUnavailable
        }
    }
}

#[instrument(level = "trace", skip_all)]
fn map_auth_req_repo_err(err: UserAuthenticationRequestRepositoryError) -> AuthenticationError {
    match err {
        UserAuthenticationRequestRepositoryError::TemporaryUnavailable => {
            AuthenticationError::TemporaryUnavailable
        }
    }
}

#[instrument(level = "trace", skip_all)]
fn map_role_sync_req_repo_err(err: RoleSyncRequestedRepositoryError) -> AuthenticationError {
    match err {
        RoleSyncRequestedRepositoryError::ServiceUnavailable => {
            AuthenticationError::TemporaryUnavailable
        }
    }
}
