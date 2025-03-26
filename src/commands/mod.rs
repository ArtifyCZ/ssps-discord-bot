use crate::Data;
use poise::Command;

mod update_information;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn enabled_commands() -> Vec<Command<Data, Error>> {
    vec![update_information::command()]
}
