use serde::{Deserialize, Serialize};

use crate::{
    api::{ApiBoth, ApiEmpty, ApiReq},
    backend::route::{AuthRoute, Route},
    user::UserId,
};
use http::Method;

/// Send password reset
pub struct AuthSendResetPasswordAny {}
impl ApiReq for AuthSendResetPasswordAny {
    const ROUTE: Route = Route::Auth(AuthRoute::SendPasswordResetAny);

    type Req = AuthSendResetPasswordRequestAny;

    const METHOD: Method = Method::POST;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthSendResetPasswordRequestAny {
    pub email: String,
}

pub struct AuthSendResetPasswordMe {}
impl ApiEmpty for AuthSendResetPasswordMe {
    const ROUTE: Route = Route::Auth(AuthRoute::SendPasswordResetMe);

    const METHOD: Method = Method::POST;
}

/// Confirm password reset
pub struct AuthConfirmResetPassword {}
impl ApiBoth for AuthConfirmResetPassword {
    const ROUTE: Route = Route::Auth(AuthRoute::ConfirmPasswordReset);

    type Req = AuthConfirmResetPasswordRequest;
    type Res = AuthConfirmResetPasswordResponse;

    const METHOD: Method = Method::POST;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthConfirmResetPasswordRequest {
    pub oob_token_id: String,
    pub oob_token_key: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthConfirmResetPasswordResponse {
    pub uid: UserId,
    pub email_verified: bool,
    pub auth_key: String,
}
