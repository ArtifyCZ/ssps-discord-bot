use async_trait::async_trait;
use domain::authentication::user_authentication_request::{
    UserAuthenticationRequest, UserAuthenticationRequestRepository,
};
use domain_shared::authentication::CsrfToken;
use sqlx::{query, PgPool};
use tracing::instrument;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type Result<T> =
    std::result::Result<T, domain::authentication::user_authentication_request::Error>;

pub struct PostgresUserAuthenticationRequestRepository {
    pool: PgPool,
}

impl PostgresUserAuthenticationRequestRepository {
    #[instrument(level = "trace", skip_all)]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserAuthenticationRequestRepository for PostgresUserAuthenticationRequestRepository {
    #[instrument(level = "debug", err, skip(self, request))]
    async fn save(&self, request: &UserAuthenticationRequest) -> Result<()> {
        let UserAuthenticationRequest {
            csrf_token,
            user_id,
            requested_at,
        } = request;

        // Check if the request already exists
        let exists = query!(
            "SELECT EXISTS(SELECT 1 FROM user_authentication_requests WHERE csrf_token = $1)",
            csrf_token.0,
        )
        .fetch_one(&self.pool)
        .await?
        .exists;

        if let Some(true) = exists {
            query!(
                "UPDATE user_authentication_requests SET user_id = $1, requested_at = $2 WHERE csrf_token = $3",
                user_id.0 as i64,
                requested_at.naive_utc(),
                csrf_token.0,
            ).execute(&self.pool).await?;
        } else {
            query!(
                "INSERT INTO user_authentication_requests (csrf_token, user_id, requested_at) VALUES ($1, $2, $3)",
                csrf_token.0,
                user_id.0 as i64,
                requested_at.naive_utc(),
            ).execute(&self.pool).await?;
        }

        Ok(())
    }

    #[instrument(level = "debug", err, skip(self, csrf_token))]
    async fn find_by_csrf_token(
        &self,
        csrf_token: &CsrfToken,
    ) -> domain::authentication::user_authentication_request::Result<
        Option<UserAuthenticationRequest>,
    > {
        let row = query!(
            "SELECT csrf_token, user_id, requested_at FROM user_authentication_requests WHERE csrf_token = $1",
            csrf_token.0,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(UserAuthenticationRequest {
                csrf_token: CsrfToken(row.csrf_token),
                user_id: domain_shared::discord::UserId(row.user_id as u64),
                requested_at: row.requested_at.and_utc(),
            }))
        } else {
            Ok(None)
        }
    }

    #[instrument(level = "debug", err, skip(self, csrf_token))]
    async fn remove_by_csrf_token(
        &self,
        csrf_token: &CsrfToken,
    ) -> domain::authentication::user_authentication_request::Result<()> {
        query!(
            "DELETE FROM user_authentication_requests WHERE csrf_token = $1",
            csrf_token.0,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
