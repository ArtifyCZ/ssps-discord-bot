use crate::application_ports::Locator;
use crate::discord::Error;
use poise::Command;
use tracing::instrument;

pub mod update_information;
pub mod verify;

#[instrument(level = "trace", skip())]
pub fn enabled_commands<L: Locator + Send + Sync + 'static>() -> Vec<Command<L, Error>> {
    vec![update_information::command(), verify::command()]
}
