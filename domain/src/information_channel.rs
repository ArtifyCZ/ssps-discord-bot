use crate::ports::discord::{CreateActionRow, CreateAttachment, CreateButton, CreateMessage};
use crate::resources;
use tracing::instrument;

pub const VERIFY_ME_BUTTON_ID: &str = "verify_me_button";

#[instrument(level = "trace", skip())]
pub fn create_messages() -> Vec<CreateMessage> {
    vec![
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
        CreateMessage::default()
            .content(resources::VERIFICATION_MD)
            .action_rows(vec![CreateActionRow::buttons(vec![CreateButton::new(
                "Ověřit se",
                VERIFY_ME_BUTTON_ID,
            )])]),
    ]
}
