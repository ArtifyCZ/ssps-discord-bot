use domain::ports::discord::ChannelId;
use poise::serenity_prelude as serenity;

pub fn domain_to_serenity_channel_id(channel_id: ChannelId) -> serenity::ChannelId {
    serenity::ChannelId::new(channel_id.0)
}
