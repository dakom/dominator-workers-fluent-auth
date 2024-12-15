use serde::{Deserialize, Serialize};

use crate::{
    api::{ApiBoth, ApiEmptyDynRoute}, backend::route::{AuthRoute, Route}, user::UserId
};
use http::Method;

/// OpenId Connect
pub struct AuthOpenIdConnect {}
impl ApiBoth for AuthOpenIdConnect {
    const ROUTE: Route = Route::Auth(AuthRoute::OpenIdConnect);

    type Req = AuthOpenIdConnectRequest;
    type Res = AuthOpenIdConnectResponse;

    const METHOD: Method = Method::POST;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthOpenIdConnectRequest {
    pub provider: OpenIdProvider,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthOpenIdConnectResponse {
    pub url: String,
}

/// OpenId Access Token hook
pub struct AuthOpenIdAccessTokenHook {
    pub provider: OpenIdProvider,
}
impl ApiEmptyDynRoute for AuthOpenIdAccessTokenHook {
    fn route(&self) -> Route {
        Route::Auth(AuthRoute::OpenIdAccessTokenHook(self.provider))
    }

    const METHOD: Method = Method::POST;
}


/// OpenId Finalize Exec
pub struct AuthOpenIdFinalizeExec { }
impl ApiBoth for AuthOpenIdFinalizeExec {
    const ROUTE: Route = Route::Auth(AuthRoute::OpenIdFinalizeExec);

    type Req = AuthOpenIdFinalizeRequest;
    type Res = AuthOpenIdFinalizeExecResponse;

    const METHOD: Method = Method::POST;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthOpenIdFinalizeRequest {
    pub session_id: String,
    pub session_key: String,
} 

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthOpenIdFinalizeExecResponse {
    pub uid: UserId,
    pub email_verified: bool,
    pub auth_key: String,
} 

/// OpenId Finalize Exec
pub struct AuthOpenIdFinalizeQuery { }
impl ApiBoth for AuthOpenIdFinalizeQuery {
    const ROUTE: Route = Route::Auth(AuthRoute::OpenIdFinalizeQuery);

    type Req = AuthOpenIdFinalizeRequest;
    type Res = AuthOpenIdFinalizeQueryResponse;

    const METHOD: Method = Method::POST;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthOpenIdFinalizeQueryResponse {
    pub email: String,
    pub user_exists: bool,
} 


#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum OpenIdProvider {
    Google,
    Facebook
}

impl OpenIdProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Facebook => "facebook",
        }
    }

    pub fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "google" => Some(Self::Google),
            "facebook" => Some(Self::Facebook),
            _ => None
        }
    }
}
