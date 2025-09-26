use poise::serenity_prelude as serenity;
use tracing::instrument;

#[instrument(level = "trace", skip(user_id))]
pub fn domain_to_serenity_user_id(user_id: domain_shared::discord::UserId) -> serenity::UserId {
    serenity::UserId::new(user_id.0)
}

#[instrument(level = "trace", skip(user_id))]
pub fn serenity_to_domain_user_id(user_id: serenity::UserId) -> domain_shared::discord::UserId {
    domain_shared::discord::UserId(user_id.get())
}
