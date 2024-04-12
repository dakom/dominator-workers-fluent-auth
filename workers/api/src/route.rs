use crate::{api_ext::{ApiBothExt, ApiBothWithExtraExt, ApiEmptyDynRouteWithExtraExt, ApiEmptyExt, ApiReqExt, ApiResExt}, auth::AuthUser, config::API_ROOT_PATH, not_found::NotFoundHandler, prelude::*};
use worker::{Context, Env};
use shared::{api::auth::{AuthCheck, AuthCheckResetPassword, AuthConfirmResetPassword, AuthConfirmVerifyEmail, AuthOpenIdAccessTokenHook, AuthOpenIdConnect, AuthOpenIdFinalizeExec, AuthOpenIdFinalizeQuery, AuthRegister, AuthSendResetPasswordAny, AuthSendResetPasswordMe, AuthSendVerifyEmail, AuthSignin, AuthSignout}, backend::route::{AuthRoute, Route}};

pub async fn handle_route(req: Request, env: Env, cf_ctx: Context) -> ApiResponse {
    Ok(match Route::try_from_url(&req.url(), API_ROOT_PATH) {
        Some(route) => {
            let user = match AuthUser::try_new(&env, &req, &route).await {
                Ok(user) => user,
                Err(err) => return Ok(err.into())
            };

            let ctx = ApiContext::new(req, env, cf_ctx, user);

            let res = match route {
                Route::Auth(auth_route) => match auth_route {
                    AuthRoute::Register => {
                        AuthRegister::router(ctx).await
                    },
                    AuthRoute::Check => {
                        AuthCheck::router(ctx).await
                    },
                    AuthRoute::Signin => {
                        AuthSignin::router(ctx).await
                    },
                    AuthRoute::Signout => {
                        AuthSignout::router(ctx).await
                    },

                    AuthRoute::SendEmailValidation => {
                        AuthSendVerifyEmail::router(ctx).await
                    },

                    AuthRoute::ConfirmEmailValidation => {
                        AuthConfirmVerifyEmail::router(ctx).await
                    },

                    AuthRoute::SendPasswordResetMe => {
                        AuthSendResetPasswordMe::router(ctx).await
                    },
                    AuthRoute::SendPasswordResetAny => {
                        AuthSendResetPasswordAny::router(ctx).await
                    },

                    AuthRoute::ConfirmPasswordReset => {
                        AuthConfirmResetPassword::router(ctx).await
                    }

                    AuthRoute::CheckPasswordReset => {
                        AuthCheckResetPassword::router(ctx).await
                    }

                    AuthRoute::OpenIdConnect=> {
                        AuthOpenIdConnect::router(ctx).await
                    }

                    AuthRoute::OpenIdAccessTokenHook(provider)=> {
                        AuthOpenIdAccessTokenHook{provider}.router(ctx).await
                    }

                    AuthRoute::OpenIdFinalizeExec => {
                        AuthOpenIdFinalizeExec::router(ctx).await
                    }

                    AuthRoute::OpenIdFinalizeQuery => {
                        AuthOpenIdFinalizeQuery::router(ctx).await
                    }
                },
            };

            match res {
                Ok(res) => res,
                Err(err) => err.into()
            }
        },
        None => {
            let ctx = ApiContext::new(req, env, cf_ctx, None);
            NotFoundHandler::new(ctx).handle().await?
        }
    })
}
