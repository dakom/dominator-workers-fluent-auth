use std::time::Duration;

use crate::{time::Timestamp, user::UserId};
use serde::{Deserialize, Serialize};

// Auth token

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
pub struct AuthSigninTokenValidateResponse {
    pub uid: UserId,
    pub user_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AuthTokenData {
    Signin {
        uid: UserId,
        user_token: String,
        key: String,
        expires_at: Timestamp,
    },
    VerifyEmail {
        uid: UserId,
        key: String,
        expires_at: Timestamp,
    },
}

impl AuthTokenData {
    pub fn expires_at(&self) -> Timestamp {
        match self {
            AuthTokenData::Signin { expires_at, .. } => *expires_at,
            AuthTokenData::VerifyEmail { expires_at, .. } => *expires_at,
        }
    }

    pub fn uid(&self) -> &UserId {
        match self {
            AuthTokenData::Signin { uid, .. } => uid,
            AuthTokenData::VerifyEmail { uid, .. } => uid,
        }
    }

    pub fn key(&self) -> &str {
        match self {
            AuthTokenData::Signin { key, .. } => key,
            AuthTokenData::VerifyEmail { key, .. } => key,
        }
    }

    pub fn update_expires_at(&mut self, new_expires_at: Timestamp) {
        match self {
            AuthTokenData::Signin { expires_at, .. } => *expires_at = new_expires_at,
            AuthTokenData::VerifyEmail { expires_at, .. } => *expires_at = new_expires_at,
        }
    }
}