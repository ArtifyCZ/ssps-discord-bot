use poise::serenity_prelude as serenity;

pub fn domain_to_serenity_user_id(user_id: domain_shared::discord::UserId) -> serenity::UserId {
    serenity::UserId::new(user_id.0)
}
