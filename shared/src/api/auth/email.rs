use serde::{Deserialize, Serialize};

use crate::{
    api::{ApiBoth, ApiEmpty, ApiReq}, backend::route::{AuthRoute, Route}, user::UserId
};
use http::Method;

use super::AuthLoginResponse;

//// Register - via email/pw
pub struct AuthRegisterEmail {}

impl ApiBoth for AuthRegisterEmail {
    const ROUTE: Route = Route::Auth(AuthRoute::RegisterEmail);
    const METHOD: Method = Method::POST;

    type Req = AuthRegisterEmailRequest;
    type Res = AuthRegisterEmailResponse;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthRegisterEmailRequest {
    pub email: String,
    pub password: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthRegisterEmailResponse {
    pub uid: UserId,
    pub email_verified: bool,
    pub auth_key: String,
}

//// Signin
pub struct AuthLoginEmail { }

impl ApiBoth for AuthLoginEmail {
    const ROUTE:Route = Route::Auth(AuthRoute::LoginEmail);

    type Req = AuthLoginEmailRequest;
    type Res = AuthLoginResponse;

    const METHOD: Method = Method::POST;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthLoginEmailRequest {
    pub email: String,
    pub password: String,
}

//// SendVerifyEmail
pub struct AuthSendVerifyEmail {}

impl ApiEmpty for AuthSendVerifyEmail {
    const ROUTE: Route = Route::Auth(AuthRoute::SendVerifyEmail);
    const METHOD: Method = Method::POST;
}

/// Confirm email validation
pub struct AuthConfirmVerifyEmail { }
impl ApiReq for AuthConfirmVerifyEmail {
    const ROUTE: Route = Route::Auth(AuthRoute::ConfirmEmailValidation);

    type Req = AuthConfirmVerifyEmailRequest;

    const METHOD: Method = Method::POST;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthConfirmVerifyEmailRequest {
    pub oob_token_id: String,
    pub oob_token_key: String,
}
