pub mod args;
pub mod command;
pub mod locator;

use crate::args::CommonArgs;
use crate::command::Command;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(flatten)]
    args: CommonArgs,
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub async fn run(self) -> anyhow::Result<()> {
        self.command.run(self.args).await
    }
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    cli.run().await
}
