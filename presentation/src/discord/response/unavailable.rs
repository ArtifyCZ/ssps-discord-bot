use poise::CreateReply;
use tracing::instrument;

#[instrument(level = "debug", skip_all)]
pub fn temporary_unavailable() -> CreateReply {
    let response = "Omlouváme se, služba je momentálně nedostupná. Zkus to prosím později, případně kontaktujte admina.";

    CreateReply::default()
        .content(response)
        .ephemeral(true)
        .reply(true)
}
