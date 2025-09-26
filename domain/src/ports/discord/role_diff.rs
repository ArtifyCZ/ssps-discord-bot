use domain_shared::discord::RoleId;
use std::ops::{AddAssign, Not};
use tracing::instrument;

#[derive(Debug)]
pub struct RoleDiff {
    pub to_assign: Vec<RoleId>,
    pub to_remove: Vec<RoleId>,
}

impl Default for RoleDiff {
    #[instrument(level = "trace", skip_all)]
    fn default() -> Self {
        Self {
            to_assign: Vec::new(),
            to_remove: Vec::new(),
        }
    }
}

impl AddAssign for RoleDiff {
    #[instrument(level = "trace", skip_all)]
    fn add_assign(&mut self, rhs: Self) {
        for assign_role in rhs.to_assign {
            self.to_assign.push(assign_role);
        }

        self.to_assign.sort();
        self.to_assign.dedup();

        let mut to_remove = vec![];

        for remove_role in &self.to_remove {
            if self.to_assign.iter().any(|r| r == remove_role).not()
                && to_remove.iter().any(|r| r == remove_role).not()
            {
                to_remove.push(*remove_role);
            }
        }

        self.to_remove = to_remove;
    }
}
