use crate::authentication::create_class_user_group_id_mails;
use domain_shared::authentication::UserGroup;
use tracing::instrument;

#[instrument(level = "trace")]
pub fn find_class_group(groups: &[UserGroup]) -> Option<&UserGroup> {
    let class_group_id_mails = create_class_user_group_id_mails();
    let class_group_mails = class_group_id_mails
        .iter()
        .map(|(_, mail)| mail)
        .collect::<Vec<_>>();

    groups.iter().find(|group| {
        group
            .mail
            .as_ref()
            .map(|mail| class_group_mails.contains(&mail))
            .unwrap_or(false)
    })
}
