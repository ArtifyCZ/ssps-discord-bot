use crate::ports::oauth::OAuthToken;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain_shared::discord::UserId;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type Result<T> = std::result::Result<T, Error>;

pub struct AuthenticatedUser {
    pub user_id: UserId,
    pub name: Option<String>,
    pub email: Option<String>,
    pub oauth_token: OAuthToken,
    pub class_id: String,
    pub authenticated_at: DateTime<Utc>,
}

#[async_trait]
pub trait AuthenticatedUserRepository {
    async fn save(&self, user: &AuthenticatedUser) -> Result<()>;
    async fn find_by_user_id(&self, user_id: UserId) -> Result<Option<AuthenticatedUser>>;
}
