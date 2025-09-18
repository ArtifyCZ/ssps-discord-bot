use crate::authentication::authenticated_user::AuthenticatedUser;
use crate::ports::oauth::OAuthToken;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain_shared::authentication::ArchivedUserId;
use domain_shared::discord::UserId;
use thiserror::Error;
use tracing::instrument;

pub struct ArchivedAuthenticatedUser {
    archived_user_id: ArchivedUserId,
    name: String,
    email: String,
    oauth_token: OAuthToken,
    class_id: String,
    authenticated_at: DateTime<Utc>,
}

impl ArchivedAuthenticatedUser {
    #[instrument(level = "trace", skip(self))]
    pub fn archived_user_id(&self) -> ArchivedUserId {
        self.archived_user_id
    }

    #[instrument(level = "trace", skip(self))]
    pub fn user_id(&self) -> UserId {
        self.archived_user_id.0
    }

    #[instrument(level = "trace", skip(self))]
    pub fn archived_at(&self) -> DateTime<Utc> {
        self.archived_user_id.1
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
    pub fn oauth_token(&self) -> &OAuthToken {
        &self.oauth_token
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

#[instrument(level = "trace")]
pub fn create_archived_authenticated_user_from_user(
    user: &AuthenticatedUser,
) -> ArchivedAuthenticatedUser {
    let user_id = user.user_id();
    let archived_at = Utc::now();

    let archived_user_id = ArchivedUserId(user_id, archived_at);
    let name = user.name().to_string();
    let email = user.email().to_string();
    let oauth_token = user.oauth_token().clone();
    let class_id = user.class_id().to_string();
    let authenticated_at = user.authenticated_at();

    ArchivedAuthenticatedUser {
        archived_user_id,
        name,
        email,
        oauth_token,
        class_id,
        authenticated_at,
    }
}

impl ArchivedAuthenticatedUser {
    #[instrument(level = "trace", skip(snapshot))]
    pub fn from_snapshot(snapshot: ArchivedAuthenticatedUserSnapshot) -> Self {
        Self {
            archived_user_id: snapshot.archived_user_id,
            name: snapshot.name,
            email: snapshot.email,
            oauth_token: snapshot.oauth_token,
            class_id: snapshot.class_id,
            authenticated_at: snapshot.authenticated_at,
        }
    }

    #[instrument(level = "trace", skip(self))]
    pub fn to_snapshot(&self) -> ArchivedAuthenticatedUserSnapshot {
        ArchivedAuthenticatedUserSnapshot {
            archived_user_id: self.archived_user_id,
            name: self.name.clone(),
            email: self.email.clone(),
            oauth_token: self.oauth_token.clone(),
            class_id: self.class_id.clone(),
            authenticated_at: self.authenticated_at,
        }
    }
}

#[derive(Clone)]
pub struct ArchivedAuthenticatedUserSnapshot {
    pub archived_user_id: ArchivedUserId,
    pub name: String,
    pub email: String,
    pub oauth_token: OAuthToken,
    pub class_id: String,
    pub authenticated_at: DateTime<Utc>,
}

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait ArchivedAuthenticatedUserRepository {
    async fn save(
        &self,
        user: &ArchivedAuthenticatedUser,
    ) -> Result<(), ArchivedAuthenticatedUserRepositoryError>;
}

#[derive(Debug, Error)]
pub enum ArchivedAuthenticatedUserRepositoryError {
    #[error("Service is temporarily unavailable")]
    ServiceUnavailable,
}
