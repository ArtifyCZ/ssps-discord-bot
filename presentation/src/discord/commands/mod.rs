use crate::application_ports::Locator;
use crate::discord::Error;
use poise::Command;
use tracing::instrument;

mod not_verified_users;
mod send_msg_to_not_verified;
pub mod update_information;
pub mod verify;

#[instrument(level = "trace", skip())]
pub fn enabled_commands<L: Locator + Send + Sync + 'static>() -> Vec<Command<L, Error>> {
    vec![
        not_verified_users::command(),
        send_msg_to_not_verified::command(),
        update_information::command(),
        verify::command(),
    ]
}
