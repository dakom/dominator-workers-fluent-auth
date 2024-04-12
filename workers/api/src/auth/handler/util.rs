use base64::Engine;
use rand::Rng;
use sha2::{Digest, Sha256};
use shared::auth::AUTH_TOKEN_ID_NAME;
use crate::{db::user::UserAccount, prelude::*};

use super::super::durable_objects::token::{AuthTokenAfterValidation, AuthTokenDO, AuthTokenKind, AuthTokenValidateResponse};

// the password was sent as an argon2 hash from the client
// but we must hash it again, otherwise that might as well just be plaintext
// if the db is compromised (user can just send the db value for comparison)
// however, we don't need a compute-intensive hash here, since the
// plaintext isn't the original password, it's the argon2 output bytes
// and so an attacker would need to brute force sha256 guesses against the argon2 output space
// for simplicity, the salt is stored alongside the password
pub fn hash_password(password:&str, salt: Option<&[u8]>) -> ApiResult<String> {
    let password = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(password).map_err(|err| ApiError::from(err.to_string()))?;
    let salt = match salt {
        Some(salt) => salt.try_into().map_err(|_| ApiError::from("salt must be 32 bytes".to_string()))?,
        None => {
            rand::thread_rng().gen::<[u8; 32]>()
        }
    };
    let msg = [salt.as_slice(), password.as_slice()].concat(); 
    let hash = Sha256::digest(msg);
    let password = [salt.as_slice(), hash.as_slice()].concat();
    let password = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&password);
    Ok(password)
}

#[cfg(debug_assertions)]
pub fn set_signin_cookie(res: &Response, auth_token_id: &str) {

    res.headers().set("Set-Cookie", &format!("{AUTH_TOKEN_ID_NAME}={auth_token_id}; Path=/; HttpOnly; Secure; SameSite=None; Partitioned; Expires=Tue, 19 Jan 2038 03:14:07 GMT")).unwrap();
}
#[cfg(not(debug_assertions))]
pub fn set_signin_cookie(res: &Response, auth_token_id: &str) {
    res.headers().set("Set-Cookie", &format!("{AUTH_TOKEN_ID_NAME}={auth_token_id}; Path=/; HttpOnly; Secure; SameSite=Strict; Partitioned; Expires=Tue, 19 Jan 2038 03:14:07 GMT")).unwrap();
}

pub fn delete_signin_cookie(res: &Response) {
    res.headers().set("Set-Cookie", &format!("{AUTH_TOKEN_ID_NAME}=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT")).unwrap();
}

pub async fn validate_oob_token(env: &Env, kind: AuthTokenKind, oob_token_id: String, oob_token_key: String, after_validate: AuthTokenAfterValidation) -> ApiResult<UserAccount> {
    let AuthTokenValidateResponse {uid, user_token} = AuthTokenDO::validate(env, kind, &oob_token_id, oob_token_key, after_validate).await?;

    let account = UserAccount::load_by_id(env, &uid).await?;
    if account.user_token != user_token {
        return Err(format!("user token mismatch for user id {uid}").into())
    }
    Ok(account)
}
