mod role_sync_job;

use crate::application_ports::Locator;
use crate::worker::role_sync_job::run_role_sync_job_handler;
use tracing::instrument;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[instrument(level = "debug", skip(locator))]
pub async fn run_worker<L: Locator + Clone + Send + Sync + 'static>(
    locator: L,
    role_sync_job_wake_channel: tokio::sync::mpsc::Receiver<()>,
) -> Result<(), Error> {
    let role_sync_handle = tokio::spawn(run_role_sync_job_handler(
        locator,
        role_sync_job_wake_channel,
    ));

    role_sync_handle.await?;

    Ok(())
}
