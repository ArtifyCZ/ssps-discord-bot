use crate::discord::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

macro_rules! impl_sensitive_tuple_debug {
    ($t:ty) => {
        impl ::core::fmt::Debug for $t {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($t)).field(&"*****").finish()
            }
        }
    };
}

pub struct AuthenticationLink(pub String);
impl_sensitive_tuple_debug!(AuthenticationLink);

#[derive(Clone, PartialEq)]
pub struct CsrfToken(pub String);
impl_sensitive_tuple_debug!(CsrfToken);

#[derive(Clone, PartialEq)]
pub struct ClientCallbackToken(pub String);
impl_sensitive_tuple_debug!(ClientCallbackToken);

#[derive(Clone, PartialEq)]
pub struct AccessToken(pub String);
impl_sensitive_tuple_debug!(AccessToken);

#[derive(Clone, PartialEq)]
pub struct RefreshToken(pub String);
impl_sensitive_tuple_debug!(RefreshToken);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserGroup {
    pub id: String,
    #[serde(rename = "displayName")]
    pub name: String,
    pub mail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArchivedUserId(pub UserId, pub DateTime<Utc>);
