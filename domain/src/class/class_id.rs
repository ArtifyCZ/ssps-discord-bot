use crate::authentication::create_class_user_group_id_mails;
use domain_shared::authentication::UserGroup;
use tracing::instrument;

#[instrument(level = "trace")]
pub fn get_class_id(group: &UserGroup) -> Option<String> {
    if let Some(mail) = &group.mail {
        let class_group_id_mails = create_class_user_group_id_mails();
        class_group_id_mails
            .into_iter()
            .find(|(_, m)| m.eq(mail))
            .map(|(id, _)| id)
    } else {
        None
    }
}
