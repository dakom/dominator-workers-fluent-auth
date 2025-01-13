use std::time::Duration;

use base64::Engine;
use rand::{rngs::OsRng, Rng};
use serde::{Deserialize, Serialize};
use shared::{
    api::auth::{
        AuthTokenAfterValidation, AuthTokenCreateResponse, AuthTokenKind, AuthTokenValidateResponse,
    },
    backend::result::{ApiError, ApiResult},
    time::Timestamp,
    user::UserId,
};
use uuid::Uuid;
use worker::{console_log, Env};

use crate::{
    config::{AUTH_TOKEN_KEY_LENGTH, KV_BINDING_AUTH_TOKEN_SIGNIN},
    delete_kv, get_kv_json, put_kv_json,
};

pub struct AuthKv {}

impl AuthKv {
    pub async fn create(
        env: &Env,
        kind: AuthTokenKind,
        uid: UserId,
        user_token: String,
        expires: Duration,
    ) -> ApiResult<AuthTokenCreateResponse> {
        let id = Uuid::now_v7().as_simple().to_string();
        let key = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(&OsRng.gen::<[u8; AUTH_TOKEN_KEY_LENGTH]>());

        let data = AuthSigninTokenData {
            uid,
            kind,
            user_token,
            key: key.clone(),
            expires_at: Timestamp::now() + expires,
        };

        put_kv_json(env, KV_BINDING_AUTH_TOKEN_SIGNIN, &id, &data).await?;

        Ok(AuthTokenCreateResponse { id, key })
    }

    pub async fn delete(env: &Env, _kind: AuthTokenKind, id: &str) -> ApiResult<()> {
        delete_kv(env, KV_BINDING_AUTH_TOKEN_SIGNIN, id).await
    }

    pub async fn validate(
        env: &Env,
        kind: AuthTokenKind,
        id: &str,
        key: String,
        after: AuthTokenAfterValidation,
    ) -> ApiResult<AuthTokenValidateResponse> {
        let mut token: AuthSigninTokenData =
            get_kv_json(env, &KV_BINDING_AUTH_TOKEN_SIGNIN, id).await?;

        console_log!("validating token: {:?}", token);

        if kind != token.kind {
            return Err("invalid kind".into());
        }

        if key != token.key {
            return Err("invalid key".into());
        }

        if token.expires_at < Timestamp::now() {
            delete_kv(env, KV_BINDING_AUTH_TOKEN_SIGNIN, id).await?;
            return Err("token expired".into());
        }

        match after {
            AuthTokenAfterValidation::Delete => {
                delete_kv(env, KV_BINDING_AUTH_TOKEN_SIGNIN, id).await?;
            }
            AuthTokenAfterValidation::ExtendExpires(expires) => {
                token.expires_at = Timestamp::now() + expires;
                put_kv_json(env, KV_BINDING_AUTH_TOKEN_SIGNIN, &id, &token).await?;
            }
        }

        Ok(AuthTokenValidateResponse {
            uid: token.uid,
            user_token: token.user_token,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
struct AuthSigninTokenData {
    uid: UserId,
    kind: AuthTokenKind,
    user_token: String,
    key: String,
    expires_at: Timestamp,
}
