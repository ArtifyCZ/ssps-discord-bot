use crate::application_ports::Locator;
use axum::Router;
use axum::extract::MatchedPath;
use axum::http::Request;
use tower_http::trace::TraceLayer;
use tracing::{info_span, instrument};

pub mod oauth;

#[instrument(level = "trace", skip())]
pub fn create_router<L: Locator + Send + Sync + Clone + 'static>() -> Router<L> {
    Router::new().route(
        "/oauth/callback",
        axum::routing::get(oauth::callback_handler::<L>),
    )
}

#[instrument(level = "debug", skip(locator))]
pub async fn run_api<L: Locator + Send + Sync + Clone + 'static>(
    locator: L,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let router: Router<()> = create_router::<L>()
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                info_span!(
                    "http_request",
                    method = ?request.method(),
                    matched_path,
                    some_other_field = tracing::field::Empty,
                )
            }),
        )
        .with_state(locator);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}
