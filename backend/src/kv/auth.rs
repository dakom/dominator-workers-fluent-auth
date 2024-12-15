use base64::Engine;
use rand::Rng;
use serde::{Deserialize, Serialize};
use shared::{
    api::auth::{
        AuthTokenAfterValidation, AuthTokenCreateResponse, AuthTokenKind, AuthTokenValidateResponse,
    },
    backend::result::ApiResult,
    user::UserId,
};
use uuid::Uuid;
use worker::{Date, Env};

use crate::{
    config::{AUTH_TOKEN_KEY_LENGTH, KV_BINDING_AUTH_TOKEN_SIGNIN},
    delete_kv, get_kv_json, put_kv,
};

pub struct AuthKv {}

impl AuthKv {
    pub async fn create(
        env: &Env,
        kind: AuthTokenKind,
        uid: UserId,
        user_token: String,
        expires_ms: u64,
    ) -> ApiResult<AuthTokenCreateResponse> {
        let id = Uuid::now_v7().as_simple().to_string();
        let key = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(&rand::thread_rng().gen::<[u8; AUTH_TOKEN_KEY_LENGTH]>());

        let data = AuthLoginTokenData {
            uid,
            kind,
            user_token,
            key: key.clone(),
            expires_at: Date::now().as_millis() + expires_ms,
        };

        put_kv(env, KV_BINDING_AUTH_TOKEN_SIGNIN, &id, &data).await?;

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
        let mut token: AuthLoginTokenData =
            get_kv_json(env, &KV_BINDING_AUTH_TOKEN_SIGNIN, id).await?;

        if kind != token.kind {
            return Err("invalid kind".into());
        }

        if key != token.key {
            return Err("invalid key".into());
        }

        if token.expires_at < Date::now().as_millis() {
            delete_kv(env, KV_BINDING_AUTH_TOKEN_SIGNIN, id).await?;
            return Err("token expired".into());
        }

        match after {
            AuthTokenAfterValidation::Delete => {
                delete_kv(env, KV_BINDING_AUTH_TOKEN_SIGNIN, id).await?;
            }
            AuthTokenAfterValidation::ExtendExpiresMs(expires_ms) => {
                token.expires_at = Date::now().as_millis() + expires_ms;
                put_kv(env, KV_BINDING_AUTH_TOKEN_SIGNIN, &id, &token).await?;
            }
        }

        Ok(AuthTokenValidateResponse {
            uid: token.uid,
            user_token: token.user_token,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthLoginTokenData {
    uid: UserId,
    kind: AuthTokenKind,
    user_token: String,
    key: String,
    expires_at: u64,
}
