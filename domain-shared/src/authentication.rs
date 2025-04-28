use serde::{Deserialize, Serialize};

pub struct AuthenticationLink(pub String);

#[derive(Clone, PartialEq, Debug)]
pub struct CsrfToken(pub String);

#[derive(Clone, Debug, PartialEq)]
pub struct ClientCallbackToken(pub String);

#[derive(Clone, Debug, PartialEq)]
pub struct AccessToken(pub String);

#[derive(Clone, Debug, PartialEq)]
pub struct RefreshToken(pub String);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserGroup {
    pub id: String,
    #[serde(rename = "displayName")]
    pub name: String,
    pub mail: Option<String>,
}
