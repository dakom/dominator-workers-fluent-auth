use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen::JsValue;

#[derive(Deserialize, Serialize, Error, Debug, Clone)]
pub enum ApiError {
    #[error("{0}")]
    Auth(#[from] AuthError),
    #[error("{0}")]
    Unknown(String),
    #[error("{0}")]
    Parse(String),
    #[error("{0}")]
    ParseBody(String),

    #[error("kv error: {0}")]
    Kv(String),

    #[error("kv error: {0}")]
    Db(String),

    #[error("missing body {0}")]
    MissingBody(String),
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

#[derive(Serialize, Deserialize, Error, Debug, Clone, PartialEq)]
pub enum AuthError {
    #[error("not authorized")]
    NotAuthorized,
    #[error("invalid login")]
    InvalidLogin,
    #[error("email address is empty")]
    EmailEmpty,
    #[error("email needs to be verified")]
    EmailNotVerified,
    #[error("email already exists")]
    EmailAlreadyExists,
    #[error("terms have not been agreed")]
    TermsNotAgreed,
}
