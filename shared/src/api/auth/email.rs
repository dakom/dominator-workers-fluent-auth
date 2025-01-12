use serde::{Deserialize, Serialize};

use crate::{
    api::{ApiBoth, ApiEmpty, ApiReq}, backend::route::{AuthRoute, Route}
};
use http::Method;

use super::AuthSigninResponse;

//// Register - via email/pw
pub struct AuthRegisterEmail {}

impl ApiBoth for AuthRegisterEmail {
    const ROUTE: Route = Route::Auth(AuthRoute::RegisterEmail);
    const METHOD: Method = Method::POST;

    type Req = AuthRegisterEmailRequest;
    type Res = AuthSigninResponse;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthRegisterEmailRequest {
    pub email: String,
    pub password: String
}


//// Signin
pub struct AuthSigninEmail { }

impl ApiBoth for AuthSigninEmail {
    const ROUTE:Route = Route::Auth(AuthRoute::SigninEmail);

    type Req = AuthSigninEmailRequest;
    type Res = AuthSigninResponse;

    const METHOD: Method = Method::POST;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthSigninEmailRequest {
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
