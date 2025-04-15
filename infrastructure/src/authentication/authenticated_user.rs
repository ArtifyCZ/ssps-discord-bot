use async_trait::async_trait;
use domain::authentication::authenticated_user::{
    AuthenticatedUser, AuthenticatedUserRepository, AuthenticatedUserSnapshot,
};
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

macro_rules! record_to_user {
    ($record:ident) => {
        AuthenticatedUser::from_snapshot(AuthenticatedUserSnapshot {
            user_id: UserId($record.user_id as u64),
            name: $record.name,
            email: $record.email,
            oauth_token: OAuthToken {
                access_token: AccessToken($record.access_token),
                expires_at: $record.access_token_expires_at.and_utc(),
                refresh_token: RefreshToken($record.refresh_token),
            },
            class_id: $record.class_id,
            authenticated_at: $record.authenticated_at.and_utc(),
        })
    };
}

#[async_trait]
impl AuthenticatedUserRepository for PostgresAuthenticatedUserRepository {
    #[instrument(level = "debug", err, skip(self, user))]
    async fn save(&self, user: &AuthenticatedUser) -> Result<()> {
        let AuthenticatedUserSnapshot {
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
        } = user.to_snapshot();

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
            name.clone(),
            email.clone(),
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
            name.clone(),
            email.clone(),
            access_token.0,
            access_token_expires_at.naive_utc(),
            refresh_token.0,
            class_id,
            authenticated_at.naive_utc(),
            ).execute(&self.pool).await?;
        }

        Ok(())
    }

    #[instrument(level = "debug", err, skip(self))]
    async fn find_all(&self) -> Result<Vec<AuthenticatedUser>> {
        let rows = query!(
            "SELECT user_id, name, email, access_token, access_token_expires_at, refresh_token, class_id, authenticated_at FROM authenticated_users",
        ).fetch_all(&self.pool).await?;
        let users = rows.into_iter().map(|row| record_to_user!(row)).collect();
        Ok(users)
    }

    #[instrument(level = "debug", err, skip(self, user_id))]
    async fn find_by_user_id(&self, user_id: UserId) -> Result<Option<AuthenticatedUser>> {
        let row = query!(
            "SELECT user_id, name, email, access_token, access_token_expires_at, refresh_token, class_id, authenticated_at FROM authenticated_users WHERE user_id = $1",
            user_id.0 as i64,
        ).fetch_optional(&self.pool).await?;

        if let Some(row) = row {
            Ok(Some(record_to_user!(row)))
        } else {
            Ok(None)
        }
    }

    #[instrument(level = "debug", err, skip(self, email))]
    async fn find_by_email(&self, email: &str) -> Result<Option<AuthenticatedUser>> {
        let row = query!(
            "SELECT user_id, name, email, access_token, access_token_expires_at, refresh_token, class_id, authenticated_at FROM authenticated_users WHERE email = $1",
            email,
        ).fetch_optional(&self.pool).await?;

        if let Some(row) = row {
            Ok(Some(record_to_user!(row)))
        } else {
            Ok(None)
        }
    }
}
