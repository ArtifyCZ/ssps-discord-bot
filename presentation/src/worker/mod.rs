mod periodic_scheduling_worker;
mod role_sync_job;
mod user_info_sync_job;

use crate::application_ports::Locator;
use crate::worker::periodic_scheduling_worker::run_periodic_scheduling_worker;
use crate::worker::role_sync_job::run_role_sync_job_handler;
use crate::worker::user_info_sync_job::run_user_info_sync_job_handler;
use tracing::instrument;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[instrument(level = "debug", skip(locator))]
pub async fn run_worker<L: Locator + Clone + Send + Sync + 'static>(
    locator: L,
    role_sync_job_wake_channel: tokio::sync::mpsc::Receiver<()>,
    user_info_sync_job_wake_channel: tokio::sync::mpsc::Receiver<()>,
) -> Result<(), Error> {
    let role_sync_handle = tokio::spawn(run_role_sync_job_handler(
        locator.clone(),
        role_sync_job_wake_channel,
    ));
    let user_info_sync_handle = tokio::spawn(run_user_info_sync_job_handler(
        locator.clone(),
        user_info_sync_job_wake_channel,
    ));
    let periodic_scheduling_handle = tokio::spawn(run_periodic_scheduling_worker(locator.clone()));

    role_sync_handle.await?;
    user_info_sync_handle.await?;
    periodic_scheduling_handle.await?;

    Ok(())
}
