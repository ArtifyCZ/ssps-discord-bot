use domain::ports::discord::CreateAttachment;
use poise::serenity_prelude as serenity;
use tracing::instrument;

#[instrument(level = "trace", skip(attachment))]
pub fn domain_to_serenity_create_attachment(
    attachment: CreateAttachment,
) -> serenity::CreateAttachment {
    let CreateAttachment { content, filename } = attachment;

    serenity::CreateAttachment::bytes(content, filename)
}
