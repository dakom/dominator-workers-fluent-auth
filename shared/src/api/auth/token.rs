use serde::{Deserialize, Serialize};
use crate::user::UserId;

// Auth token

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AuthTokenKind {
    Login,
}

impl TryFrom<String> for AuthTokenKind {
    type Error = &'static str;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "login" => Ok(Self::Login),
            _ => Err("invalid kind"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AuthTokenAfterValidation {
    Delete,
    ExtendExpiresMs(u64),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthTokenCreateResponse {
    pub id: String,
    pub key: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthTokenValidateResponse {
    pub uid: UserId,
    pub user_token: String,
}
