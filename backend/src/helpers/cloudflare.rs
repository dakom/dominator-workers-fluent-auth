use crate::{config::DB_BINDING, prelude::*};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use worker::{kv::ToRawKvValue, D1Database, D1Result, Env, State};

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

pub fn get_d1(env: &Env) -> ApiResult<D1Database> {
    env.d1(&DB_BINDING).map_err(|err| err.to_string().into())
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

pub async fn put_kv_json(
    env: &Env,
    namespace: &str,
    key: &str,
    value: impl Serialize,
) -> ApiResult<()> {
    // technically, we could maybe take a into<RawKvValue> here, but
    // that makes it harder to debug and cloudflare wants to receive it as a JsValue, not impl Serialize
    let bytes = serde_json::to_vec(&value).map_err(|e| ApiError::Kv(e.to_string()))?;

    env.kv(namespace)
        .map_err(|e| ApiError::Kv(e.to_string()))?
        .put_bytes(key, &bytes)
        .map_err(|e| ApiError::Kv(e.to_string()))?
        .execute()
        .await
        .map_err(|e| ApiError::Kv(e.to_string()))
}

pub async fn try_get_kv_json<D: DeserializeOwned>(
    env: &Env,
    namespace: &str,
    key: &str,
) -> ApiResult<Option<D>> {
    let bytes = env
        .kv(namespace)
        .map_err(|e| ApiError::Kv(e.to_string()))?
        .get(key)
        .bytes()
        .await
        .map_err(|e| ApiError::Kv(e.to_string()))?;

    bytes
        .map(|bytes| serde_json::from_slice(&bytes).map_err(|e| ApiError::Kv(e.to_string())))
        .transpose()
}

pub async fn get_kv_json<D: DeserializeOwned>(
    env: &Env,
    namespace: &str,
    key: &str,
) -> ApiResult<D> {
    try_get_kv_json(env, namespace, key)
        .await?
        .ok_or_else(|| ApiError::Kv(format!("missing kv key {} in namespace {}", key, namespace)))
}

pub async fn delete_kv(env: &Env, namespace: &str, key: &str) -> ApiResult<()> {
    env.kv(namespace)
        .map_err(|e| ApiError::Kv(e.to_string()))?
        .delete(key)
        .await
        .map_err(|e| ApiError::Kv(e.to_string()))
}
