use crate::application_ports::Locator;
use crate::discord::{Context, Error};
use domain_shared::discord::UserId;
use poise::serenity_prelude::CreateEmbed;
use poise::CreateReply;
use tracing::{error, info, instrument};

#[poise::command(
    slash_command,
    rename = "non-verified-users-with-role",
    required_permissions = "ADMINISTRATOR"
)]
#[instrument(level = "info", skip(ctx))]
pub async fn command<D: Sync + Locator>(
    ctx: Context<'_, D>,
    #[description = "Force remove roles (default false)"] force_removal: Option<bool>,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .expect("Command should be run only in guilds");
    info!(
        guild_id = guild_id.get(),
        user_id = ctx.author().id.get(),
        "Listing non-verified users with role",
    );
    let force_removal = force_removal.unwrap_or(false);

    ctx.defer().await?;

    let authentication_port = ctx.data().get_authentication_port();

    let main_student_role_id = authentication_port.get_main_student_role().await;

    let users = ctx
        .http()
        .get_guild_members(guild_id, Some(1000), None)
        .await?;
    let members = users
        .into_iter()
        .filter(|user| {
            user.roles
                .iter()
                .any(|role| role.get() == main_student_role_id.0)
        })
        .collect::<Vec<_>>();

    let mut non_verified_users = Vec::with_capacity(members.len());

    let mut error_occurred = false;

    for member in members {
        let user_id = UserId(member.user.id.get());
        let user_info = match authentication_port.get_user_info(user_id, false).await {
            Ok(user_info) => user_info,
            Err(error) => {
                error_occurred = true;
                error!(error = ?error, "Failed to fetch user info");
                continue;
            }
        };
        if user_info.is_none() {
            non_verified_users.push(user_id);
            if force_removal {
                authentication_port
                    .remove_roles_from_non_authenticated_user(user_id)
                    .await?;
            }
        }
    }

    let fields = non_verified_users
        .into_iter()
        .map(|user_id| ("", format!("<@{}>", user_id.0), false));

    let embed = CreateEmbed::default()
        .title("Neověření studenti s rolí")
        .description(format!(
            "Došlo k chybě? {}",
            if error_occurred { "Ano" } else { "Ne" }
        ))
        .fields(fields);

    let reply = CreateReply::default()
        .reply(true)
        .ephemeral(false)
        .embed(embed);
    ctx.send(reply).await?;

    Ok(())
}
