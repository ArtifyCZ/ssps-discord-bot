use domain_shared::authentication::AuthenticationLink;
use oauth2::url::Url;
use tracing::instrument;

#[instrument(level = "trace", skip(link))]
pub fn oauth_to_domain_authentication_link(link: Url) -> AuthenticationLink {
    AuthenticationLink(link.to_string())
}
