use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen::JsValue;

#[derive(Serialize, Deserialize, Error, Debug, Clone)]
pub enum ApiError {
    #[error("{0}")]
    Auth(#[from] AuthError),
    #[error("{0}")]
    Unknown(String),
}

pub type ApiResult<T> = Result<T, ApiError>;


impl From<JsValue> for ApiError {
    fn from(err: JsValue) -> Self {
        Self::Unknown(format!("{:?}", err))
    }
}


impl From<String> for ApiError {
    fn from(err: String) -> Self {
        Self::Unknown(err)
    }
}

impl From<&str> for ApiError {
    fn from(err: &str) -> Self {
        Self::Unknown(err.to_string())
    }
}

#[derive(Serialize, Deserialize, Error, Debug, Clone)]
pub enum AuthError {
    #[error("email needs to be verified")]
    EmailNotVerified,
    #[error("email already exists")]
    EmailAlreadyExists,
    #[error("not authorized")]
    NotAuthorized,
    #[error("invalid signin")]
    InvalidSignin,
    #[error("no such user for password reset")]
    NoUserPasswordReset,
}
