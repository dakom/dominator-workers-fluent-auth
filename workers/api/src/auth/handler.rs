mod openid;
mod util;

use async_trait::async_trait;
use base64::Engine;
use rand::Rng;
use shared::{api::{auth::{AuthCheck, AuthCheckResetPassword, AuthCheckResetPasswordRequest, AuthCheckResetPasswordResponse, AuthCheckResponse, AuthConfirmResetPassword, AuthConfirmResetPasswordRequest, AuthConfirmResetPasswordResponse, AuthConfirmVerifyEmail, AuthConfirmVerifyEmailRequest, AuthOpenIdAccessTokenHook, AuthOpenIdConnect, AuthOpenIdConnectRequest, AuthOpenIdConnectResponse, AuthOpenIdFinalizeExec, AuthOpenIdFinalizeExecResponse, AuthOpenIdFinalizeQuery, AuthOpenIdFinalizeQueryResponse, AuthOpenIdFinalizeRequest, AuthRegister, AuthRegisterRequest, AuthRegisterResponse, AuthSendResetPasswordAny, AuthSendResetPasswordMe, AuthSendResetPasswordRequestAny, AuthSendVerifyEmail, AuthSignin, AuthSigninRequest, AuthSigninResponse, AuthSignout}, ApiBoth, ApiReq, ApiRes}, backend::{result::{ApiError, ApiResult, AuthError}, worker::ResponseExt}, frontend::route::NotFoundReason as FrontendNotFoundReason, user::UserId};
use web_sys::Response;
use crate::{
    api_ext::{ApiBothExt, ApiBothWithExtraExt, ApiEmptyDynRouteWithExtraExt, ApiEmptyExt, ApiReqExt, ApiResExt}, auth::{durable_objects::token::{AuthTokenDO, AuthTokenKind}, handler::util::hash_password}, config::{AUTH_RESET_PASSWORD_TOKEN_EXPIRES, AUTH_SIGNIN_TOKEN_EXPIRES, AUTH_VERIFY_EMAIL_TOKEN_EXPIRES, FRONTEND_DOMAIN, FRONTEND_ROOT_PATH, OAUTH_REGISTER_PASSWORD_LENGTH}, db::user::UserAccount, mailer::{self, MailerKind}, ApiContext
};
use self::{openid::OpenIdProcessor, util::{delete_signin_cookie, set_signin_cookie, validate_oob_token}};
use super::durable_objects::{openid::{OpenIdSession, OpenIdSessionDO, OpenIdSessionFinalizeInfo}, token::{AuthTokenAfterValidation, AuthTokenCreateResponse}};
use shared::frontend::route::{Route as FrontendRoute, Landing as FrontendLanding, AuthRoute as FrontendAuthRoute};


#[async_trait(?Send)]
impl ApiBothWithExtraExt for AuthSignin {
    type Req = <AuthSignin as ApiBoth>::Req;
    type Res = <AuthSignin as ApiBoth>::Res;
    type Extra = AuthTokenCreateResponse;

    async fn handle(ctx: &ApiContext, data: AuthSigninRequest) -> ApiResult<(Self::Res, Self::Extra)> {
        async fn inner(ctx: &ApiContext, data: AuthSigninRequest) -> ApiResult<(AuthSigninResponse, AuthTokenCreateResponse)> {
            let AuthSigninRequest { email, password } = data;
            let user = UserAccount::load_by_email(&ctx.env, &email).await?;

            // see registration, this is *not* the user's plaintext password, it's just the argon2 output hash
            // we need to get the salt from the db and hash it again for comparison, however
            let db_salt = &base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(&user.password)
                .map_err(|err| ApiError::from(err.to_string()))?
                [0..32];

            let req_password = hash_password(&password, Some(db_salt))?;

            // see if they match
            if user.password != req_password {
                return Err("mismatched password".into())
            }

            // sign the user in and return
            let uid = user.id;
            let auth_token = AuthTokenDO::create(&ctx.env, AuthTokenKind::Signin, uid.clone(), user.user_token, AUTH_SIGNIN_TOKEN_EXPIRES).await?;
            let auth_key = auth_token.key.clone();
            Ok((AuthSigninResponse{
                uid,
                email_verified: user.email_verified,
                auth_key
            }, auth_token))
        }

        inner(ctx, data).await.map_err(|_| {
            // can log the specific error here, but clients only see InvalidSignin
            // to avoid leaking semi-sensitive info (like who has an account etc.)
            AuthError::InvalidSignin.into()
        })
    }

    fn response(_ctx: &ApiContext, data: AuthSigninResponse, auth_token: AuthTokenCreateResponse) -> Response {
        let res = Response::new_json(&data);
        set_signin_cookie(&res, &auth_token.id);
        res
    }
}

#[async_trait(?Send)]
impl ApiBothWithExtraExt for AuthRegister {
    type Req = <AuthRegister as ApiBoth>::Req;
    type Res = <AuthRegister as ApiBoth>::Res;
    type Extra = AuthTokenCreateResponse;

    async fn handle(ctx: &ApiContext, data: AuthRegisterRequest) -> ApiResult<(Self::Res, Self::Extra)> {
        let AuthRegisterRequest {email, password} = data;

        if UserAccount::exists_by_email(&ctx.env, &email).await? {
            return Err(AuthError::EmailAlreadyExists.into())
        }

        let password = hash_password(&password, None)?;

        // create a new user account
        let uid = UserId::new(uuid::Uuid::now_v7());
        let user_token = uuid::Uuid::now_v7().as_simple().to_string();
        UserAccount::insert(&ctx.env, &uid, &password, &email, &user_token).await?;

        // sign the user in and return
        let auth_token = AuthTokenDO::create(&ctx.env, AuthTokenKind::Signin, uid.clone(), user_token.clone(), AUTH_SIGNIN_TOKEN_EXPIRES).await?;

        mailer::send(&ctx, &email, MailerKind::EmailVerification { 
            oob_token_id: auth_token.id.clone(), 
            oob_token_key: auth_token.key.clone()
        }).await?;

        let auth_key = auth_token.key.clone();
        Ok((AuthRegisterResponse{
            uid,
            email_verified: false,
            auth_key
        }, auth_token))
    }

    fn response(_ctx: &ApiContext, data: AuthRegisterResponse, auth_token: AuthTokenCreateResponse) -> Response {
        let res = Response::new_json(&data);
        set_signin_cookie(&res, &auth_token.id);
        res
    }
}


#[async_trait(?Send)]
impl ApiResExt for AuthCheck {
    type Res = <AuthCheck as ApiRes>::Res;

    async fn handle(ctx: &ApiContext) -> ApiResult<Self::Res> {
        let uid = ctx.uid_unchecked();
        Ok(AuthCheckResponse {
            uid
        })
    }
}


#[async_trait(?Send)]
impl ApiBothExt for AuthOpenIdConnect {
    type Req = <AuthOpenIdConnect as ApiBoth>::Req;
    type Res = <AuthOpenIdConnect as ApiBoth>::Res;

    async fn handle(ctx: &ApiContext, data: AuthOpenIdConnectRequest) -> ApiResult<Self::Res> {
        let url = OpenIdProcessor::new(data.provider).get_auth_url(&ctx.env).await?;
        Ok(AuthOpenIdConnectResponse{url})
    }
}

#[async_trait(?Send)]
impl ApiEmptyDynRouteWithExtraExt for AuthOpenIdAccessTokenHook {
    // the second param is for whether a user exists
    type Extra = ApiResult<OpenIdSession>;

    async fn handle(&self, ctx: &ApiContext) -> ApiResult<Self::Extra> {
        let url = web_sys::Url::new(&ctx.req.url())?;
        let search_params = url.search_params();

        let processor = OpenIdProcessor::new(self.provider);
        match (search_params.get("code"), search_params.get("state")) {
            (Some(code), Some(state)) => {
                let (session, _) = processor.validate_token_claims(ctx, code, state).await?;
                Ok(Ok(session))
            }
            _ => {
                Ok(Err("missing code or state".into()))
            }
        }
    }

    fn response(&self, _ctx: &ApiContext, session: ApiResult<OpenIdSession>) -> Response {
        let frontend_route = match session {
            Ok(session) => {
                FrontendRoute::Landing(FrontendLanding::Auth(FrontendAuthRoute::OpenIdFinalize{
                    session_id: session.id,
                    session_key: session.key
                }))
            },
            Err(_) => {
                FrontendRoute::NotFound(FrontendNotFoundReason::NoAuth)
            }
        };

        worker::console_log!("redirecting to {:?}", frontend_route.link(FRONTEND_DOMAIN, FRONTEND_ROOT_PATH));

        Response::new_temp_redirect(&frontend_route.link(FRONTEND_DOMAIN, FRONTEND_ROOT_PATH))
    }
}

#[async_trait(?Send)]
impl ApiBothExt for AuthOpenIdFinalizeQuery {
    type Req = <AuthOpenIdFinalizeQuery as ApiBoth>::Req;
    type Res = <AuthOpenIdFinalizeQuery as ApiBoth>::Res;

    async fn handle(ctx: &ApiContext, data: AuthOpenIdFinalizeRequest) -> ApiResult<AuthOpenIdFinalizeQueryResponse> {
        let AuthOpenIdFinalizeRequest{session_id, session_key} = data;
        let session = OpenIdSession{id: session_id, key: session_key};

        let OpenIdSessionFinalizeInfo{ email, .. } = OpenIdSessionDO::finalize_query(&ctx.env, session.clone()).await?; 
        let user_exists = UserAccount::load_by_email(&ctx.env, &email).await.is_ok();

        Ok(AuthOpenIdFinalizeQueryResponse{
            email,
            user_exists
        })
    }

}

#[async_trait(?Send)]
impl ApiBothWithExtraExt for AuthOpenIdFinalizeExec {
    type Req = <AuthOpenIdFinalizeExec as ApiBoth>::Req;
    type Res = <AuthOpenIdFinalizeExec as ApiBoth>::Res;
    type Extra = AuthTokenCreateResponse;

    async fn handle(ctx: &ApiContext, data: AuthOpenIdFinalizeRequest) -> ApiResult<(AuthOpenIdFinalizeExecResponse, AuthTokenCreateResponse)> {
        let AuthOpenIdFinalizeRequest{session_id, session_key} = data;
        let session = OpenIdSession{id: session_id, key: session_key};

        let OpenIdSessionFinalizeInfo{ email, email_verified, .. } = OpenIdSessionDO::finalize_exec(&ctx.env, session.clone()).await?; 

        let mut user = match UserAccount::load_by_email(&ctx.env, &email).await.ok() {
            // user already exists, just sign them in
            Some(user) => {
                worker::console_log!("user already exists, signing in");
                user
            }, 

            // user doesn't exist, register them
            None => {
                worker::console_log!("user does not exist, registering");
                // theoretically we could use finalize_info.access_token to load profile info etc.
                // but, meh, let the user just set it all fresh - makes it easier to integrate
                // with various providers too

                // create a new user account
                let uid = UserId::new(uuid::Uuid::now_v7());
                let user_token = uuid::Uuid::now_v7().as_simple().to_string();
                // random password
                let password = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&rand::thread_rng().gen::<[u8; OAUTH_REGISTER_PASSWORD_LENGTH]>());

                UserAccount::insert(&ctx.env, &uid, &password, &email, &user_token).await?;
                
                UserAccount::load_by_email(&ctx.env, &email).await?
            }
        };

        // update the user's email_verified status if it's changed to true
        if !user.email_verified && email_verified {
            UserAccount::update_email_verified(&ctx.env, &user.id, email_verified).await?;
            user.email_verified = true;
        }

        // sign the user in and return
        let auth_token = AuthTokenDO::create(&ctx.env, AuthTokenKind::Signin, user.id.clone(), user.user_token.clone(), AUTH_SIGNIN_TOKEN_EXPIRES).await?;
        let auth_key = auth_token.key.clone();
        Ok((AuthOpenIdFinalizeExecResponse{
            uid: user.id,
            email_verified: user.email_verified || email_verified,
            auth_key
        }, auth_token))
    }

    fn response(_ctx: &ApiContext, data: AuthOpenIdFinalizeExecResponse, auth_token: AuthTokenCreateResponse) -> Response {
        let res = Response::new_json(&data);
        set_signin_cookie(&res, &auth_token.id);
        res
    }
}

#[async_trait(?Send)]
impl ApiEmptyExt for AuthSignout {

    async fn handle(ctx: &ApiContext) -> ApiResult<()> {
        let user = ctx.user.as_ref().unwrap();
        AuthTokenDO::destroy(&ctx.env, &user.token_id).await?;

        Ok(())
    }
    fn response(_ctx: &ApiContext) -> Response {
        let res = Response::new_empty();
        delete_signin_cookie(&res);
        res
    }
}

#[async_trait(?Send)]
impl ApiEmptyExt for AuthSendVerifyEmail {
    async fn handle(ctx: &ApiContext) -> ApiResult<()> {
        let user = ctx.user.as_ref().unwrap();

        // create a new oob token
        let auth_token = AuthTokenDO::create(&ctx.env, AuthTokenKind::VerifyEmail, user.account.id.clone(), user.account.user_token.clone(), AUTH_VERIFY_EMAIL_TOKEN_EXPIRES).await?;

        mailer::send(ctx, &user.account.email, MailerKind::EmailVerification { 
            oob_token_id: auth_token.id, 
            oob_token_key: auth_token.key 
        }).await?;

        Ok(())
    }
}

#[async_trait(?Send)]
impl ApiReqExt for AuthConfirmVerifyEmail {
    type Req = <AuthConfirmVerifyEmail as ApiReq>::Req;

    async fn handle(ctx: &ApiContext, data: AuthConfirmVerifyEmailRequest) -> ApiResult<()> {
        let AuthConfirmVerifyEmailRequest {oob_token_id, oob_token_key} = data;
        let account = validate_oob_token(&ctx.env, AuthTokenKind::VerifyEmail, oob_token_id, oob_token_key, AuthTokenAfterValidation::Delete).await?;

        // now update the DB
        UserAccount::update_email_verified(&ctx.env, &account.id, true).await?;

        Ok(())
    }
}


#[async_trait(?Send)]
impl ApiReqExt for AuthSendResetPasswordAny {
    type Req = <AuthSendResetPasswordAny as ApiReq>::Req;

    async fn handle(ctx: &ApiContext, data: AuthSendResetPasswordRequestAny) -> ApiResult<()> {
        let account = UserAccount::load_by_email(&ctx.env, &data.email).await.map_err(|_| AuthError::NoUserPasswordReset)?;
        helper_send_password_reset(ctx, &account).await
    }
}

#[async_trait(?Send)]
impl ApiEmptyExt for AuthSendResetPasswordMe {

    async fn handle(ctx: &ApiContext) -> ApiResult<()> {
        let user = ctx.user.as_ref().unwrap();
        helper_send_password_reset(ctx, &user.account).await
    }
}


pub async fn helper_send_password_reset(ctx: &ApiContext, account: &UserAccount) -> ApiResult<()> {
    // create a new oob token
    let auth_token = AuthTokenDO::create(&ctx.env, AuthTokenKind::PasswordReset, account.id.clone(), account.user_token.clone(), AUTH_RESET_PASSWORD_TOKEN_EXPIRES).await?;

    mailer::send(&ctx, &account.email, MailerKind::PasswordReset { 
        oob_token_id: auth_token.id, 
        oob_token_key: auth_token.key
    }).await?;
    Ok(())
}


#[async_trait(?Send)]
impl ApiBothWithExtraExt for AuthConfirmResetPassword {
    type Req = <AuthConfirmResetPassword as ApiBoth>::Req;
    type Res = <AuthConfirmResetPassword as ApiBoth>::Res;
    type Extra = AuthTokenCreateResponse;

    async fn handle(ctx: &ApiContext, data: AuthConfirmResetPasswordRequest) -> ApiResult<(AuthConfirmResetPasswordResponse, AuthTokenCreateResponse)> {
        let AuthConfirmResetPasswordRequest{oob_token_id, oob_token_key, password} = data;

        let account = validate_oob_token(&ctx.env, AuthTokenKind::PasswordReset, oob_token_id, oob_token_key, AuthTokenAfterValidation::Delete).await?;
        let password = hash_password(&password, None)?;
        let user_token = uuid::Uuid::now_v7().as_simple().to_string();

        UserAccount::reset_password(&ctx.env, &account.id, &password, &user_token).await?;

        // note that this uses the new user_token
        let auth_token = AuthTokenDO::create(&ctx.env, AuthTokenKind::Signin, account.id.clone(), user_token.clone(), AUTH_SIGNIN_TOKEN_EXPIRES).await?;
        let auth_key = auth_token.key.clone();
        Ok((AuthConfirmResetPasswordResponse{
            uid: account.id.clone(),
            email_verified: account.email_verified,
            auth_key
        }, auth_token))
    }

    fn response(_ctx: &ApiContext, data: AuthConfirmResetPasswordResponse, auth_token: AuthTokenCreateResponse) -> Response {
        let res = Response::new_json(&data);
        set_signin_cookie(&res, &auth_token.id);
        res
    }
}

#[async_trait(?Send)]
impl ApiBothExt for AuthCheckResetPassword {
    type Req = <AuthCheckResetPassword as ApiBoth>::Req;
    type Res = <AuthCheckResetPassword as ApiBoth>::Res;

    async fn handle(ctx: &ApiContext, data: AuthCheckResetPasswordRequest) -> ApiResult<AuthCheckResetPasswordResponse> {
        let AuthCheckResetPasswordRequest{oob_token_id, oob_token_key} = data;

        let account = validate_oob_token(&ctx.env, AuthTokenKind::PasswordReset, oob_token_id, oob_token_key, AuthTokenAfterValidation::ExtendExpiresMs(AUTH_RESET_PASSWORD_TOKEN_EXPIRES)).await?;
        Ok(AuthCheckResetPasswordResponse{
            uid: account.id,
            email: account.email
        })
    }
}