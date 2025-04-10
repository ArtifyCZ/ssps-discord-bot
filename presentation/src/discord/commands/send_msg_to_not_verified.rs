use crate::application_ports::Locator;
use crate::discord::{Context, Error};
use domain_shared::discord::UserId;
use poise::serenity_prelude::CreateMessage;
use poise::{serenity_prelude as serenity, CreateReply};
use tracing::{info, instrument, warn};

#[poise::command(
    slash_command,
    rename = "send-msg-to-not-verified-users",
    required_permissions = "ADMINISTRATOR"
)]
#[instrument(level = "info", skip(ctx))]
pub async fn command<D: Sync + Locator>(
    ctx: Context<'_, D>,
    #[description = "Selected role"] target: serenity::Role,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    info!(
        guild_id = guild_id.get(),
        user_id = ctx.author().id.get(),
        "Accessing verified users",
    );

    let authentication_port = ctx.data().get_authentication_port();
    let members = target.guild_id.members(ctx.http(), None, None).await?;
    let target_members = members
        .into_iter()
        .filter(|member| member.roles.contains(&target.id))
        .map(|member| UserId(member.user.id.get()))
        .collect::<Vec<_>>();
    let result = match authentication_port
        .get_not_verified_users(target_members)
        .await
    {
        Ok(result) => result,
        Err(error) => {
            warn!(error = ?error, "Error during getting not verified users");
            return Err("Failed to get not verified users".into());
        }
    };
    for user_id in result.iter() {
        let user = serenity::UserId::new(user_id.0).to_user(ctx.http()).await?;
        user
            .dm(ctx.http(), CreateMessage::default()
                .content("Zdravím. Chtěl bych Vás znovu požádat o ověření Vašeho účtu. Bohužel došlo k technickým problémům, v jejichž důsledku se ověření nezdařilo. Omlouváme se za komplikace a děkujeme za pochopení.")
            ).await?;
        ctx.http()
            .remove_member_role(
                guild_id,
                user.id,
                target.id,
                Some("Lost OAuth2 Azure AD authentication data"),
            )
            .await?;
    }

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Message sent to: {}",
                result
                    .iter()
                    .map(|user_id| format!("<@{}>", user_id.0))
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
            .ephemeral(true)
            .reply(true),
    )
    .await?;

    Ok(())
}
