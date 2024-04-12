use serde::{Deserialize, Serialize};
use worker::{console_error, D1Database, D1Result, Env, State};
use crate::prelude::*;

// sqllite uses integers for booleans
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct DbBool(u32);

impl From<bool> for DbBool {
    fn from(v: bool) -> Self {
        if v {
            Self(1)
        } else {
            Self(0)
        }
    }
}

impl From<DbBool> for bool {
    fn from(v: DbBool) -> Self {
        v.0 == 1
    }
}

impl From<u32> for DbBool {
    fn from(v: u32) -> Self {
        match v {
            0 => Self(0),
            1 => Self(1),
            _ => panic!("invalid value for DbBool: {}", v),
        }
    }
}

impl From<DbBool> for JsValue {
    fn from(b: DbBool) -> Self {
        b.0.into()
    }
}

#[cfg(debug_assertions)]
pub fn get_d1(env: &Env) -> ApiResult<D1Database> {
    env.d1("DB_DEV").map_err(|err| err.into())
}

#[cfg(not(debug_assertions))]
pub fn get_d1(env: &Env) -> ApiResult<D1Database> {
    env.d1("DB_PROD").map_err(|err| err.into())
}

pub trait D1ResultExt {
    fn into_result(self) -> ApiResult<()>;
} 

impl D1ResultExt for D1Result {
    fn into_result(self) -> ApiResult<()> {
        match self.error() {
            Some(err) => Err(err.into()),
            None => Ok(()),
        }
    }
}

pub trait DurableObjectExt {
    fn state(&self) -> &State;
    async fn storage_get<T: serde::de::DeserializeOwned>(&self, key: &str) -> worker::Result<T> {
        match self.state().storage().get(key).await {
            Err(err) => {
                console_error!("error getting storage key {} on DurableObject id {}", key.to_string(), self.state().id().to_string());
                Err(err)
            },
            Ok(value) => Ok(value)
        }
    }
}

pub fn get_secret(env: &Env, key: &str) -> worker::Result<String> {
    env.secret(key)
        .map(|secret| secret.to_string())
        .map_err(|err| {
            console_error!("could not get env secret: {key}");
            err
        })
}
