use base64::Engine;
use sha2::{Digest, Sha256};
use shared::{api::auth::{AuthLoginEmail, AuthLoginEmailRequest, AuthLoginResponse, AuthOpenIdConnect, AuthOpenIdConnectRequest, AuthRegisterEmail, AuthRegisterEmailRequest, AuthRegisterEmailResponse, AuthSendResetPasswordAny, AuthSendResetPasswordMe, AuthSendResetPasswordRequestAny, AuthSendVerifyEmail, OpenIdProvider}, backend::result::{ApiError, ApiResult}};
use argon2::{
    password_hash::{
        PasswordHasher, SaltString
    },
    Argon2
};

use crate::prelude::*;

// this is used on the root page
pub async fn send_email_validation() -> ApiResult<()> {
    AuthSendVerifyEmail::fetch().await
}

pub(super) async fn register_email(email: &str, password: &str) -> ApiResult<()> {
    let password = hash_password(email, password).map_err(|err| ApiError::Unknown(err.to_string()))?;

    let AuthRegisterEmailResponse{uid, email_verified, auth_key} = AuthRegisterEmail::fetch(AuthRegisterEmailRequest { email: email.to_string(), password }).await?;

    AUTH.on_login(uid, email_verified, auth_key).await
}

pub(super) async fn login_email(email: &str, password: &str) -> ApiResult<()> {
    let password = hash_password(email, password).map_err(|err| ApiError::Unknown(err.to_string()))?;

    let AuthLoginResponse{uid, email_verified, auth_key} = AuthLoginEmail::fetch(AuthLoginEmailRequest{ email: email.to_string(), password }).await?;

    AUTH.on_login(uid, email_verified, auth_key).await
}

pub(super) async fn openid_connect(provider: OpenIdProvider) -> ApiResult<()> {
    let res = AuthOpenIdConnect::fetch(AuthOpenIdConnectRequest{provider}).await?;

    web_sys::window().unwrap_ext().location().replace(&res.url).unwrap_ext();

    Ok(())
}

pub(super) async fn send_password_reset(email: Option<&str>) -> ApiResult<()> {
    match email {
        Some(email) => {
            AuthSendResetPasswordAny::fetch(AuthSendResetPasswordRequestAny { email: email.to_string() }).await
        },
        None => {
            AuthSendResetPasswordMe::fetch().await
        }
    }
}



fn hash_password(email: &str, password: &str) -> Result<String> {
    // salt is composed of email (unique to this record) and global salt
    // idea is this makes it generally globally unique across the internet
    // it does not need to be secret, just unique enough to not match some other product's breach
    // by deriving it from known values, the client doesn't need to ask the server for the salt value
    // and it's maybe a little bit of an extra protection that the attacker needs to know the email address too
    // if the user changes their email, they'll need to reset their password too - which is likely a good thing
    // however, argon2 salts shouldn't be larger than 64 bytes, so we hash the salt itself to get a sha256 hash
    let salt = [email.as_bytes(), CONFIG.argon2_global_salt].concat();
    let salt = Sha256::digest(&salt);
    let salt = SaltString::encode_b64(&salt).map_err(|err| anyhow!("{:?}", err))?;

    // derive the argon2 hash, which takes some time to compute from the user's password
    // this makes it much harder to brute force the password, even if the database is breached
    // it's computed clientside to avoid denial-of-service attacks on the server
    // and there's simply no need for the server to know the real password
    // on the server, it will be hashed again but with a simpler sha256 hash merely to avoid data breaches
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt).map_err(|err| anyhow!("{:?}", err))?;
    let hash = hash.hash.expect("hash should be present");

    // now encode this hash into a string that can be sent over the wire and decoded serverside 
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&hash.as_bytes()))

}
