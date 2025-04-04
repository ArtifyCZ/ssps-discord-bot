use crate::application_ports::Locator;
use crate::discord::Error;
use poise::Command;

pub mod update_information;

pub fn enabled_commands<L: Locator + Send + Sync + 'static>() -> Vec<Command<L, Error>> {
    vec![update_information::command()]
}
