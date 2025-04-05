use domain_shared::authentication::CsrfToken;

pub fn oauth_to_domain_csrf_token(csrf_token: oauth2::CsrfToken) -> CsrfToken {
    CsrfToken(csrf_token.into_secret())
}
