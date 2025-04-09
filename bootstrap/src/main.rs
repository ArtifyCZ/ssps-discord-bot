pub mod args;
pub mod command;
pub mod locator;

use crate::args::CommonArgs;
use crate::command::Command;
use clap::Parser;
use sentry::types::Dsn;
use std::borrow::Cow;
use std::str::FromStr;
use tracing::instrument;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

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
        let _guard = self
            .args
            .sentry_dsn
            .as_ref()
            .map(|dsn| Dsn::from_str(dsn).expect("Invalid Sentry DSN"))
            .map(|sentry_dsn| {
                sentry::init(sentry::ClientOptions {
                    dsn: Some(sentry_dsn),
                    environment: self.args.sentry_environment.clone().map(Cow::from),
                    release: sentry::release_name!(),
                    sample_rate: self.args.sentry_sample_rate.unwrap_or(0.0),
                    traces_sample_rate: self.args.sentry_traces_sample_rate.unwrap_or(0.0),
                    ..Default::default()
                })
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
