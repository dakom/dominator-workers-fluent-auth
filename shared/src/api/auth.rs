mod email;
mod open_id;
mod password_reset;
mod token;

pub use email::*;
pub use open_id::*;
pub use password_reset::*;
pub use token::*;

use serde::{Deserialize, Serialize};

use crate::{
    backend::route::{AuthRoute, Route},
    user::UserId,
};
use http::Method;

use super::{ApiReq, ApiRes};

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthLoginResponse {
    pub uid: UserId,
    pub email_verified: bool,
    pub auth_key: String,
}

//// Signout - via User action
pub struct AuthSignout {}

impl ApiReq for AuthSignout {
    const ROUTE: Route = Route::Auth(AuthRoute::Signout);

    const METHOD: Method = Method::POST;

    type Req = AuthSignoutRequest;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthSignoutRequest {
    pub everywhere: bool,
}

//// Check
pub struct AuthCheck {}

impl ApiRes for AuthCheck {
    const ROUTE: Route = Route::Auth(AuthRoute::Check);

    type Res = AuthCheckResponse;

    const METHOD: Method = Method::POST;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthCheckResponse {
    pub uid: UserId,
}