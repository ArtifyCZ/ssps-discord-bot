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
use domain::jobs::role_sync_job::{
    request_role_sync, RoleSyncRequestedRepository, RoleSyncRequestedRepositoryError,
};
use domain::jobs::user_info_sync_job::{
    request_user_info_sync, UserInfoSyncRequestedRepository, UserInfoSyncRequestedRepositoryError,
};
use domain::ports::oauth::{OAuthError, OAuthPort};
use domain_shared::authentication::{AuthenticationLink, ClientCallbackToken, CsrfToken};
use domain_shared::discord::UserId;
use std::sync::Arc;
use tracing::{error, info, instrument, warn, Span};

pub struct AuthenticationService<
    TArchivedAuthenticatedUserRepository,
    TAuthenticatedUserRepository,
    TUserAuthenticationRequestRepository,
> {
    pub oauth_port: Arc<dyn OAuthPort + Send + Sync>,
    pub archived_authenticated_user_repository: TArchivedAuthenticatedUserRepository,
    pub authenticated_user_repository: TAuthenticatedUserRepository,
    pub user_authentication_request_repository: TUserAuthenticationRequestRepository,
    pub user_info_sync_requested_repository: Arc<dyn UserInfoSyncRequestedRepository + Send + Sync>,
    pub role_sync_requested_repository: Arc<dyn RoleSyncRequestedRepository + Send + Sync>,
    pub invite_link: InviteLink,
}

#[async_trait]
impl<
        TArchivedAuthenticatedUserRepository,
        TAuthenticatedUserRepository,
        TUserAuthenticationRequestRepository,
    > AuthenticationPort
    for AuthenticationService<
        TArchivedAuthenticatedUserRepository,
        TAuthenticatedUserRepository,
        TUserAuthenticationRequestRepository,
    >
where
    TArchivedAuthenticatedUserRepository: ArchivedAuthenticatedUserRepository + Send + Sync,
    TAuthenticatedUserRepository: AuthenticatedUserRepository + Send + Sync,
    TUserAuthenticationRequestRepository: UserAuthenticationRequestRepository + Send + Sync,
{
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
    ) -> Result<(UserId, InviteLink), AuthenticationError> {
        let mut request = match self
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

        if request.is_confirmed() {
            warn!(
                csrf_token = csrf_token.0,
                "The user tried to authenticate with an already confirmed CSRF token",
            );
            return Err(AuthenticationError::AuthenticationRequestAlreadyConfirmed);
        }

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
                email = user.email(),
                "Updating user's info due to a new user authenticating with the same email",
            );

            let user_info_request = request_user_info_sync(user.user_id());
            self.user_info_sync_requested_repository
                .save(&user_info_request)
                .await
                .map_err(map_user_info_sync_req_repo_err)?;
        }

        request.confirm();
        let user = create_user_from_successful_authentication(
            &request,
            user_info.name,
            user_info.email,
            oauth_token,
        );

        self.authenticated_user_repository
            .save(&user)
            .await
            .map_err(map_user_repo_err)?;
        self.user_authentication_request_repository
            .save(&request)
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

        Ok((user.user_id(), self.invite_link.clone()))
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
fn map_user_info_sync_req_repo_err(
    err: UserInfoSyncRequestedRepositoryError,
) -> AuthenticationError {
    match err {
        UserInfoSyncRequestedRepositoryError::ServiceUnavailable => {
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
