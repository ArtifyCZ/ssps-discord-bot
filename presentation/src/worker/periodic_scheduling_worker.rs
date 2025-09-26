use crate::application_ports::Locator;
use application_ports::periodic_scheduling_handler::{
    PeriodicSchedulingHandlerError, PeriodicSchedulingHandlerPort,
};
use std::time::Duration;
use tracing::{error, instrument, warn};

#[instrument(level = "debug", skip(locator))]
pub async fn run_periodic_scheduling_worker<L: Locator + Send + Sync + 'static>(locator: L) {
    let mut handler = locator.create_periodic_scheduling_handler_port();
    let mut unavailable_sleep_duration: Option<Duration> = None;
    loop {
        tokio::time::sleep(Duration::from_millis(3000)).await;

        let error = match handler.tick().await {
            Ok(()) => {
                unavailable_sleep_duration = None;
                continue;
            }
            Err(error) => error,
        };

        match error {
            PeriodicSchedulingHandlerError::TemporarilyUnavailable => {
                let mut duration = unavailable_sleep_duration
                    .map(|d| d * 2)
                    .unwrap_or(Duration::from_secs(3 * 60));
                if duration >= Duration::from_secs(60 * 60) {
                    duration = Duration::from_secs(30 * 60);
                    error!("Periodic scheduling worker is temporarily unavailable. Sleeping for one hour");
                }
                if duration >= Duration::from_secs(30 * 60) {
                    error!(
                        "Periodic scheduling worker is temporarily unavailable. Sleeping for {} seconds",
                        duration.as_secs(),
                    );
                }
                unavailable_sleep_duration = Some(duration);
                warn!(
                    "Periodic scheduling worker is temporarily unavailable. Sleeping for {} seconds",
                    duration.as_secs(),
                );
                tokio::time::sleep(duration).await;
            }
        }
    }
}
