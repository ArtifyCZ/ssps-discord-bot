use crate::authentication::authenticated_user::AuthenticatedUser;
use crate::authentication::create_class_ids;
use crate::ports::discord::{DiscordPort, RoleDiff};
use domain_shared::discord::RoleId;
use std::sync::Arc;
use tracing::instrument;

pub struct UserRoleService {
    additional_student_roles: Vec<RoleId>,
    class_id_to_role_id: Vec<(String, RoleId)>,
}

impl UserRoleService {
    #[instrument(level = "trace", skip_all)]
    pub async fn new(
        discord_port: Arc<dyn DiscordPort + Sync + Send>,
        additional_student_roles: Vec<RoleId>,
    ) -> Self {
        let class_ids = create_class_ids();
        let mut class_id_to_role_id = vec![];
        for class_id in class_ids {
            let role = discord_port
                .find_or_create_role_by_name(&class_id.to_uppercase(), "Role for students of class")
                .await
                .expect("Cannot construct UserRoleService");
            class_id_to_role_id.push((class_id, role.role_id));
        }

        Self {
            additional_student_roles,
            class_id_to_role_id,
        }
    }

    pub fn assign_user_roles(&self, user: &AuthenticatedUser) -> RoleDiff {
        let mut to_assign = vec![];
        let mut to_remove = vec![];

        for role in &self.additional_student_roles {
            to_assign.push(*role);
        }

        for (class_id, role_id) in &self.class_id_to_role_id {
            if user.class_id() == class_id {
                to_assign.push(*role_id);
            } else {
                to_remove.push(*role_id);
            }
        }

        RoleDiff {
            to_assign,
            to_remove,
        }
    }

    pub fn remove_user_roles(&self) -> RoleDiff {
        let mut to_remove = vec![];

        for role in &self.additional_student_roles {
            to_remove.push(*role);
        }

        for (_, role_id) in &self.class_id_to_role_id {
            to_remove.push(*role_id);
        }

        RoleDiff {
            to_assign: vec![],
            to_remove,
        }
    }
}
