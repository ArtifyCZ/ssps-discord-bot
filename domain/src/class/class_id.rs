use crate::authentication::create_class_ids;
use domain_shared::authentication::UserGroup;
use tracing::{error, instrument};

#[instrument(level = "trace")]
pub fn get_class_id(group: &UserGroup) -> Option<String> {
    let mail = group.mail.as_ref()?;
    let mut mail = mail.split('@');
    let local_part = mail.next().unwrap_or("").trim();
    if local_part.is_empty() {
        return None;
    }

    let class_id = local_part.to_string();
    let class_ids = create_class_ids();
    if !class_ids.contains(&class_id) {
        error!(
            class_id = class_id.as_str(),
            "Class ID does not match any of the class IDs in the application",
        );
    }

    Some(class_id)
}
