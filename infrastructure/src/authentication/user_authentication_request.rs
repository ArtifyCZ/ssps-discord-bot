use async_trait::async_trait;
use domain::authentication::user_authentication_request::{
    UserAuthenticationRequest, UserAuthenticationRequestRepository,
    UserAuthenticationRequestRepositoryError, UserAuthenticationRequestSnapshot,
};
use domain_shared::authentication::CsrfToken;
use sqlx::{query, PgPool};
use tracing::{instrument, warn};

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
    async fn save(
        &self,
        request: &UserAuthenticationRequest,
    ) -> Result<(), UserAuthenticationRequestRepositoryError> {
        let UserAuthenticationRequestSnapshot {
            csrf_token,
            user_id,
            requested_at,
        } = request.to_snapshot();

        query!(
            "INSERT INTO user_authentication_requests (csrf_token, user_id, requested_at) VALUES ($1, $2, $3)
            ON CONFLICT (csrf_token) DO UPDATE SET user_id = $2, requested_at = $3",
            csrf_token.0,
            user_id.0 as i64,
            requested_at.naive_utc(),
        ).execute(&self.pool).await.map_err(map_err)?;

        Ok(())
    }

    #[instrument(level = "debug", err, skip(self, csrf_token))]
    async fn find_by_csrf_token(
        &self,
        csrf_token: &CsrfToken,
    ) -> Result<Option<UserAuthenticationRequest>, UserAuthenticationRequestRepositoryError> {
        let row = query!(
            "SELECT csrf_token, user_id, requested_at FROM user_authentication_requests WHERE csrf_token = $1",
            csrf_token.0,
        )
        .fetch_optional(&self.pool)
        .await.map_err(map_err)?;

        if let Some(row) = row {
            Ok(Some(UserAuthenticationRequest::from_snapshot(
                UserAuthenticationRequestSnapshot {
                    csrf_token: CsrfToken(row.csrf_token),
                    user_id: domain_shared::discord::UserId(row.user_id as u64),
                    requested_at: row.requested_at.and_utc(),
                },
            )))
        } else {
            Ok(None)
        }
    }

    #[instrument(level = "debug", err, skip(self, csrf_token))]
    async fn remove_by_csrf_token(
        &self,
        csrf_token: &CsrfToken,
    ) -> Result<(), UserAuthenticationRequestRepositoryError> {
        query!(
            "DELETE FROM user_authentication_requests WHERE csrf_token = $1",
            csrf_token.0,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(())
    }
}

#[instrument(level = "trace", skip_all)]
fn map_err(err: sqlx::Error) -> UserAuthenticationRequestRepositoryError {
    warn!(
        error = ?err,
        "Failed to fetch authenticated user",
    );
    UserAuthenticationRequestRepositoryError::TemporaryUnavailable
}
