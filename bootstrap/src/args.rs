use clap::Args;

#[derive(Args)]
pub struct CommonArgs {
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,
}
