use async_trait::async_trait;
use domain::authentication::archived_authenticated_user::{
    ArchivedAuthenticatedUser, ArchivedAuthenticatedUserRepository,
    ArchivedAuthenticatedUserSnapshot,
};
use domain::ports::oauth::OAuthToken;
use domain_shared::authentication::ArchivedUserId;
use sqlx::{query, PgPool};
use tracing::instrument;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type Result<T> = std::result::Result<T, Error>;

pub struct PostgresArchivedAuthenticatedUserRepository {
    pool: PgPool,
}

impl PostgresArchivedAuthenticatedUserRepository {
    #[instrument(level = "trace", skip_all)]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ArchivedAuthenticatedUserRepository for PostgresArchivedAuthenticatedUserRepository {
    #[instrument(level = "debug", err, skip(self, user))]
    async fn save(&self, user: &ArchivedAuthenticatedUser) -> Result<()> {
        let ArchivedAuthenticatedUserSnapshot {
            archived_user_id: ArchivedUserId(user_id, archived_at),
            name,
            email,
            oauth_token:
                OAuthToken {
                    access_token,
                    expires_at: access_token_expires_at,
                    refresh_token,
                },
            class_id,
            authenticated_at,
        } = user.to_snapshot();

        query!(
            "INSERT INTO archived_authenticated_users (user_id, archived_at, name, email, access_token, access_token_expires_at, refresh_token, class_id, authenticated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            user_id.0 as i64,
            archived_at.naive_utc(),
            name.clone(),
            email.clone(),
            access_token.0,
            access_token_expires_at.naive_utc(),
            refresh_token.0,
            class_id,
            authenticated_at.naive_utc(),
        ).execute(&self.pool).await?;

        Ok(())
    }
}
