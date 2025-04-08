use application_ports::authentication::{AuthenticationError, AuthenticationPort};
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
use domain_shared::discord::UserId;
use std::sync::Arc;
use tracing::{instrument, Span};

pub struct AuthenticationService {
    discord_port: Arc<dyn DiscordPort + Send + Sync>,
    oauth_port: Arc<dyn OAuthPort + Send + Sync>,
    authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
    user_authentication_request_repository:
        Arc<dyn UserAuthenticationRequestRepository + Send + Sync>,
    invite_link: InviteLink,
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
    ) -> Self {
        Self {
            discord_port,
            oauth_port,
            authenticated_user_repository,
            user_authentication_request_repository,
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
        if let Some(_user) = self
            .authenticated_user_repository
            .find_by_user_id(user_id)
            .await?
        {
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
            .find_by_csrf_token(csrf_token)
            .await?
        {
            Some(request) => request,
            None => {
                todo!();
            }
        };
        Span::current().record("user_id", request.user_id.0);

        let (access_token, refresh_token) = self
            .oauth_port
            .exchange_code_after_callback(client_callback_token)
            .await?;
        let groups = self
            .oauth_port
            .get_user_groups(access_token.clone())
            .await?;
        let class_group = find_class_group(&groups)
            .ok_or_else(|| AuthenticationError::Error("User is not in the Class group".into()))?;
        let class_id = get_class_id(class_group)
            .ok_or_else(|| AuthenticationError::Error("User's class group ID not found".into()))?;

        let user = AuthenticatedUser {
            user_id: request.user_id,
            access_token,
            refresh_token,
            class_id: class_id.clone(),
            authenticated_at: Utc::now(),
        };

        let user_id = request.user_id;

        self.authenticated_user_repository.save(user).await?;
        self.user_authentication_request_repository
            .remove(request)
            .await?;

        self.discord_port
            .assign_user_to_class_role(user_id, class_id)
            .await?;

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
