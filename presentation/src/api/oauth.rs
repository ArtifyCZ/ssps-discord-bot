use crate::application_ports::Locator;
use application_ports::authentication::AuthenticationError;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use domain_shared::authentication::{ClientCallbackToken, CsrfToken};
use serde::Deserialize;
use tracing::{instrument, warn};

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
    let invite_link = match authentication_port
        .confirm_authentication(CsrfToken(query.state), ClientCallbackToken(query.code))
        .await
    {
        Ok(invite_link) => invite_link,
        Err(AuthenticationError::Error(error)) => {
            warn!("Error during authentication: {:?}", error);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        Err(AuthenticationError::AlreadyAuthenticated) => {
            return StatusCode::NO_CONTENT.into_response()
        }
        Err(AuthenticationError::AuthenticationRequestNotFound) => {
            warn!("Authentication request not found");
            return Redirect::to("/").into_response();
        }
        Err(AuthenticationError::EmailAlreadyInUse) => {
            warn!("Email already in use");
            return StatusCode::CONFLICT.into_response();
        }
    };

    Redirect::to(&invite_link.0).into_response()
}
