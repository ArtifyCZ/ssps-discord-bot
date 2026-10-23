pub mod args;
pub mod command;
pub mod locator;

use crate::args::CommonArgs;
use crate::command::Command;
use clap::Parser;
use tracing::instrument;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser)]
pub struct Cli {
    #[command(flatten)]
    args: CommonArgs,
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    #[instrument(level = "trace", skip(self))]
    pub async fn run(self) -> anyhow::Result<()> {
        let _guard = self.args.sentry_dsn.as_ref().map(|sentry_dsn| {
            sentry::init(
                sentry::ClientOptions::new()
                    .dsn(sentry_dsn)
                    .environment(
                        self.args
                            .sentry_environment
                            .clone()
                            .unwrap_or("Unknown".to_string()),
                    )
                    .maybe_release(sentry::release_name!())
                    .sample_rate(self.args.sentry_sample_rate.unwrap_or(0.0))
                    .traces_sample_rate(self.args.sentry_traces_sample_rate.unwrap_or(0.0)),
            )
        });

        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_line_number(true)
                    .with_file(true),
            )
            .with(sentry_tracing::layer())
            .init();

        self.command.run(self.args).await
    }
}

#[instrument(level = "trace", skip())]
#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    cli.run().await
}
