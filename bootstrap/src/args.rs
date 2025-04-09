use clap::Args;

#[derive(Args)]
pub struct CommonArgs {
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,
    #[arg(long, env = "SENTRY_DSN")]
    pub sentry_dsn: Option<String>,
    #[arg(long, env = "SENTRY_ENVIRONMENT")]
    pub sentry_environment: Option<String>,
    #[arg(long, env = "SENTRY_SAMPLE_RATE")]
    pub sentry_sample_rate: Option<f32>,
    #[arg(long, env = "SENTRY_TRACES_SAMPLE_RATE")]
    pub sentry_traces_sample_rate: Option<f32>,
}
