use crate::authentication::authenticated_user::AuthenticatedUser;
use crate::ports::discord::RoleDiff;
use domain_shared::discord::RoleId;
use tracing::{error, instrument};

pub struct RolesDiffService {
    pub everyone_roles: Vec<RoleId>,
    pub additional_student_roles: Vec<RoleId>,
    pub unknown_class_role_id: RoleId,
    pub class_ids: Vec<String>,
    pub class_id_to_role_id: Vec<(String, RoleId)>,
}

impl RolesDiffService {
    #[instrument(level = "trace", skip_all)]
    pub fn diff_roles(&self, user: Option<&AuthenticatedUser>) -> RoleDiff {
        let mut diff = RoleDiff::default();

        self.diff_everyone_roles(&mut diff);
        self.diff_additional_student_roles(user, &mut diff);
        self.diff_class_roles(user, &mut diff);

        diff
    }

    #[instrument(level = "trace", skip_all)]
    fn diff_everyone_roles(&self, diff: &mut RoleDiff) {
        for everyone_role in &self.everyone_roles {
            diff.assign(*everyone_role);
        }
    }

    #[instrument(level = "trace", skip_all)]
    fn diff_additional_student_roles(&self, user: Option<&AuthenticatedUser>, diff: &mut RoleDiff) {
        for additional_student_role in &self.additional_student_roles {
            if let Some(_user) = user {
                diff.assign(*additional_student_role);
            } else {
                diff.remove(*additional_student_role);
            }
        }
    }

    #[instrument(level = "trace", skip_all)]
    fn diff_class_roles(&self, user: Option<&AuthenticatedUser>, diff: &mut RoleDiff) {
        diff.remove(self.unknown_class_role_id);

        for (_, role_id) in &self.class_id_to_role_id {
            diff.remove(*role_id);
        }

        if let Some(user) = user
            && let Some(user_class_id) = user.class_id()
        {
            for (class_id, role) in &self.class_id_to_role_id {
                if class_id.eq_ignore_ascii_case(user_class_id) {
                    diff.assign(*role);
                    break;
                }
            }
        }

        for (class_id, role_id) in &self.class_id_to_role_id {
            if user
                .and_then(|u| u.class_id().map(|c| c.eq_ignore_ascii_case(class_id)))
                .unwrap_or(false)
            {
                diff.assign(*role_id);
                break;
            }
        }

        if let Some(user) = user {
            let class_role_id = self
                .get_class_role_id(user)
                .unwrap_or(self.unknown_class_role_id);
            diff.assign(class_role_id);
        }
    }

    #[instrument(level = "trace", skip_all)]
    fn get_class_role_id(&self, user: &AuthenticatedUser) -> Option<RoleId> {
        let class_id = user.class_id()?;
        let (_, role_id) = self
            .class_id_to_role_id
            .iter()
            .find(|(c, _)| c.eq_ignore_ascii_case(class_id))
            .or_else(|| {
                error!(
                    "Not found class role for class id {:?} for user {:?}: {:?}",
                    class_id,
                    user.user_id(),
                    self.class_id_to_role_id,
                );
                None
            })?;
        Some(*role_id)
    }
}
