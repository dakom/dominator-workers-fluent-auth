mod openid;
mod password_reset;
mod register;
mod signin;
mod verify_email;

use std::sync::atomic::AtomicBool;

use dominator::text_signal;
use dominator_helpers::futures::AsyncLoader;
use futures_signals::signal::{option, OptionSignal};
use sha2::{Digest, Sha256};
use base64::engine::Engine;
use rand::Rng;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use shared::{
    api::auth::{AuthCheck, AuthCheckResetPassword, AuthCheckResetPasswordRequest, AuthCheckResetPasswordResponse, AuthConfirmResetPassword, AuthConfirmResetPasswordRequest, AuthConfirmResetPasswordResponse, AuthConfirmVerifyEmail, AuthConfirmVerifyEmailRequest, AuthOpenIdConnect, AuthOpenIdConnectRequest, AuthOpenIdFinalizeExec, AuthOpenIdFinalizeExecResponse, AuthOpenIdFinalizeQuery, AuthOpenIdFinalizeQueryResponse, AuthOpenIdFinalizeRequest, AuthRegister, AuthRegisterRequest, AuthRegisterResponse, AuthSendResetPasswordAny, AuthSendResetPasswordMe, AuthSendResetPasswordRequestAny, AuthSendVerifyEmail, AuthSignin, AuthSigninRequest, AuthSigninResponse, AuthSignout}, auth::FRONTEND_ROUTE_AFTER_SIGNIN, backend::{
        result::{ApiError, ApiResult, AuthError}, 
        route::{AuthRoute as ApiAuthRoute, OpenIdProvider, Route as ApiRoute}
    }, user::UserId
};

use signin::Signin;
use register::Register;
use verify_email::{VerifyEmailWaiting, VerifyEmailConfirm};
use password_reset::VerifyPasswordResetConfirm;
use openid::OpenIdFinalize;

use crate::{prelude::*, atoms::input::TextInput};

pub fn render(auth_route: AuthRoute) -> Dom {
    match auth_route {
        AuthRoute::Signin => {
            Signin::new().render()
        },
        AuthRoute::Register => {
            Register::new().render()
        },
        AuthRoute::VerifyEmailWaiting => {
            VerifyEmailWaiting::new().render()
        },
        AuthRoute::VerifyEmailConfirm {oob_token_id, oob_token_key} => {
            VerifyEmailConfirm::new(oob_token_id.clone(), oob_token_key.to_string()).render()
        },
        AuthRoute::PasswordResetConfirm { oob_token_id, oob_token_key} => {
            VerifyPasswordResetConfirm::new(oob_token_id.clone(), oob_token_key.to_string()).render()
        },
        AuthRoute::OpenIdFinalize{ session_id, session_key} => {
            OpenIdFinalize::new(session_id.clone(), session_key.to_string()).render()
        },
    }
}

pub(super) fn hash_password(email: &str, password: &str) -> Result<String> {
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

/////// Api calls

pub(super) async fn openid_connect(provider: OpenIdProvider) -> ApiResult<()> {
    let res = AuthOpenIdConnect::fetch(AuthOpenIdConnectRequest{provider}).await?;

    web_sys::window().unwrap_ext().location().replace(&res.url).unwrap_ext();

    Ok(())
}
pub(super) async fn signin(email: &str, password: &str) -> ApiResult<()> {
    let password = hash_password(email, password).map_err(|err| ApiError::Unknown(err.to_string()))?;

    let AuthSigninResponse{uid, email_verified, auth_key} = AuthSignin::fetch(AuthSigninRequest { email: email.to_string(), password }).await?;


    AUTH.on_signin(uid, email_verified, auth_key).await
}

pub(super) async fn register(email: &str, password: &str) -> ApiResult<()> {
    let password = hash_password(email, password).map_err(|err| ApiError::Unknown(err.to_string()))?;

    let AuthRegisterResponse{uid, email_verified, auth_key} = AuthRegister::fetch(AuthRegisterRequest { email: email.to_string(), password }).await?;

    AUTH.on_signin(uid, email_verified, auth_key).await?;
    FRONTEND_ROUTE_AFTER_SIGNIN.go_to_url();

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

pub(super) async fn check_password_reset(oob_token_id: String, oob_token_key: String) -> ApiResult<AuthCheckResetPasswordResponse> {
    AuthCheckResetPassword::fetch(AuthCheckResetPasswordRequest{ oob_token_id, oob_token_key }).await
}

pub(super) async fn confirm_password_reset(oob_token_id: String, oob_token_key: String, email: &str, password: &str) -> ApiResult<()> {
    let password = hash_password(email, password).map_err(|err| ApiError::Unknown(err.to_string()))?;

    let res = AuthConfirmResetPassword::fetch(AuthConfirmResetPasswordRequest{ oob_token_id, oob_token_key, password }).await?;
    let AuthConfirmResetPasswordResponse{uid, email_verified, auth_key} = res;

    AUTH.on_signin(uid, email_verified, auth_key).await?;
    FRONTEND_ROUTE_AFTER_SIGNIN.go_to_url();

    Ok(())
}

// this is used on the root page
pub async fn send_email_validation() -> ApiResult<()> {
    AuthSendVerifyEmail::fetch().await
}

pub(super) async fn confirm_email_validation(oob_token_id: String, oob_token_key: String) -> ApiResult<()> {
    AuthConfirmVerifyEmail::fetch(AuthConfirmVerifyEmailRequest { oob_token_id, oob_token_key }).await;

    AUTH.check().await;
    FRONTEND_ROUTE_AFTER_SIGNIN.go_to_url();

    Ok(())
}

pub(super) async fn openid_session_query(session_id: String, session_key: String) -> ApiResult<AuthOpenIdFinalizeQueryResponse> {
    AuthOpenIdFinalizeQuery::fetch(AuthOpenIdFinalizeRequest{ session_id, session_key}).await
}

pub(super) async fn openid_session_finalize(session_id: String, session_key: String) -> ApiResult<()> {
    let res = AuthOpenIdFinalizeExec::fetch(AuthOpenIdFinalizeRequest{ session_id, session_key}).await?;
    let AuthOpenIdFinalizeExecResponse{uid, email_verified, auth_key} = res;

    AUTH.on_signin(uid, email_verified, auth_key).await?;
    FRONTEND_ROUTE_AFTER_SIGNIN.go_to_url();

    Ok(())
}