use crate::discord::create_attachment::domain_to_serenity_create_attachment;
use domain::ports::discord::CreateMessage;
use poise::serenity_prelude as serenity;

pub fn domain_to_serenity_create_message(message: CreateMessage) -> serenity::CreateMessage {
    let CreateMessage {
        content,
        attachments,
    } = message;

    let mut message = serenity::CreateMessage::default();

    if let Some(content) = content {
        message = message.content(content);
    }

    for attachment in attachments {
        let attachment = domain_to_serenity_create_attachment(attachment);
        message = message.add_file(attachment);
    }

    message
}
