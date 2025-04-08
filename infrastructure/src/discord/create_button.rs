use domain::ports::discord::{ButtonKind, CreateButton};
use poise::serenity_prelude as serenity;

pub fn domain_to_serenity_create_button(button: CreateButton) -> serenity::CreateButton {
    let CreateButton { label, data } = button;

    match data {
        ButtonKind::Link { url } => serenity::CreateButton::new_link(url),
        ButtonKind::NonLink { button_id } => serenity::CreateButton::new(button_id.0.to_string()),
    }
    .label(label)
}
