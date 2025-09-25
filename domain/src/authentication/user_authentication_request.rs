use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain_shared::authentication::CsrfToken;
use domain_shared::discord::UserId;
use thiserror::Error;
use tracing::instrument;

pub struct UserAuthenticationRequest {
    csrf_token: CsrfToken,
    user_id: UserId,
    requested_at: DateTime<Utc>,
    confirmed_at: Option<DateTime<Utc>>,
}

#[instrument(level = "trace", skip(csrf_token))]
pub fn create_user_authentication_request(
    csrf_token: CsrfToken,
    user_id: UserId,
) -> UserAuthenticationRequest {
    UserAuthenticationRequest {
        csrf_token,
        user_id,
        requested_at: Utc::now(),
        confirmed_at: None,
    }
}

impl UserAuthenticationRequest {
    #[instrument(level = "trace", skip(self))]
    pub fn csrf_token(&self) -> &CsrfToken {
        &self.csrf_token
    }

    #[instrument(level = "trace", skip(self))]
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    #[instrument(level = "trace", skip(self))]
    pub fn requested_at(&self) -> DateTime<Utc> {
        self.requested_at
    }

    #[instrument(level = "trace", skip(self))]
    pub fn confirmed_at(&self) -> Option<DateTime<Utc>> {
        self.confirmed_at
    }

    #[instrument(level = "trace", skip(self))]
    pub fn is_confirmed(&self) -> bool {
        self.confirmed_at.is_some()
    }

    #[instrument(level = "trace", skip(self))]
    pub fn confirm(&mut self) {
        self.confirmed_at = Some(Utc::now());
    }
}

impl UserAuthenticationRequest {
    #[instrument(level = "trace", skip(snapshot))]
    pub fn from_snapshot(snapshot: UserAuthenticationRequestSnapshot) -> Self {
        Self {
            csrf_token: snapshot.csrf_token,
            user_id: snapshot.user_id,
            requested_at: snapshot.requested_at,
            confirmed_at: snapshot.confirmed_at,
        }
    }

    #[instrument(level = "trace", skip(self))]
    pub fn to_snapshot(&self) -> UserAuthenticationRequestSnapshot {
        UserAuthenticationRequestSnapshot {
            csrf_token: self.csrf_token.clone(),
            user_id: self.user_id,
            requested_at: self.requested_at,
            confirmed_at: self.confirmed_at,
        }
    }
}

#[derive(Clone)]
pub struct UserAuthenticationRequestSnapshot {
    pub csrf_token: CsrfToken,
    pub user_id: UserId,
    pub requested_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait UserAuthenticationRequestRepository {
    async fn save(
        &self,
        request: &UserAuthenticationRequest,
    ) -> Result<(), UserAuthenticationRequestRepositoryError>;
    async fn find_by_csrf_token(
        &self,
        csrf_token: &CsrfToken,
    ) -> Result<Option<UserAuthenticationRequest>, UserAuthenticationRequestRepositoryError>;
}

#[derive(Debug, Error)]
pub enum UserAuthenticationRequestRepositoryError {
    #[error("Service is temporarily unavailable")]
    TemporaryUnavailable,
}
