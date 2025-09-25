use crate::authentication::create_class_ids;
use domain_shared::authentication::UserGroup;
use tracing::{error, instrument};

#[instrument(level = "trace")]
pub fn find_class_group(groups: &[UserGroup]) -> Option<&UserGroup> {
    let class_ids = create_class_ids();

    for group in groups {
        if let Some(ref mail) = group.mail {
            let mut mail = mail.split('@');
            let local_part = mail.next().unwrap_or("").trim();
            let domain_part = mail.next().unwrap_or("").trim();

            for class_id in &class_ids {
                if class_id.eq_ignore_ascii_case(local_part) && check_mail_domain_part(domain_part)
                {
                    return Some(group);
                }
            }
        }
    }

    error!(
        groups = ?groups,
        "Should have found class group, but no class group found in user's groups",
    );

    None
}

/// Checks if the domain part of a mail address matches the school domain.
#[instrument(level = "trace")]
fn check_mail_domain_part(domain_part: &str) -> bool {
    let allowed_domain_parts = ["ssps.cz", "skola.ssps.cz"];

    for allowed_domain_part in &allowed_domain_parts {
        if domain_part.trim().eq_ignore_ascii_case(allowed_domain_part) {
            return true;
        }
    }

    false
}
