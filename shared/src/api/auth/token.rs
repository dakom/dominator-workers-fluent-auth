use std::time::Duration;

use crate::user::UserId;
use serde::{Deserialize, Serialize};

// Auth token

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthTokenKind {
    Signin,
    VerifyEmail,
    ChangeEmail,
    ResetPassword,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AuthTokenAfterValidation {
    Delete,
    ExtendExpires(Duration),
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
