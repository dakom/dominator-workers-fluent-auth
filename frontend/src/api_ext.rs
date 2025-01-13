// this builds on the comments on shared/api, but here in the frontend it's just about extending the api traits
use crate::{CONFIG, LOCALE};
use async_trait::async_trait;
use awsm_web::{
    loaders::fetch::{fetch_with_headers, fetch_with_headers_and_data, Response},
    prelude::UnwrapExt,
};
use serde::de::DeserializeOwned;
use shared::{
    api::{ApiBoth, ApiEmpty, ApiReq, ApiRes},
    auth::HEADER_AUTH_TOKEN_KEY,
    backend::{
        result::{ApiError, ApiResult, AuthError},
        route::RouteAuthKind,
    },
};

use crate::auth::AUTH;

#[async_trait(?Send)]
pub trait ApiBothExt<Req, Res> {
    async fn fetch(data: Req) -> ApiResult<Res>;
}

#[async_trait(?Send)]
impl<T> ApiBothExt<<T as ApiBoth>::Req, <T as ApiBoth>::Res> for T
where
    T: ApiBoth,
{
    async fn fetch(data: <T as ApiBoth>::Req) -> ApiResult<<T as ApiBoth>::Res> {
        let route = T::ROUTE;
        let url = route.link(CONFIG.api_domain, CONFIG.api_root_path);
        let method = T::METHOD;
        let method = method.as_str();

        let res = match route.auth_kind() {
            RouteAuthKind::None => {
                fetch_with_headers_and_data(&url, method, false, &noauth_headers(), Some(data))
                    .await
            }
            RouteAuthKind::NoAuthCookieSetter => {
                fetch_with_headers_and_data(&url, method, true, &noauth_headers(), Some(data)).await
            }
            RouteAuthKind::Admin
            | RouteAuthKind::Full
            | RouteAuthKind::PartialAuthTokenOnly
            | RouteAuthKind::PartialAuthAndUserTokenOnly => {
                fetch_with_headers_and_data(&url, method, true, &auth_headers()?, Some(data)).await
            }
        };

        map_response_data(res).await
    }
}

#[async_trait(?Send)]
pub trait ApiReqExt<Req> {
    async fn fetch(data: Req) -> ApiResult<()>;
}
#[async_trait(?Send)]
impl<T> ApiReqExt<<T as ApiReq>::Req> for T
where
    T: ApiReq,
{
    async fn fetch(data: <T as ApiReq>::Req) -> ApiResult<()> {
        let route = T::ROUTE;
        let url = route.link(CONFIG.api_domain, CONFIG.api_root_path);
        let method = T::METHOD;
        let method = method.as_str();

        let res = match route.auth_kind() {
            RouteAuthKind::None => {
                fetch_with_headers_and_data(&url, method, false, &noauth_headers(), Some(data))
                    .await
            }
            RouteAuthKind::NoAuthCookieSetter => {
                fetch_with_headers_and_data(&url, method, true, &noauth_headers(), Some(data)).await
            }
            RouteAuthKind::Admin
            | RouteAuthKind::Full
            | RouteAuthKind::PartialAuthTokenOnly
            | RouteAuthKind::PartialAuthAndUserTokenOnly => {
                fetch_with_headers_and_data(&url, method, true, &auth_headers()?, Some(data)).await
            }
        };

        map_response_empty(res).await
    }
}

#[async_trait(?Send)]
pub trait ApiResExt<Res> {
    async fn fetch() -> ApiResult<Res>;
}

#[async_trait(?Send)]
impl<T> ApiResExt<<T as ApiRes>::Res> for T
where
    T: ApiRes,
{
    async fn fetch() -> ApiResult<<T as ApiRes>::Res> {
        let route = T::ROUTE;
        let url = route.link(CONFIG.api_domain, CONFIG.api_root_path);
        let method = T::METHOD;
        let method = method.as_str();

        let res = match route.auth_kind() {
            RouteAuthKind::None => {
                fetch_with_headers(&url, &method, false, &noauth_headers()).await
            }
            RouteAuthKind::NoAuthCookieSetter => {
                fetch_with_headers(&url, &method, true, &noauth_headers()).await
            }
            RouteAuthKind::Admin
            | RouteAuthKind::Full
            | RouteAuthKind::PartialAuthTokenOnly
            | RouteAuthKind::PartialAuthAndUserTokenOnly => {
                fetch_with_headers(&url, &method, true, &auth_headers()?).await
            }
        };

        map_response_data(res).await
    }
}
#[async_trait(?Send)]
pub trait ApiEmptyExt {
    async fn fetch() -> ApiResult<()>;
}

#[async_trait(?Send)]
impl<T> ApiEmptyExt for T
where
    T: ApiEmpty,
{
    async fn fetch() -> ApiResult<()> {
        let route = T::ROUTE;
        let url = route.link(CONFIG.api_domain, CONFIG.api_root_path);
        let method = T::METHOD;
        let method = method.as_str();

        let res = match route.auth_kind() {
            RouteAuthKind::None => fetch_with_headers(&url, method, false, &noauth_headers()).await,
            RouteAuthKind::NoAuthCookieSetter => {
                fetch_with_headers(&url, method, true, &noauth_headers()).await
            }
            RouteAuthKind::Admin
            | RouteAuthKind::Full
            | RouteAuthKind::PartialAuthTokenOnly
            | RouteAuthKind::PartialAuthAndUserTokenOnly => {
                fetch_with_headers(&url, method, true, &auth_headers()?).await
            }
        };

        map_response_empty(res).await
    }
}

// helpers
fn auth_headers() -> ApiResult<Vec<(&'static str, String)>> {
    let token = match AUTH.try_clone_token_key() {
        Some(token) => Some(token),
        None => {
            match web_sys::window()
                .unwrap_ext()
                .local_storage()
                .unwrap_ext()
                .unwrap_ext()
                .get_item(CONFIG.auth_login_key_storage_name)
                .ok()
                .flatten()
            {
                Some(token) => {
                    *AUTH.token_key.write().unwrap() = Some(token.clone());
                    Some(token)
                }
                None => None,
            }
        }
    };

    match token {
        Some(token) => Ok(vec![
            (HEADER_AUTH_TOKEN_KEY, token),
            (
                "Content-Language",
                LOCALE.current.lock_ref().lang_id.to_string(),
            ),
        ]),
        None => Err(ApiError::Auth(AuthError::NotAuthorized)),
    }
}

fn noauth_headers() -> [(&'static str, String); 1] {
    [(
        "Content-Language",
        LOCALE.current.lock_ref().lang_id.to_string(),
    )]
}

async fn map_response_data<T: DeserializeOwned>(
    res: Result<Response, awsm_web::errors::Error>,
) -> ApiResult<T> {
    match res {
        Ok(res) => match res.status() {
            200 => match res.json_from_obj().await {
                Ok(json) => Ok(json),
                Err(err) => Err(err.into_api_error()),
            },
            _ => Err(map_bad_status(res).await),
        },
        Err(err) => Err(err.into_api_error()),
    }
}
async fn map_response_empty(res: Result<Response, awsm_web::errors::Error>) -> ApiResult<()> {
    match res {
        Ok(res) => match res.status() {
            200 => Ok(()),
            _ => Err(map_bad_status(res).await),
        },
        Err(err) => Err(err.into_api_error()),
    }
}

async fn map_bad_status(res: Response) -> ApiError {
    let err = match res.text().await {
        Ok(text) => match serde_json::from_str::<ApiError>(&text) {
            Ok(err) => err,
            Err(_) => ApiError::Unknown(text),
        },
        Err(err) => ApiError::Unknown(err.to_string()),
    };

    match &err {
        ApiError::Auth(auth_error) => {
            match auth_error {
                AuthError::NotAuthorized => {
                    AUTH.clear();
                }
                AuthError::InvalidPassword
                | AuthError::EmailEmpty
                | AuthError::EmailNotVerified
                | AuthError::EmailAlreadyExists
                | AuthError::TermsNotAgreed => {
                    // do nothing
                }
            }
        }
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
