use crate::{
    any_to_json_response, api_ext::*, config::AUTH_TOKEN_SIGNIN_EXPIRES_DURATION, db::user::{UserAccountDb, UserEmailDb, UserInsertKind}, empty_response, kv::auth::AuthKv, utils::crypto::{hash_password, PasswordSalt}, ApiContext
};
use async_trait::async_trait;
use auth::{
    AuthCheck, AuthCheckResponse, AuthRegisterEmail, AuthRegisterEmailRequest, AuthSigninEmail, AuthSigninEmailRequest, AuthSigninResponse, AuthSignout, AuthSignoutRequest, AuthTokenCreateResponse, AuthTokenKind
};
use shared::{
    api::*,
    auth::HEADER_AUTH_TOKEN_ID,
    backend::result::{ApiError, ApiResult, AuthError}, user::UserId,
};
use worker::{HttpRequest, HttpResponse};

// Register
#[async_trait(?Send)]
impl ApiBothWithExtraExt for AuthRegisterEmail {
    type Req = <Self as ApiBoth>::Req;
    type Res = <Self as ApiBoth>::Res;
    type Extra = AuthTokenCreateResponse;

    async fn handle(
        ctx: &ApiContext<AuthRegisterEmailRequest>,
    ) -> ApiResult<(AuthSigninResponse, AuthTokenCreateResponse)> {
        let AuthRegisterEmailRequest {
            email,
            password
        } = &ctx.req;


        if UserEmailDb::exists(&ctx.env, &email).await? {
            return Err(ApiError::Auth(AuthError::EmailAlreadyExists));
        }

        let password = hash_password(&password, PasswordSalt::CreateNew)?;

        // Register in database
        let uid = UserId::new(uuid::Uuid::now_v7());
        let user_token = uuid::Uuid::now_v7().as_simple().to_string();
        UserAccountDb::insert(&ctx.env, &uid, &user_token, UserInsertKind::EmailPw { email, password: &password }, Vec::new()).await?;


        // Sign user in
        let auth_token = AuthKv::create(
            &ctx.env,
            AuthTokenKind::Signin,
            uid.clone(),
            user_token.clone(),
            *AUTH_TOKEN_SIGNIN_EXPIRES_DURATION,
        )
        .await?;
        let auth_key = auth_token.key.clone();

        Ok((AuthSigninResponse{ auth_key}, auth_token))
    }

    async fn response(
        _ctx: &ApiContext<AuthRegisterEmailRequest>,
        data: AuthSigninResponse,
        auth_token: AuthTokenCreateResponse,
    ) -> HttpResponse {
        let mut res = any_to_json_response(&data, None).await;
        set_login_cookie(&mut res, &auth_token.id);
        res
    }
}

impl FromHttpRequest for AuthRegisterEmailRequest {}

// Signin
#[async_trait(?Send)]
impl ApiBothWithExtraExt for AuthSigninEmail {
    type Req = <Self as ApiBoth>::Req;
    type Res = <Self as ApiBoth>::Res;
    type Extra = AuthTokenCreateResponse;

    async fn handle(
        ctx: &ApiContext<AuthSigninEmailRequest>,
    ) -> ApiResult<(AuthSigninResponse, AuthTokenCreateResponse)> {
        let AuthSigninEmailRequest {
            email,
            password
        } = &ctx.req;

        let user_email_account = UserEmailDb::load(&ctx.env, &email).await?;

        let password = hash_password(&password, PasswordSalt::Recover)?;

        if user_email_account.password != password {
            return Err(ApiError::Auth(AuthError::InvalidPassword));
        }

        // eh, this could be a join with above, but whatever
        let user = UserAccountDb::load(&ctx.env, &user_email_account.user_id).await?;

        // Sign user in
        let auth_token = AuthKv::create(
            &ctx.env,
            AuthTokenKind::Signin,
            user.id.clone(),
            user.user_token.clone(),
            *AUTH_TOKEN_SIGNIN_EXPIRES_DURATION,
        )
        .await?;
        let auth_key = auth_token.key.clone();

        Ok((
            AuthSigninResponse {
                auth_key,
            },
            auth_token,
        ))
    }

    async fn response(
        _ctx: &ApiContext<AuthSigninEmailRequest>,
        data: AuthSigninResponse,
        auth_token: AuthTokenCreateResponse,
    ) -> HttpResponse {
        let mut res = any_to_json_response(&data, None).await;
        set_login_cookie(&mut res, &auth_token.id);
        res
    }
}

impl FromHttpRequest for AuthSigninEmailRequest {}

// Signout
#[async_trait(?Send)]
impl ApiReqExt for AuthSignout {
    type Req = <Self as ApiReq>::Req;

    async fn handle(ctx: &ApiContext<AuthSignoutRequest>) -> ApiResult<()> {
        let AuthSignoutRequest { everywhere } = &ctx.req;

        // safe, signout requires that the auth_token in kv was validated
        let user = ctx.user.as_ref().unwrap();

        AuthKv::delete(&ctx.env, AuthTokenKind::Signin, &user.token_id).await?;

        if *everywhere {
            let user_token = uuid::Uuid::now_v7().as_simple().to_string();
            UserAccountDb::update_user_token(&ctx.env, &user.id, &user_token).await?;
        }

        Ok(())
    }

    async fn response(_ctx: &ApiContext<AuthSignoutRequest>) -> HttpResponse {
        let mut res = empty_response(None);
        delete_login_cookie(&mut res);
        res
    }
}

impl FromHttpRequest for AuthSignoutRequest {}

// Check
#[async_trait(?Send)]
impl ApiResExt for AuthCheck {
    type Res = <Self as ApiRes>::Res;

    async fn handle(ctx: &ApiContext<HttpRequest>) -> ApiResult<AuthCheckResponse> {
        let user = ctx.user_unchecked();
        let uid = user.id.clone();
        let roles = user.roles.clone();

        Ok(AuthCheckResponse { uid, roles })
    }
}

#[cfg(debug_assertions)]
pub fn set_login_cookie(res: &mut HttpResponse, auth_token_id: &str) {
    let value = format!("{HEADER_AUTH_TOKEN_ID}={auth_token_id}; Path=/; HttpOnly; Secure; Partitioned; SameSite=None; Max-Age=2147483647");
    res.headers_mut()
        .insert("Set-Cookie", value.parse().unwrap());
}

// since our api server is on a different domain
// we need samesite=none
// see https://www.troyhunt.com/promiscuous-cookies-and-their-impending-death-via-the-samesite-policy/
#[cfg(not(debug_assertions))]
pub fn set_login_cookie(res: &mut HttpResponse, auth_token_id: &str) {
    let value = format!("{HEADER_AUTH_TOKEN_ID}={auth_token_id}; Path=/; HttpOnly; Secure; Partitioned; SameSite=None; Max-Age=2147483647");
    res.headers_mut()
        .insert("Set-Cookie", value.parse().unwrap());
}

pub fn delete_login_cookie(res: &mut HttpResponse) {
    let value = format!("{HEADER_AUTH_TOKEN_ID}=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT");
    res.headers_mut()
        .insert("Set-Cookie", value.parse().unwrap());
}
