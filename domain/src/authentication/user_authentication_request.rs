use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain_shared::authentication::CsrfToken;
use domain_shared::discord::UserId;
use tracing::instrument;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type Result<T> = std::result::Result<T, Error>;

pub struct UserAuthenticationRequest {
    csrf_token: CsrfToken,
    user_id: UserId,
    requested_at: DateTime<Utc>,
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
}

impl UserAuthenticationRequest {
    #[instrument(level = "trace", skip(snapshot))]
    pub fn from_snapshot(snapshot: UserAuthenticationRequestSnapshot) -> Self {
        Self {
            csrf_token: snapshot.csrf_token,
            user_id: snapshot.user_id,
            requested_at: snapshot.requested_at,
        }
    }

    #[instrument(level = "trace", skip(self))]
    pub fn to_snapshot(&self) -> UserAuthenticationRequestSnapshot {
        UserAuthenticationRequestSnapshot {
            csrf_token: self.csrf_token.clone(),
            user_id: self.user_id,
            requested_at: self.requested_at,
        }
    }
}

pub struct UserAuthenticationRequestSnapshot {
    pub csrf_token: CsrfToken,
    pub user_id: UserId,
    pub requested_at: DateTime<Utc>,
}

#[async_trait]
pub trait UserAuthenticationRequestRepository {
    async fn save(&self, request: &UserAuthenticationRequest) -> Result<()>;
    async fn find_by_csrf_token(
        &self,
        csrf_token: &CsrfToken,
    ) -> Result<Option<UserAuthenticationRequest>>;
    async fn remove_by_csrf_token(&self, csrf_token: &CsrfToken) -> Result<()>;
}
