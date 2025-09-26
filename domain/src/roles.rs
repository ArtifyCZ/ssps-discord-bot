use crate::authentication::authenticated_user::AuthenticatedUser;
use crate::ports::discord::{Role, RoleDiff};
use domain_shared::discord::RoleId;
use std::ops::Not;
use tracing::{error, instrument};

#[instrument(level = "trace", skip_all)]
pub fn diff_everyone_roles(everyone_roles: &[RoleId], assigned_roles: &[Role]) -> RoleDiff {
    let mut to_assign = Vec::new();

    for everyone_role in everyone_roles {
        if assigned_roles
            .iter()
            .any(|r| r.role_id == *everyone_role)
            .not()
        {
            to_assign.push(*everyone_role);
        }
    }

    RoleDiff {
        to_assign,
        to_remove: Vec::new(),
    }
}

#[instrument(level = "trace", skip_all)]
pub fn diff_additional_student_roles(
    additional_student_roles: &[RoleId],
    assigned_roles: &[Role],
    is_authenticated: bool,
) -> RoleDiff {
    let mut diff = RoleDiff::default();

    if is_authenticated {
        for additional_student_role in additional_student_roles {
            if assigned_roles
                .iter()
                .any(|r| r.role_id == *additional_student_role)
                .not()
            {
                diff.to_assign.push(*additional_student_role);
            }
        }
    } else {
        for additional_student_role in additional_student_roles {
            if assigned_roles
                .iter()
                .any(|r| r.role_id == *additional_student_role)
            {
                diff.to_remove.push(*additional_student_role);
            }
        }
    }

    diff
}

/// If the `user` is `None`, then the user is not authenticated
#[instrument(level = "trace", skip_all)]
pub fn diff_class_roles(
    unknown_class_role_id: RoleId,
    class_ids: &[String],
    class_id_to_role_id: &[(String, RoleId)],
    user: Option<&AuthenticatedUser>,
    assigned_roles: &[Role],
) -> RoleDiff {
    let mut diff = RoleDiff::default();

    for assigned_role in assigned_roles {
        if assigned_role.role_id == unknown_class_role_id
            && user.map(|u| u.class_id().is_some()).unwrap_or(true)
        {
            diff.to_remove.push(assigned_role.role_id);
            continue;
        }

        for class_id in class_ids {
            if assigned_role.name.eq_ignore_ascii_case(class_id)
                && user
                    .and_then(|u| u.class_id().map(|c| c.eq_ignore_ascii_case(class_id).not()))
                    .unwrap_or(true)
            {
                diff.to_remove.push(assigned_role.role_id);
                continue;
            }
        }
    }

    if let Some(user) = user {
        let class_role_id = match user.class_id() {
            None => unknown_class_role_id,
            Some(class_id) => class_id_to_role_id.iter().find(|(c, _)| c.eq_ignore_ascii_case(class_id)).map(|(_, r)| *r).unwrap_or_else(|| {
                error!(
                    "Not found class role for class id {:?} for user {:?}, defaulting to unknown class role",
                    class_id,
                    user.user_id(),
                );
                unknown_class_role_id
            }),
        };

        if assigned_roles
            .iter()
            .any(|r| r.role_id == class_role_id)
            .not()
        {
            diff.to_assign.push(class_role_id);
        }
    }

    diff
}
