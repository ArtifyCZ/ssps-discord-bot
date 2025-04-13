use async_trait::async_trait;
use domain::authentication::authenticated_user::{AuthenticatedUser, AuthenticatedUserRepository};
use domain::ports::oauth::OAuthToken;
use domain_shared::authentication::{AccessToken, RefreshToken};
use domain_shared::discord::UserId;
use sqlx::{query, PgPool};
use tracing::instrument;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type Result<T> = std::result::Result<T, Error>;

pub struct PostgresAuthenticatedUserRepository {
    pool: PgPool,
}

impl PostgresAuthenticatedUserRepository {
    #[instrument(level = "trace", skip_all)]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthenticatedUserRepository for PostgresAuthenticatedUserRepository {
    #[instrument(level = "debug", err, skip(self, user))]
    async fn save(&self, user: AuthenticatedUser) -> Result<()> {
        let AuthenticatedUser {
            user_id,
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
        } = user;

        // Check if the user already exists
        let exists = query!(
            "SELECT EXISTS(SELECT 1 FROM authenticated_users WHERE user_id = $1)",
            user_id.0 as i64,
        )
        .fetch_one(&self.pool)
        .await?
        .exists;
        if let Some(true) = exists {
            query!(
            "UPDATE authenticated_users SET name = $1, email = $2, access_token = $3, access_token_expires_at = $4, refresh_token = $5, class_id = $6, authenticated_at = $7 WHERE user_id = $8",
            name,
            email,
            access_token.0,
            access_token_expires_at.naive_utc(),
            refresh_token.0,
            class_id,
            authenticated_at.naive_utc(),
            user_id.0 as i64,
            ).execute(&self.pool).await?;
        } else {
            query!(
            "INSERT INTO authenticated_users (user_id, name, email, access_token, access_token_expires_at, refresh_token, class_id, authenticated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            user_id.0 as i64,
            name,
            email,
            access_token.0,
            access_token_expires_at.naive_utc(),
            refresh_token.0,
            class_id,
            authenticated_at.naive_utc(),
            ).execute(&self.pool).await?;
        }

        Ok(())
    }

    #[instrument(level = "debug", err, skip(self, user_id))]
    async fn find_by_user_id(&self, user_id: UserId) -> Result<Option<AuthenticatedUser>> {
        let row = query!(
            "SELECT user_id, name, email, access_token, access_token_expires_at, refresh_token, class_id, authenticated_at FROM authenticated_users WHERE user_id = $1",
            user_id.0 as i64,
        ).fetch_optional(&self.pool).await?;

        if let Some(row) = row {
            Ok(Some(AuthenticatedUser {
                user_id: UserId(row.user_id as u64),
                name: row.name,
                email: row.email,
                oauth_token: OAuthToken {
                    access_token: AccessToken(row.access_token),
                    expires_at: row.access_token_expires_at.and_utc(),
                    refresh_token: RefreshToken(row.refresh_token),
                },
                class_id: row.class_id,
                authenticated_at: row.authenticated_at.and_utc(),
            }))
        } else {
            Ok(None)
        }
    }
}
