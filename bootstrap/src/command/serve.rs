use crate::locator;
use anyhow::anyhow;
use clap::Args;
use domain_shared::discord::{InviteLink, RoleId};
use infrastructure::oauth::{OAuthAdapter, TenantId};
use oauth2::{ClientId, ClientSecret};
use presentation::api::run_api;
use presentation::discord::run_bot;
use serenity::all::{ClientBuilder, GuildId};
use std::sync::Arc;
use url::Url;

use crate::args::CommonArgs;
use application::authentication::AuthenticationService;
use application::information_channel::InformationChannelService;
use infrastructure::authentication::authenticated_user::PostgresAuthenticatedUserRepository;
use infrastructure::authentication::user_authentication_request::PostgresUserAuthenticationRequestRepository;
use infrastructure::discord::DiscordAdapter;
use poise::serenity_prelude as serenity;
use tracing::instrument;

#[derive(Args)]
pub struct ServeArgs {
    #[arg(long, env = "AUTHENTICATION_CALLBACK_URL")]
    pub authentication_callback_url: String,
    /// The token for the Discord bot
    #[arg(long, env = "DISCORD_BOT_TOKEN")]
    pub discord_bot_token: String,
    /// The ID of the Discord guild (server) for the bot to serve in
    #[arg(long, env = "DISCORD_GUILD_ID")]
    pub guild: u64,
    /// The client ID for the OAuth2 application
    #[arg(long, env = "OAUTH_CLIENT_ID")]
    pub oauth_client_id: String,
    /// The client secret for the OAuth2 application
    #[arg(long, env = "OAUTH_CLIENT_SECRET")]
    pub oauth_client_secret: String,
    /// The tenant ID for the Azure AD application
    #[arg(long, env = "TENANT_ID")]
    pub tenant_id: String,
    /// The invite link for the Discord server
    #[arg(long, env = "INVITE_LINK")]
    pub invite_link: String,
    #[arg(long, env = "ADDITIONAL_STUDENT_ROLES")]
    pub additional_student_roles: String,
}

#[instrument(level = "trace", skip(common_args, args))]
pub async fn run(common_args: CommonArgs, args: ServeArgs) -> anyhow::Result<()> {
    let CommonArgs {
        database_url,
        sentry_dsn: _,
        sentry_environment: _,
        sentry_sample_rate: _,
        sentry_traces_sample_rate: _,
    } = common_args;
    let ServeArgs {
        authentication_callback_url,
        discord_bot_token,
        guild,
        oauth_client_id,
        oauth_client_secret,
        tenant_id,
        invite_link,
        additional_student_roles,
    } = args;
    let guild = GuildId::new(guild);
    let authentication_callback_url = Url::parse(&authentication_callback_url)?;
    let oauth_client_id = ClientId::new(oauth_client_id);
    let oauth_client_secret = ClientSecret::new(oauth_client_secret);
    let tenant_id = TenantId(tenant_id);
    let invite_link = InviteLink(invite_link);
    let additional_student_roles = serde_json::from_str::<Vec<u64>>(&additional_student_roles)?
        .into_iter()
        .map(RoleId)
        .collect();

    let intents = serenity::GatewayIntents::non_privileged();

    let database_connection = sqlx::PgPool::connect(&database_url).await?;
    let serenity_client = ClientBuilder::new(&discord_bot_token, intents).await?.http;

    let discord_adapter = Arc::new(DiscordAdapter::new(serenity_client.clone(), guild));
    let oauth_adapter = Arc::new(OAuthAdapter::new(
        authentication_callback_url,
        oauth_client_id,
        oauth_client_secret,
        tenant_id,
    ));
    let authenticated_user_repository = Arc::new(PostgresAuthenticatedUserRepository::new(
        database_connection.clone(),
    ));
    let user_authentication_request_repository = Arc::new(
        PostgresUserAuthenticationRequestRepository::new(database_connection.clone()),
    );

    let authentication_adapter = Arc::new(AuthenticationService::new(
        discord_adapter.clone(),
        oauth_adapter,
        authenticated_user_repository,
        user_authentication_request_repository,
        invite_link,
        additional_student_roles,
    ));
    let information_channel_adapter =
        Arc::new(InformationChannelService::new(discord_adapter.clone()));

    let locator =
        locator::ApplicationPortLocator::new(authentication_adapter, information_channel_adapter);

    let api = tokio::spawn(run_api(locator.clone(), 8080));
    let bot = tokio::spawn(run_bot(locator, discord_bot_token, intents, guild));

    api.await?.map_err(|e| anyhow!(e))?;
    bot.await?.map_err(|e| anyhow!(e))?;

    Ok(())
}
