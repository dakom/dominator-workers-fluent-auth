use shared::{auth::{AUTH_TOKEN_ID_NAME, AUTH_TOKEN_KEY_NAME}, backend::route::{Route, RouteAuthKind}};
use worker::Env;

use crate::{prelude::*, config::AUTH_SIGNIN_TOKEN_EXPIRES, db::user::UserAccount};

use super::durable_objects::token::{AuthTokenAfterValidation, AuthTokenDO, AuthTokenKind, AuthTokenValidateResponse};

pub struct AuthUser {
    pub account: UserAccount,
    pub token_id: String,
    pub token_key: String,
}

impl AuthUser {
    pub async fn try_new(env: &Env, req: &Request, route: &Route) -> ApiResult<Option<AuthUser>> {
        // early exit or get the auth token
        let user = match route.auth_kind() {
            RouteAuthKind::None | RouteAuthKind::CookiesOnly => {
                None
            },
            auth_kind => match AuthUser::validate(&env, &req, auth_kind).await {
                Ok(user) => {
                    Some(user)
                },
                Err(err) => {
                    return Err(match err {
                        ApiError::Auth(AuthError::EmailNotVerified) => err,
                        _ => {
                            worker::console_log!("Auth error: {:?}", err);
                            // we could log the specific error here,
                            // but for clients we just want to say "not authorized"
                            // in case errors leak semi-sensitive info for debugging (like the nature of the auth keys, etc.) 
                            ApiError::Auth(AuthError::NotAuthorized)
                        }
                    });
                }
            }
        };

        Ok(user)
    }

    async fn validate(env: &Env, req: &Request, auth_kind: RouteAuthKind) -> ApiResult<AuthUser> {
        // first try and get it from the header, e.g. for non-browser clients
        let mut token_id = req.headers().get(AUTH_TOKEN_ID_NAME)?;

        if token_id.is_none() {
            // then try to get token id from cookie, which is sent from browser clients
            if let Some(cookie_header) = req.headers().get("cookie")? {
                token_id = cookie_header
                    .split(";")
                    .map(|x| x.trim())
                    .find(|x| x.starts_with(AUTH_TOKEN_ID_NAME))
                    .map(|x| x.split_once("=").map(|x| x.1.to_string()))
                    .flatten()
            }
        }

        let token_id = token_id.ok_or(ApiError::from("missing token id".to_string()))?;
        // token key is always from header
        let token_key = req.headers().get(AUTH_TOKEN_KEY_NAME)?.ok_or(ApiError::from("missing token key".to_string()))?;


        // validate the token id and key
        let AuthTokenValidateResponse {uid, user_token} = AuthTokenDO::validate(env, AuthTokenKind::Signin, &token_id, token_key.clone(), AuthTokenAfterValidation::ExtendExpiresMs(AUTH_SIGNIN_TOKEN_EXPIRES)).await?;

        let account = UserAccount::load_by_id(env, &uid).await?;

        match auth_kind {
            // no need to handle all the variants here, we've early-exited for non-auth routes
            // and anyway we end up with a strict fallback of at least getting a valid token and user id
            RouteAuthKind::PartialAuthTokenOnly => {
                // no further validation needed, having a valid token is enough to destroy it
            },
            _ => {
                // validate the user token
                if account.user_token != user_token {
                    return Err(format!("user token mismatch for user id {uid}").into())
                }
                // only for fully protected routes do we need to also validate the email
                // in other words, for send-verify-email, they are partially signed in
                // and may not actually have a valid email to send to, so they need to resend the verification link 
                if auth_kind == RouteAuthKind::Full {
                    // validate the email
                    if !account.email_verified {
                        return Err(AuthError::EmailNotVerified.into())
                    }
                }
            }
        }

        Ok(AuthUser {
            account,
            token_id,
            token_key
        })
    }
}