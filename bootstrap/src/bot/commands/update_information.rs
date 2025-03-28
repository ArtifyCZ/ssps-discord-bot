use crate::resources;
use crate::{Context, Error};
use serenity::all::{ChannelId, CreateAttachment, CreateMessage, Http};
use serenity::futures::StreamExt;

#[poise::command(
    slash_command,
    rename = "update-information",
    required_permissions = "ADMINISTRATOR"
)]
pub async fn command(ctx: Context<'_>) -> Result<(), Error> {
    let channel = ctx.channel_id();

    ctx.defer().await?;

    purge_messages(ctx.http(), channel).await?;

    let messages = vec![
        CreateMessage::default().add_file(CreateAttachment::bytes(
            resources::SSPS_BANNER_PNG,
            "ssps_banner.png",
        )),
        CreateMessage::default().content("# Web: <https://ssps.cz/>"),
        CreateMessage::default().content(resources::SCHOOL_MANAGEMENT_MD),
        CreateMessage::default().content(resources::SOCIAL_NETWORKS_MD),
        CreateMessage::default().add_file(CreateAttachment::bytes(
            resources::SSPS_ON_MAP_PNG,
            "ssps_on_map.png",
        )),
        CreateMessage::default().content(resources::CONTACTS_MD),
        CreateMessage::default().content(resources::RULES_MD),
        CreateMessage::default().content(resources::ANNOUNCEMENT_GUIDELINES_MD),
    ];

    for message in messages {
        channel.send_message(ctx.http(), message).await?;
    }

    Ok(())
}

async fn purge_messages(http: &Http, channel_id: ChannelId) -> Result<(), Error> {
    let mut messages = channel_id.messages_iter(http).boxed();
    while let Some(message) = messages.next().await {
        let message = message?;
        message.delete(http).await?;
    }
    Ok(())
}
