use domain_shared::discord::RoleId;

#[derive(Debug)]
pub struct RoleDiff {
    pub to_assign: Vec<RoleId>,
    pub to_remove: Vec<RoleId>,
}
