use async_trait::async_trait;
use domain::jobs::role_sync_job::{
    RoleSyncRequested, RoleSyncRequestedRepository, RoleSyncRequestedRepositoryError,
};
use domain_shared::discord::UserId;
use sqlx::{query, PgPool};
use tokio::sync::mpsc;
use tracing::{instrument, warn};

pub struct PostgresRoleSyncRequestedRepository<'a> {
    pool: &'a PgPool,
    role_sync_job_wake_tx: &'a mpsc::Sender<()>,
}

impl<'a> PostgresRoleSyncRequestedRepository<'a> {
    #[instrument(level = "trace", skip_all)]
    pub fn new(pool: &'a PgPool, role_sync_job_wake_tx: &'a mpsc::Sender<()>) -> Self {
        Self {
            pool,
            role_sync_job_wake_tx,
        }
    }
}

#[async_trait]
impl<'a> RoleSyncRequestedRepository for PostgresRoleSyncRequestedRepository<'a> {
    #[instrument(level = "debug", err, skip_all)]
    async fn save(
        &self,
        request: &RoleSyncRequested,
    ) -> Result<(), RoleSyncRequestedRepositoryError> {
        if request.low_priority {
            query!(
                "INSERT INTO role_sync_requested (user_id, queued_at, low_priority) VALUES ($1, $2, true) ON CONFLICT (user_id) DO NOTHING",
                request.user_id.0 as i64,
                request.queued_at.naive_utc(),
            )
        } else {
            query!(
                "INSERT INTO role_sync_requested (user_id, queued_at, low_priority) VALUES ($1, $2, false) ON CONFLICT (user_id) DO UPDATE SET queued_at = $2, low_priority = false",
                request.user_id.0 as i64,
                request.queued_at.naive_utc(),
            )
        }.execute(self.pool).await.map_err(|err| {
            warn!(error = ?err, "Failed to save role sync request");
            RoleSyncRequestedRepositoryError::ServiceUnavailable
        })?;

        // Wake up the role sync job handler
        self.role_sync_job_wake_tx.try_send(()).ok();

        Ok(())
    }

    #[instrument(level = "debug", err, skip_all)]
    async fn pop_oldest(
        &self,
        low_priority: bool,
    ) -> Result<Option<RoleSyncRequested>, RoleSyncRequestedRepositoryError> {
        let row =
            query!(
                "SELECT user_id, queued_at, low_priority FROM role_sync_requested WHERE low_priority = $1 ORDER BY queued_at LIMIT 1",
                low_priority,
            )
                .fetch_optional(self.pool)
                .await
                .map_err(|err| {
                    warn!(error = ?err, "Failed to fetch oldest role sync request");
                    RoleSyncRequestedRepositoryError::ServiceUnavailable
                })?;

        if let Some(ref row) = row {
            query!(
                "DELETE FROM role_sync_requested WHERE user_id = $1 AND low_priority = $2",
                row.user_id as i64,
                row.low_priority,
            )
            .execute(self.pool)
            .await
            .map_err(|err| {
                warn!(error = ?err, "Failed to pop oldest role sync request");
                RoleSyncRequestedRepositoryError::ServiceUnavailable
            })?;
        }

        Ok(row.map(|row| RoleSyncRequested {
            user_id: UserId(row.user_id as u64),
            queued_at: row.queued_at.and_utc(),
            low_priority,
        }))
    }
}
