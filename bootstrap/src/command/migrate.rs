use crate::args::CommonArgs;
use clap::Args;
use tracing::instrument;

#[derive(Args, Debug)]
pub struct MigrateArgs {}

#[instrument(level = "info", skip(common_args, args))]
pub async fn run(common_args: CommonArgs, args: MigrateArgs) -> anyhow::Result<()> {
    let CommonArgs { database_url } = common_args;
    let MigrateArgs {} = args;

    let connection = sqlx::PgPool::connect(&database_url).await?;
    infrastructure::database::MIGRATOR.run(&connection).await?;
    Ok(())
}
