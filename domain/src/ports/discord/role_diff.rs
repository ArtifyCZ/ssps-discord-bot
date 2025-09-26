use domain_shared::discord::RoleId;
use std::ops::Not;
use tracing::instrument;

#[derive(Debug, Default)]
pub struct RoleDiff {
    to_assign: Vec<RoleId>,
    to_remove: Vec<RoleId>,
}

impl RoleDiff {
    #[instrument(level = "trace", skip_all)]
    pub fn assign(&mut self, role_id: RoleId) -> &mut Self {
        self.to_assign.push(role_id);
        self.to_assign.sort();
        self.to_assign.dedup();

        self.to_remove.retain(|r| r != &role_id);

        self
    }

    #[instrument(level = "trace", skip_all)]
    pub fn remove(&mut self, role_id: RoleId) -> &mut Self {
        if self.to_assign.contains(&role_id).not() {
            self.to_remove.push(role_id);
            self.to_remove.sort();
            self.to_remove.dedup();
        }

        self
    }

    #[instrument(level = "trace", skip_all)]
    pub fn optimize_by_already_assigned_roles(&mut self, assigned_roles: &[RoleId]) -> &mut Self {
        // Assign only those that the user doesn't have yet
        self.to_assign.retain(|r| assigned_roles.contains(r).not());

        // Remove only those that the user still has
        self.to_remove.retain(|r| assigned_roles.contains(r));

        self
    }

    #[instrument(level = "trace", skip_all)]
    pub fn to_assign(&self) -> &[RoleId] {
        &self.to_assign
    }

    #[instrument(level = "trace", skip_all)]
    pub fn to_remove(&self) -> &[RoleId] {
        &self.to_remove
    }
}
