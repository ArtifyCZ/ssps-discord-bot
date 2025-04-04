use serde::{Deserialize, Serialize};

pub struct AuthenticationLink(pub String);

pub struct CsrfToken(pub String);

pub struct ClientCallbackToken(pub String);

#[derive(Clone)]
pub struct AccessToken(pub String);

#[derive(Clone)]
pub struct RefreshToken(pub String);

#[derive(Serialize, Deserialize, Debug)]
pub struct UserGroup {
    pub id: String,
    #[serde(rename = "displayName")]
    pub name: String,
    pub mail: Option<String>,
}
