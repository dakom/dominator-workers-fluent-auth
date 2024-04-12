use std::sync::{Arc, RwLock};
use awsm_web::loaders::helpers::AsyncLoader;
use base64::engine::Engine;
use rand::Rng;
use sha2::{Sha256, Digest};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

use dominator::clone;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use once_cell::sync::Lazy;
use serde::Deserialize;
use shared::{
    api::auth::{AuthCheck, AuthCheckResetPassword, AuthCheckResetPasswordRequest, AuthCheckResetPasswordResponse, AuthConfirmResetPassword, AuthConfirmResetPasswordRequest, AuthConfirmResetPasswordResponse, AuthConfirmVerifyEmail, AuthConfirmVerifyEmailRequest, AuthOpenIdConnect, AuthOpenIdConnectRequest, AuthRegister, AuthRegisterRequest, AuthRegisterResponse, AuthSendResetPasswordAny, AuthSendResetPasswordMe, AuthSendResetPasswordRequestAny, AuthSendVerifyEmail, AuthSignin, AuthSigninRequest, AuthSigninResponse, AuthSignout}, auth::FRONTEND_ROUTE_AFTER_SIGNIN, backend::{
        result::{ApiError, ApiResult, AuthError}, 
        route::{AuthRoute as ApiAuthRoute, OpenIdProvider, Route as ApiRoute}
    }, user::UserId
};

use crate::{page::landing::auth::send_email_validation, prelude::*};


/////// This is a global singleton that holds the current auth state 
pub static AUTH:Lazy<Auth> = Lazy::new(|| {
    let phase = Mutable::new(AuthPhase::Init);
    let loader = Arc::new(AsyncLoader::new());
    let token_key = Arc::new(RwLock::new(None));
    let uid = Arc::new(RwLock::new(None));

    let _auth = Auth{phase, loader, token_key, uid};

    _auth.loader.load(clone!(_auth => async move {_auth.check().await}));
    _auth
});

#[derive(Clone)]
pub struct Auth {
    pub phase: Mutable<AuthPhase>,
    pub token_key: Arc<RwLock<Option<String>>>,
    pub uid: Arc<RwLock<Option<UserId>>>,
    loader: Arc<AsyncLoader>
}

#[derive(Clone, PartialEq, Debug)]
pub enum AuthPhase {
    Init,
    Authenticated,
    EmailNotVerified,
    Unauthenticated,
}

impl Auth {
    pub fn clear(&self) {
        *self.token_key.write().unwrap() = None;
        *self.uid.write().unwrap() = None;
        self.phase.set_neq(AuthPhase::Unauthenticated);
        let _ = web_sys::window().unwrap_ext().local_storage().unwrap_ext().unwrap_ext().delete(CONFIG.auth_signin_key_storage_name);
    }

    pub async fn signout(&self) -> ApiResult<()> {
        AuthSignout::fetch().await?;
        self.clear();
        Ok(())
    }

    pub fn try_clone_uid(&self) -> Option<UserId>
    {
        self.uid.read().unwrap().clone()
    }
    pub fn try_clone_token_key(&self) -> Option<String>
    {
        self.token_key.read().unwrap().clone()
    }

    pub async fn on_signin(&self, uid: UserId, email_verified: bool, auth_key: String) -> ApiResult<()> {
        web_sys::window().unwrap_ext().local_storage().unwrap_ext().unwrap_ext().set_item(CONFIG.auth_signin_key_storage_name, &auth_key).unwrap_ext();
        *self.uid.write().unwrap() = Some(uid);
        *self.token_key.write().unwrap() = Some(auth_key);


        if !email_verified {
            send_email_validation().await?;
            self.phase.set_neq(AuthPhase::EmailNotVerified);
        } else {
            self.phase.set_neq(AuthPhase::Authenticated);
        }

        Ok(())
    }

    pub async fn check(&self) {
        let res = AuthCheck::fetch().await;
        match res {
            Ok(res) => {
                *self.uid.write().unwrap() = Some(res.uid);
                self.phase.set_neq(AuthPhase::Authenticated);
            },
            Err(err) => {
                match err {
                    // if email is not verified, we need to show a different screen
                    // and it's nicer to not wait until an api call from *within* a protected screen
                    // since it will avoid a flash of content
                    // so we immediately set the phase to EmailNotVerified here
                    // but if an API call happens to be made somehow, it will be protected
                    ApiError::Auth(AuthError::EmailNotVerified) => {
                        self.phase.set_neq(AuthPhase::EmailNotVerified);
                    },
                    _ => {
                        log::error!("{:?}", err);
                        self.phase.set_neq(AuthPhase::Unauthenticated);
                    }
                }
            }
        }
    }
}