use serde::{Deserialize, Serialize};

use crate::{backend::route::{AuthRoute, OpenIdProvider, Route}, user::UserId};

use super::{ApiBoth, ApiEmpty, ApiEmptyDynRoute, ApiReq, ApiRes, Method};

//// Signin
pub struct AuthSignin { }

impl ApiBoth for AuthSignin {
    const ROUTE:Route = Route::Auth(AuthRoute::Signin);

    type Req = AuthSigninRequest;
    type Res = AuthSigninResponse;

    const METHOD: Method = Method::Post;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthSigninRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthSigninResponse {
    pub uid: UserId,
    pub email_verified: bool,
    pub auth_key: String,
}

//// Signin
pub struct AuthSignout { }

impl ApiEmpty for AuthSignout {
    const ROUTE:Route = Route::Auth(AuthRoute::Signout);

    const METHOD: Method = Method::Post;
}

//// Register
pub struct AuthRegister { }

impl ApiBoth for AuthRegister {
    const ROUTE: Route = Route::Auth(AuthRoute::Register);

    type Req = AuthRegisterRequest;
    type Res = AuthRegisterResponse;

    const METHOD: Method = Method::Post;
}
#[derive(Deserialize, Serialize, Debug)]
pub struct AuthRegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthRegisterResponse {
    pub uid: UserId,
    pub email_verified: bool,
    pub auth_key: String,
} 

//// Check
pub struct AuthCheck { }

impl ApiRes for AuthCheck {
    const ROUTE: Route = Route::Auth(AuthRoute::Check);

    type Res = AuthCheckResponse;

    const METHOD: Method = Method::Post;
}
#[derive(Deserialize, Serialize, Debug)]
pub struct AuthCheckResponse{
    pub uid: UserId,
}

/// (re) Send email validation
pub struct AuthSendVerifyEmail { }

impl ApiEmpty for AuthSendVerifyEmail {
    const ROUTE: Route = Route::Auth(AuthRoute::SendEmailValidation);

    const METHOD: Method = Method::Post;
}

/// Confirm email validation
pub struct AuthConfirmVerifyEmail { }
impl ApiReq for AuthConfirmVerifyEmail {
    const ROUTE: Route = Route::Auth(AuthRoute::ConfirmEmailValidation);

    type Req = AuthConfirmVerifyEmailRequest;

    const METHOD: Method = Method::Post;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthConfirmVerifyEmailRequest {
    pub oob_token_id: String,
    pub oob_token_key: String,
}

/// Send password reset
pub struct AuthSendResetPasswordAny { }
impl ApiReq for AuthSendResetPasswordAny {
    const ROUTE: Route = Route::Auth(AuthRoute::SendPasswordResetAny);

    type Req = AuthSendResetPasswordRequestAny;

    const METHOD: Method = Method::Post;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthSendResetPasswordRequestAny { 
    pub email: String
}

pub struct AuthSendResetPasswordMe { }
impl ApiEmpty for AuthSendResetPasswordMe {
    const ROUTE: Route = Route::Auth(AuthRoute::SendPasswordResetMe);

    const METHOD: Method = Method::Post;
}

/// Confirm password reset
pub struct AuthConfirmResetPassword { }
impl ApiBoth for AuthConfirmResetPassword {
    const ROUTE: Route = Route::Auth(AuthRoute::ConfirmPasswordReset);

    type Req = AuthConfirmResetPasswordRequest;
    type Res = AuthConfirmResetPasswordResponse;

    const METHOD: Method = Method::Post;
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

/// Check password reset
pub struct AuthCheckResetPassword { }
impl ApiBoth for AuthCheckResetPassword {
    const ROUTE: Route = Route::Auth(AuthRoute::CheckPasswordReset);

    type Req = AuthCheckResetPasswordRequest;
    type Res = AuthCheckResetPasswordResponse;

    const METHOD: Method = Method::Post;
}
#[derive(Deserialize, Serialize, Debug)]
pub struct AuthCheckResetPasswordRequest {
    pub oob_token_id: String,
    pub oob_token_key: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct AuthCheckResetPasswordResponse {
    pub uid: UserId,
    pub email: String,
}

/// OpenId Connect
pub struct AuthOpenIdConnect {}
impl ApiBoth for AuthOpenIdConnect {
    const ROUTE: Route = Route::Auth(AuthRoute::OpenIdConnect);

    type Req = AuthOpenIdConnectRequest;
    type Res = AuthOpenIdConnectResponse;

    const METHOD: Method = Method::Post;
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

    const METHOD: Method = Method::Post;
}


/// OpenId Finalize Exec
pub struct AuthOpenIdFinalizeExec { }
impl ApiBoth for AuthOpenIdFinalizeExec {
    const ROUTE: Route = Route::Auth(AuthRoute::OpenIdFinalizeExec);

    type Req = AuthOpenIdFinalizeRequest;
    type Res = AuthOpenIdFinalizeExecResponse;

    const METHOD: Method = Method::Post;
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

    const METHOD: Method = Method::Post;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthOpenIdFinalizeQueryResponse {
    pub email: String,
    pub user_exists: bool,
} 
