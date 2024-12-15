use dominator::clone;
use dominator_helpers::futures::AsyncLoader;
use futures_signals::signal::Mutable;
use shared::{
    api::auth::{AuthCheck, AuthSignout, AuthSignoutRequest},
    backend::result::ApiResult,
    user::UserId,
};
use std::sync::{Arc, RwLock};

use crate::{page::landing::auth::actions::send_email_validation, prelude::*};

/////// This is a global singleton that holds the current auth state
pub static AUTH: LazyLock<Auth> = LazyLock::new(|| {
    let phase = Mutable::new(AuthPhase::Init);
    let loader = Arc::new(AsyncLoader::new());
    let token_key = Arc::new(RwLock::new(None));
    let uid = Arc::new(RwLock::new(None));

    let _auth = Auth {
        phase,
        loader,
        token_key,
        uid,
    };

    // since the AuthPhase starts as Init, the page won't actually show anything useful until this resolves
    // the gating for that is in route
    _auth
        .loader
        .load(clone!(_auth => async move {_auth.check().await}));
    _auth
});

#[derive(Clone)]
pub struct Auth {
    pub phase: Mutable<AuthPhase>,
    pub token_key: Arc<RwLock<Option<String>>>,
    pub uid: Arc<RwLock<Option<UserId>>>,
    loader: Arc<AsyncLoader>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AuthPhase {
    Init,
    Authenticated,
    Unauthenticated,
    EmailNotVerified
}

impl Auth {
    pub fn clear(&self) {
        *self.token_key.write().unwrap() = None;
        *self.uid.write().unwrap() = None;
        web_sys::window()
            .unwrap_ext()
            .local_storage()
            .unwrap_ext()
            .unwrap_ext()
            .delete(CONFIG.auth_login_key_storage_name)
            .unwrap_ext();
        self.phase.set_neq(AuthPhase::Unauthenticated);
    }

    pub async fn signout(&self, everywhere: bool) -> ApiResult<()> {
        AuthSignout::fetch(AuthSignoutRequest { everywhere }).await?;
        self.clear();
        Ok(())
    }

    pub fn try_clone_uid(&self) -> Option<UserId> {
        self.uid.read().unwrap().clone()
    }
    pub fn try_clone_token_key(&self) -> Option<String> {
        self.token_key.read().unwrap().clone()
    }

    pub async fn on_login(&self, uid: UserId, email_verified: bool, auth_key: String) -> ApiResult<()> {
        web_sys::window()
            .unwrap_ext()
            .local_storage()
            .unwrap_ext()
            .unwrap_ext()
            .set_item(CONFIG.auth_login_key_storage_name, &auth_key)
            .unwrap_ext();
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
            }
            Err(err) => {
                tracing::error!("auth check failed: {:?}", err);
                self.phase.set_neq(AuthPhase::Unauthenticated);
            }
        }
    }
}
