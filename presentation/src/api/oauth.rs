use crate::application_ports::Locator;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Redirect;
use domain_shared::authentication::{ClientCallbackToken, CsrfToken};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}

pub async fn callback_handler<L: Locator>(
    State(service_locator): State<L>,
    Query(query): Query<AuthRequest>,
) -> Redirect {
    let authentication_port = service_locator.get_authentication_port();
    let invite_link = authentication_port
        .confirm_authentication(CsrfToken(query.state), ClientCallbackToken(query.code))
        .await
        .unwrap();

    Redirect::to(&invite_link.0)
}
