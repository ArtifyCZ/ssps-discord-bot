use crate::authentication::user_authentication_request::UserAuthenticationRequest;
use crate::ports::oauth::OAuthToken;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain_shared::discord::UserId;
use tracing::instrument;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type Result<T> = std::result::Result<T, Error>;

pub struct AuthenticatedUser {
    user_id: UserId,
    name: String,
    email: String,
    oauth_token: OAuthToken,
    class_id: String,
    authenticated_at: DateTime<Utc>,
}

impl AuthenticatedUser {
    #[instrument(level = "trace", skip(self))]
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    #[instrument(level = "trace", skip(self))]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[instrument(level = "trace", skip(self))]
    pub fn email(&self) -> &str {
        &self.email
    }

    #[instrument(level = "trace", skip(self))]
    pub fn set_user_info(&mut self, name: String, email: String) {
        self.name = name;
        self.email = email;
    }

    #[instrument(level = "trace", skip(self))]
    pub fn oauth_token(&self) -> &OAuthToken {
        &self.oauth_token
    }

    #[instrument(level = "trace", skip(self, oauth_token))]
    pub fn update_oauth_token(&mut self, oauth_token: OAuthToken) {
        self.oauth_token = oauth_token;
    }

    #[instrument(level = "trace", skip(self))]
    pub fn class_id(&self) -> &str {
        &self.class_id
    }

    #[instrument(level = "trace", skip(self))]
    pub fn authenticated_at(&self) -> DateTime<Utc> {
        self.authenticated_at
    }
}

#[instrument(level = "trace", skip(request, oauth_token))]
pub fn create_user_from_successful_authentication(
    request: &UserAuthenticationRequest,
    name: String,
    email: String,
    oauth_token: OAuthToken,
    class_id: String,
) -> AuthenticatedUser {
    AuthenticatedUser {
        user_id: request.user_id(),
        name,
        email,
        oauth_token,
        class_id,
        authenticated_at: Utc::now(),
    }
}

impl AuthenticatedUser {
    #[instrument(level = "trace", skip(snapshot))]
    pub fn from_snapshot(snapshot: AuthenticatedUserSnapshot) -> Self {
        Self {
            user_id: snapshot.user_id,
            name: snapshot.name,
            email: snapshot.email,
            oauth_token: snapshot.oauth_token,
            class_id: snapshot.class_id,
            authenticated_at: snapshot.authenticated_at,
        }
    }

    #[instrument(level = "trace", skip(self))]
    pub fn to_snapshot(&self) -> AuthenticatedUserSnapshot {
        AuthenticatedUserSnapshot {
            user_id: self.user_id,
            name: self.name.clone(),
            email: self.email.clone(),
            oauth_token: self.oauth_token.clone(),
            class_id: self.class_id.clone(),
            authenticated_at: self.authenticated_at,
        }
    }
}

#[derive(Clone)]
pub struct AuthenticatedUserSnapshot {
    pub user_id: UserId,
    pub name: String,
    pub email: String,
    pub oauth_token: OAuthToken,
    pub class_id: String,
    pub authenticated_at: DateTime<Utc>,
}

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait AuthenticatedUserRepository {
    async fn save(&self, user: &AuthenticatedUser) -> Result<()>;
    async fn find_all(&self) -> Result<Vec<AuthenticatedUser>>;
    async fn find_by_user_id(&self, user_id: UserId) -> Result<Option<AuthenticatedUser>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<AuthenticatedUser>>;
}
