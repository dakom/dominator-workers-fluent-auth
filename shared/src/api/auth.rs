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
pub struct AuthSigninResponse {
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
    pub roles: Vec<UserRole>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum UserRole {
    EmailVerified,
    Admin
}

impl From<u8> for UserRole {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::EmailVerified,
            1 => Self::Admin,
            _ => panic!("invalid user role"),
        }
    }
}

impl From<UserRole> for u8 {
    fn from(v: UserRole) -> Self {
        v as u8
    }
}