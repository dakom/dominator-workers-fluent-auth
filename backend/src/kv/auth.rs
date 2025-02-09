use std::time::Duration;

use base64::Engine;
use rand::{rngs::OsRng, Rng};
use shared::{
    api::auth::{
        AuthTokenAfterValidation, AuthTokenCreateResponse, AuthTokenData,
    },
    backend::result::ApiResult,
    time::Timestamp,
    user::UserId,
};
use uuid::Uuid;
use worker::{console_log, Env};

use crate::{
    config::{AUTH_TOKEN_KEY_LENGTH, AUTH_TOKEN_SIGNIN_EXPIRES_DURATION, AUTH_TOKEN_VERIFY_EMAIL_EXPIRES_DURATION, KV_BINDING_AUTH_TOKEN},
    delete_kv, get_kv_json, put_kv_json,
};

pub struct AuthKv {}

impl AuthKv {
    async fn create_inner(env: &Env, id: String, data: AuthTokenData) -> ApiResult<AuthTokenCreateResponse> {
        put_kv_json(env, KV_BINDING_AUTH_TOKEN, &id, &data).await?;

        Ok(AuthTokenCreateResponse { id, key: data.key().to_string() })
    }

    pub async fn create_signin(
        env: &Env,
        uid: UserId,
        user_token: String,
    ) -> ApiResult<AuthTokenCreateResponse> {
        let (id, key) = new_id_key();

        Self::create_inner(env, id, AuthTokenData::Signin {
            uid,
            user_token,
            key: key.clone(),
            expires_at: Timestamp::now() + *AUTH_TOKEN_SIGNIN_EXPIRES_DURATION,
        }).await
    }

    pub async fn create_verify_email(
        env: &Env,
        uid: UserId,
    ) -> ApiResult<AuthTokenCreateResponse> {
        let (id, key) = new_id_key();

        Self::create_inner(env, id, AuthTokenData::VerifyEmail { 
            uid,
            key: key.clone(),
            expires_at: Timestamp::now() + *AUTH_TOKEN_VERIFY_EMAIL_EXPIRES_DURATION,
        }).await
    }

    pub async fn delete(env: &Env, id: &str) -> ApiResult<()> {
        delete_kv(env, KV_BINDING_AUTH_TOKEN, id).await
    }

    pub async fn validate(
        env: &Env,
        id: &str,
        key: String,
        after: AuthTokenAfterValidation,
    ) -> ApiResult<AuthTokenData> {
        let mut token: AuthTokenData =
            get_kv_json(env, &KV_BINDING_AUTH_TOKEN, id).await?;

        console_log!("validating token: {:?}", token);

        if token.key() != key {
            return Err("invalid key".into());
        }

        if token.expires_at() < Timestamp::now() {
            delete_kv(env, KV_BINDING_AUTH_TOKEN, id).await?;
            return Err("token expired".into());
        }

        match after {
            AuthTokenAfterValidation::Delete => {
                delete_kv(env, KV_BINDING_AUTH_TOKEN, id).await?;
            }
            AuthTokenAfterValidation::ExtendExpires(expires) => {
                token.update_expires_at(Timestamp::now() + expires);
                put_kv_json(env, KV_BINDING_AUTH_TOKEN, &id, &token).await?;
            }
        }

        Ok(token)
    }
}

fn new_id_key() -> (String, String) {
    let id = Uuid::now_v7().as_simple().to_string();
    let key = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(&OsRng.gen::<[u8; AUTH_TOKEN_KEY_LENGTH]>());

    (id, key)
}