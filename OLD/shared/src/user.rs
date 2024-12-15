use std::fmt::Display;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new(u: Uuid) -> Self {
        Self(u)
    }

    pub fn to_string(&self) -> String {
        self.0.simple().to_string()
    }
}

impl From<&UserId> for JsValue {
    fn from(u: &UserId) -> Self {
        u.to_string().into()
    }
}

impl From<UserId> for JsValue {
    fn from(u: UserId) -> Self {
        u.to_string().into()
    }
}

impl TryFrom<&str> for UserId {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match Uuid::parse_str(value) {
            Ok(u) => Ok(Self(u)),
            Err(e) => Err(e.to_string())
        }
    }
}

impl TryFrom<String> for UserId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}