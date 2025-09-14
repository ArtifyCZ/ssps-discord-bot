use domain_shared::discord::RoleId;
use poise::serenity_prelude as serenity;
use tracing::instrument;

#[instrument(level = "trace", skip(role_id))]
pub fn domain_to_serenity_role_id(role_id: RoleId) -> serenity::RoleId {
    serenity::RoleId::new(role_id.0)
}

#[instrument(level = "trace", skip(role_id))]
pub fn serenity_to_domain_role_id(role_id: serenity::RoleId) -> RoleId {
    RoleId(role_id.get())
}
