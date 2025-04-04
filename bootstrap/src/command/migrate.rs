use crate::args::CommonArgs;
use clap::Args;

#[derive(Args, Debug)]
pub struct MigrateArgs {}

pub async fn run(common_args: CommonArgs, args: MigrateArgs) -> anyhow::Result<()> {
    let CommonArgs { database_url } = common_args;
    let MigrateArgs {} = args;

    let connection = sqlx::PgPool::connect(&database_url).await?;
    infrastructure::database::MIGRATOR.run(&connection).await?;
    Ok(())
}
