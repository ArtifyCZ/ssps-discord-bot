use crate::discord::create_attachment::domain_to_serenity_create_attachment;
use crate::discord::create_button::domain_to_serenity_create_button;
use domain::ports::discord::{CreateActionRow, CreateMessage};
use poise::serenity_prelude as serenity;

pub fn domain_to_serenity_create_message(message: CreateMessage) -> serenity::CreateMessage {
    let CreateMessage {
        content,
        attachments,
        action_rows,
    } = message;

    let mut message = serenity::CreateMessage::default();

    if let Some(content) = content {
        message = message.content(content);
    }

    for attachment in attachments {
        let attachment = domain_to_serenity_create_attachment(attachment);
        message = message.add_file(attachment);
    }

    let action_rows = action_rows
        .into_iter()
        .map(|action_row| match action_row {
            CreateActionRow::Buttons { components } => {
                let components = components
                    .into_iter()
                    .map(domain_to_serenity_create_button)
                    .collect::<Vec<_>>();

                serenity::CreateActionRow::Buttons(components)
            }
        })
        .collect();

    message = message.components(action_rows);

    message
}
