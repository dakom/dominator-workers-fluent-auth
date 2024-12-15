mod api_ext;
mod auth;
mod config;
mod context;
mod db;
mod handlers;
mod helpers;
mod kv;
mod not_found;
mod prelude;
mod route;
mod telegram;

use config::ALLOWED_ORIGINS;
use http::{HeaderValue, Method, StatusCode};
use prelude::*;
use route::handle_route;
use shared::{
    auth::{HEADER_ADMIN_CODE, HEADER_ADMIN_UID, HEADER_AUTH_TOKEN_ID, HEADER_AUTH_TOKEN_KEY},
    logger::init_logger,
};
use worker::{event, Context, Env};

#[event(fetch, respond_with_errors)]
async fn main(req: HttpRequest, env: Env, ctx: Context) -> worker::Result<HttpResponse> {
    init_logger();

    let origin = match req.headers().get("origin") {
        Some(origin) => Some(origin.clone()),
        None => req.headers().get("referrer").cloned(),
    };

    // early-exit for CORS options
    if req.method() == Method::OPTIONS {
        Ok(apply_cors(origin, empty_response(None)))
    } else {
        let res = match handle_route(req, env, ctx).await {
            Ok(res) => res,
            Err(err) => {
                tracing::error!("weird, got an error: {:?}", err);

                let status_code = match &err {
                    // just a nice helper to debug things
                    // it's up to the frontend to decide what to do with this
                    ApiError::Auth(_) => StatusCode::UNAUTHORIZED,
                    ApiError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    ApiError::Telegram(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    ApiError::Kv(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    ApiError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    ApiError::Parse(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    ApiError::MissingBody(_) => StatusCode::BAD_REQUEST,
                    ApiError::ParseBody(_) => StatusCode::BAD_REQUEST,
                };

                let res = any_to_json_response(&err, Some(status_code)).await;

                res
            }
        };
        Ok(apply_cors(origin, res))
    }
}

fn apply_cors(origin: Option<HeaderValue>, mut res: HttpResponse) -> HttpResponse {
    let headers = res.headers_mut();

    if let Some(origin) = origin {
        if ALLOWED_ORIGINS.iter().any(|x| *x == origin) {
            headers.insert("Access-Control-Allow-Origin", origin);
        }
    }

    headers.insert("Access-Control-Allow-Credentials", "true".parse().unwrap());
    headers.insert("Access-Control-Max-Age", "86400".parse().unwrap());
    headers.insert(
        "Access-Control-Allow-Methods",
        "GET, HEAD, POST, OPTIONS".parse().unwrap(),
    );
    headers.insert("Access-Control-Allow-Headers", format!("Content-Type, {HEADER_AUTH_TOKEN_KEY}, {HEADER_AUTH_TOKEN_ID}, {HEADER_ADMIN_CODE}, {HEADER_ADMIN_UID}").parse().unwrap());

    res
}
