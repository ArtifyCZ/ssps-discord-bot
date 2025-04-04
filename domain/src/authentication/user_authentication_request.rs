use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain_shared::authentication::CsrfToken;
use domain_shared::discord::UserId;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type Result<T> = std::result::Result<T, Error>;

pub struct UserAuthenticationRequest {
    pub csrf_token: CsrfToken,
    pub user_id: UserId,
    pub requested_at: DateTime<Utc>,
}

#[async_trait]
pub trait UserAuthenticationRequestRepository {
    async fn save(&self, request: UserAuthenticationRequest) -> Result<()>;
    async fn find_by_csrf_token(
        &self,
        csrf_token: CsrfToken,
    ) -> Result<Option<UserAuthenticationRequest>>;
    async fn remove(&self, request: UserAuthenticationRequest) -> Result<()>;
}
