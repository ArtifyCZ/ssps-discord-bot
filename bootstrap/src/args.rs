use clap::Args;

#[derive(Args, Debug)]
pub struct CommonArgs {
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,
}
