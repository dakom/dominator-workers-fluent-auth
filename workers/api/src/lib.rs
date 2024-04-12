mod auth;
mod context;
mod config;
mod prelude;
mod db;
mod route;
mod not_found;
mod api_ext;
mod helpers;
mod mailer;

use config::ALLOWED_ORIGINS;
use route::handle_route;
// use route::handle_route;
use shared::auth::{AUTH_TOKEN_ID_NAME, AUTH_TOKEN_KEY_NAME};
use worker::{
    Env,
    event,
    Context,
};
use prelude::*;

#[event(fetch, respond_with_errors)]
async fn main(req: Request, env: Env, ctx: Context) -> worker::Result<Response> {
    set_panic_hook();

    let origin = match req.headers().get("origin")? {
        Some(origin) => Some(origin),
        None => req.headers().get("referrer")?
    };

    // early-exit for CORS options
    if req.method().to_uppercase() == "OPTIONS" {
        Ok(apply_cors(origin, Response::new_empty()))
    } else {
        let res = match handle_route(req, env, ctx).await {
            Ok(res) => res,
            Err(res) => {
                worker::console_error!("error: {:?}", res);
                res.into()
            }
        };
        Ok(apply_cors(origin, res))
    }
}

fn apply_cors(origin: Option<String>, res: Response) -> Response {

    if let Some(origin) = origin {
        if ALLOWED_ORIGINS.iter().any(|x| *x == origin) {
            res.headers().set("Access-Control-Allow-Origin", &origin).unwrap();
        }
    }

    res.headers().set("Access-Control-Allow-Credentials", "true").unwrap();
    res.headers().set("Access-Control-Max-Age", "86400").unwrap();
    res.headers().set("Access-Control-Allow-Methods", "GET, HEAD, POST, OPTIONS").unwrap();
    res.headers().set("Access-Control-Allow-Headers", &format!("Content-Type, {AUTH_TOKEN_KEY_NAME}, {AUTH_TOKEN_ID_NAME}")).unwrap();

    res
}

#[cfg(feature = "console_error_panic_hook")]
fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

#[cfg(not(feature = "console_error_panic_hook"))]
fn set_panic_hook() {
} 