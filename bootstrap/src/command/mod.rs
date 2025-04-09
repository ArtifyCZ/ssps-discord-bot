pub mod migrate;
pub mod serve;

use crate::args::CommonArgs;
use crate::command::migrate::MigrateArgs;
use crate::command::serve::ServeArgs;
use anyhow::anyhow;
use clap::Subcommand;
use tracing::instrument;

#[derive(Subcommand)]
pub enum Command {
    #[command(name = "serve")]
    Serve(#[arg(flatten)] ServeArgs),
    #[command(name = "migrate")]
    Migrate(#[arg(flatten)] MigrateArgs),
}

impl Command {
    #[instrument(level = "trace", skip(self, common_args))]
    pub async fn run(self, common_args: CommonArgs) -> anyhow::Result<()> {
        match self {
            Command::Serve(args) => serve::run(common_args, args).await.map_err(|e| anyhow!(e)),
            Command::Migrate(args) => migrate::run(common_args, args).await,
        }
    }
}
