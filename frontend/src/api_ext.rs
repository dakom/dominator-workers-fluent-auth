// this builds on the comments on shared/api, but here in the frontend it's just about extending the api traits
use std::ops::Deref;

use async_trait::async_trait;
use awsm_web::{loaders::fetch::{fetch_url, fetch_with_data, fetch_with_headers, fetch_with_headers_and_data, Response}, prelude::UnwrapExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use shared::{api::{ApiEmpty, ApiBoth, ApiReq, ApiRes}, auth::AUTH_TOKEN_KEY_NAME, backend::{
    result::{ApiError, ApiResult, AuthError}, route::{AuthRoute, Route as ApiRoute, RouteAuthKind}
}};
use crate::{CONFIG, LOCALE};

use crate::auth::{AuthPhase, AUTH};

#[async_trait(?Send)]
pub trait ApiBothExt<Req, Res> {
    async fn fetch(data: Req) -> ApiResult<Res>;
}

#[async_trait(?Send)]
impl <T> ApiBothExt<<T as ApiBoth>::Req, <T as ApiBoth>::Res> for T 
where T: ApiBoth
{
    async fn fetch(data: <T as ApiBoth>::Req) -> ApiResult<<T as ApiBoth>::Res> {
        let route = T::ROUTE;
        let url = route.link(CONFIG.api_domain, CONFIG.api_root_path);
        let method = T::METHOD.as_str();

        let res = match route.auth_kind() {
            RouteAuthKind::None => {
                fetch_with_headers_and_data(&url, method, false, &noauth_headers(), Some(data)).await
            },
            RouteAuthKind::CookiesOnly => {
                fetch_with_headers_and_data(&url, method, true, &noauth_headers(), Some(data)).await
            },
            RouteAuthKind::Full | RouteAuthKind::PartialAuthTokenOnly | RouteAuthKind::PartialAuthAndUserTokenOnly => {
                fetch_with_headers_and_data(&url, method, true, &auth_headers()?, Some(data)).await
            },
        };

        map_response_data(res).await
    }
}


#[async_trait(?Send)]
pub trait ApiReqExt<Req> {
    async fn fetch(data: Req) -> ApiResult<()>;
}
#[async_trait(?Send)]
impl <T> ApiReqExt<<T as ApiReq>::Req> for T 
where T: ApiReq
{
    async fn fetch(data: <T as ApiReq>::Req) -> ApiResult<()> {
        let route = T::ROUTE;
        let url = route.link(CONFIG.api_domain, CONFIG.api_root_path);
        let method = T::METHOD.as_str();

        let res = match route.auth_kind() {
            RouteAuthKind::None => {
                fetch_with_headers_and_data(&url, method, false, &noauth_headers(), Some(data)).await
            },
            RouteAuthKind::CookiesOnly => {
                fetch_with_headers_and_data(&url, method, true, &noauth_headers(), Some(data)).await
            },
            RouteAuthKind::Full | RouteAuthKind::PartialAuthTokenOnly | RouteAuthKind::PartialAuthAndUserTokenOnly => {
                fetch_with_headers_and_data(&url, method, true, &auth_headers()?, Some(data)).await 
            },
        };

        map_response_empty(res).await
    }
}

#[async_trait(?Send)]
pub trait ApiResExt<Res> {
    async fn fetch() -> ApiResult<Res>;
}

#[async_trait(?Send)]
impl <T> ApiResExt<<T as ApiRes>::Res> for T 
where T: ApiRes
{
    async fn fetch() -> ApiResult<<T as ApiRes>::Res> {
        let route = T::ROUTE;
        let url = route.link(CONFIG.api_domain, CONFIG.api_root_path);
        let method = T::METHOD.as_str();

        let res = match route.auth_kind() {
            RouteAuthKind::None => {
                fetch_with_headers(&url, method, false, &noauth_headers()).await
            },
            RouteAuthKind::CookiesOnly => {
                fetch_with_headers(&url, method, true, &noauth_headers()).await
            },
            RouteAuthKind::Full | RouteAuthKind::PartialAuthTokenOnly | RouteAuthKind::PartialAuthAndUserTokenOnly => {
                fetch_with_headers(&url, method, true, &auth_headers()?).await
            },
        };

        map_response_data(res).await
    }
}
#[async_trait(?Send)]
pub trait ApiEmptyExt {
    async fn fetch() -> ApiResult<()>;
}

#[async_trait(?Send)]
impl <T> ApiEmptyExt for T 
where T: ApiEmpty
{
    async fn fetch() -> ApiResult<()> {
        let route = T::ROUTE;
        let url = route.link(CONFIG.api_domain, CONFIG.api_root_path);
        let method = T::METHOD.as_str();

        let res = match route.auth_kind() {
            RouteAuthKind::None => {
                fetch_with_headers(&url, method, false, &noauth_headers()).await
            },
            RouteAuthKind::CookiesOnly => {
                fetch_with_headers(&url, method, true, &noauth_headers()).await
            },
            RouteAuthKind::Full | RouteAuthKind::PartialAuthTokenOnly | RouteAuthKind::PartialAuthAndUserTokenOnly => {
                fetch_with_headers(&url, method, true, &auth_headers()?).await
            },
        };

        map_response_empty(res).await
    }
}

// helpers
fn auth_headers() -> ApiResult<[(&'static str, String);2]> {
    let token = match AUTH.try_clone_token_key() {
        Some(token) => Some(token),
        None => {
            match web_sys::window().unwrap_ext().local_storage().unwrap_ext().unwrap_ext().get_item(CONFIG.auth_signin_key_storage_name).ok().flatten() {
                Some(token) => { 
                    *AUTH.token_key.write().unwrap() = Some(token.clone());
                    Some(token)
                },
                None => None
            }
        }
    };

    match token {
        Some(token) => Ok([(AUTH_TOKEN_KEY_NAME, token), ("Content-Language", LOCALE.current.lock_ref().lang_id.to_string())]),
        None => Err(ApiError::Auth(AuthError::NotAuthorized))
    }
}

fn noauth_headers() -> [(&'static str, String);1] {
    [("Content-Language", LOCALE.current.lock_ref().lang_id.to_string())]
}

async fn map_response_data<T: DeserializeOwned>(res: Result<Response, awsm_web::errors::Error>) -> ApiResult<T> {
    match res {
        Ok(res) => {
            match res.status() {
                200 => {
                    match res.json_from_obj().await {
                        Ok(json) => Ok(json),
                        Err(err) => Err(err.into_api_error())
                    }
                },
                _ => Err(map_bad_status(res).await)
            }
        },
        Err(err) => Err(err.into_api_error())
    }
}
async fn map_response_empty(res: Result<Response, awsm_web::errors::Error>) -> ApiResult<()> {
    match res {
        Ok(res) => {
            match res.status() {
                200 => {
                    Ok(())
                },
                _ => Err(map_bad_status(res).await)
            }
        },
        Err(err) => Err(err.into_api_error())
    }
}

async fn map_bad_status(res: Response) -> ApiError {
    let err = match res.text().await {
        Ok(text) => match serde_json::from_str::<ApiError>(&text) {
            Ok(err) => err,
            Err(_) => ApiError::Unknown(text)
        },
        Err(err) => ApiError::Unknown(err.to_string())
    };


    match &err {
        ApiError::Auth(auth_error) => {
            match auth_error {
                AuthError::NotAuthorized => {
                    AUTH.clear();
                },
                // do not clear the auth in these cases, it's part of the auth flow itself
                // and clearing it would just cause a page refresh
                AuthError::EmailNotVerified => {
                    // this probably won't actually be hit, since we set the phase immediately
                    // in auth_check()... but better safe than sorry!
                    AUTH.phase.set_neq(AuthPhase::EmailNotVerified);
                },
                AuthError::InvalidSignin | AuthError::NoUserPasswordReset | AuthError::EmailAlreadyExists => {
                    // do nothing
                },

            }
        },
        _ => {}
    }

    err
}

pub trait ApiErrorExt {
    fn into_api_error(self) -> ApiError;
} 

impl ApiErrorExt for awsm_web::errors::Error {
    fn into_api_error(self) -> ApiError {
        ApiError::Unknown(self.to_string())
    }
}
