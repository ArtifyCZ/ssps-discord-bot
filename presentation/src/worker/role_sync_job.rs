use crate::application_ports::Locator;
use application_ports::role_sync_job_handler::RoleSyncJobHandlerError;
use std::time::Duration;
use tracing::{error, instrument, warn};

#[instrument(level = "debug", skip(locator))]
pub async fn run_role_sync_job_handler<L: Locator + Send + Sync + 'static>(
    locator: L,
    mut wake_channel: tokio::sync::mpsc::Receiver<()>,
) {
    let handler = locator.get_role_sync_job_handler_port();
    let mut unavailable_sleep_duration: Option<Duration> = None;
    loop {
        loop {
            if let Ok(()) = wake_channel.try_recv() {
                continue;
            }

            break;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;

        let error = match handler.tick().await {
            Ok(()) => {
                unavailable_sleep_duration = None;
                continue;
            }
            Err(error) => error,
        };

        match error {
            RoleSyncJobHandlerError::NoRequestToHandle => {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(3)) => {
                        continue;
                    }
                    _ = wake_channel.recv() => {
                        continue;
                    }
                }
            }
            RoleSyncJobHandlerError::TemporaryUnavailable => {
                let duration = unavailable_sleep_duration
                    .map(|d| d * 2)
                    .unwrap_or(Duration::from_secs(3));
                if duration >= Duration::from_secs(90) {
                    error!(
                        "Role sync job handler temporarily unavailable. Sleeping for {} seconds.",
                        duration.as_secs(),
                    );
                }
                unavailable_sleep_duration = Some(duration);
                warn!(
                    "Role sync job handler temporarily unavailable. Sleeping for {} seconds.",
                    duration.as_secs(),
                );
                tokio::time::sleep(duration).await;
            }
        }
    }
}
