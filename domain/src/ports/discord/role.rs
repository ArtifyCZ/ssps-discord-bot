use domain_shared::discord::RoleId;

#[derive(Debug)]
pub struct Role {
    pub role_id: RoleId,
    pub name: String,
}
