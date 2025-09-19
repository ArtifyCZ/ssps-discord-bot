use crate::application_ports::Locator;
use application_ports::authentication::AuthenticationError;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use domain_shared::authentication::{ClientCallbackToken, CsrfToken};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{CreateMessage, Mentionable};
use serde::Deserialize;
use tracing::{error, instrument, warn};

#[derive(Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}

#[instrument(level = "info", skip(service_locator, query))]
pub async fn callback_handler<L: Locator>(
    State(service_locator): State<L>,
    Query(query): Query<AuthRequest>,
) -> Response {
    let authentication_port = service_locator.get_authentication_port();
    let discord_client = service_locator.get_discord_client();

    let (user_id, invite_link) = match authentication_port
        .confirm_authentication(CsrfToken(query.state), ClientCallbackToken(query.code))
        .await
    {
        Ok(result) => result,
        Err(err) => return response_auth_error(err),
    };
    let user_id = serenity::UserId::new(user_id.0);
    let msg = response_successfully_verified(user_id, &invite_link.0);
    let message = match user_id.direct_message(discord_client, msg).await {
        Ok(msg) => msg,
        Err(err) => {
            error!(
                error = ?err,
                user_id = user_id.get(),
                "Failed to send direct message confirming the verification",
            );
            return response_successful_but_failed_to_send_message();
        }
    };

    Redirect::to(&message.link()).into_response()
}

#[instrument(level = "trace", skip_all)]
fn response_auth_error(error: AuthenticationError) -> Response {
    match error {
        AuthenticationError::AuthenticationRequestNotFound => {
            warn!("Authentication request not found");
            (
                StatusCode::NOT_FOUND,
                "Authentication request not found, the request may have been fulfilled already",
            )
                .into_response()
        }
        AuthenticationError::TemporaryUnavailable => {
            warn!("Authentication is temporarily unavailable");
            (StatusCode::SERVICE_UNAVAILABLE, "Verification is currently unavailable, please contact the admin team and try later").into_response()
        }
    }
}

#[instrument(level = "trace", skip_all)]
fn response_successful_but_failed_to_send_message() -> Response {
    (
        StatusCode::OK,
        "Byl jsi úspětně ověřen, ale bot ti nemohl poslat potvrzující zprávu. Nyní se můžeš vrátit na SSPŠ Discord server.",
        ).into_response()
}

#[instrument(level = "trace", skip_all)]
fn response_successfully_verified(user_id: serenity::UserId, invite_link: &str) -> CreateMessage {
    CreateMessage::default().content(format!(
        "Ahoj, {}! Byl jsi úspěšně ověřen. Nyní se můžeš vrátit na [SSPŠ Discord server]({})!",
        user_id.mention(),
        invite_link,
    ))
}
