use crate::api_ext::*;
use crate::{auth::User, config::API_ROOT_PATH, not_found::NotFoundHandler, prelude::*};
use shared::{
    api::{
        auth::{AuthCheck, AuthRegisterEmail, AuthSigninEmail, AuthSignout},
        info::ServerInfo,
    },
    backend::route::{AdminRoute, AuthRoute, Route},
};
use worker::{Context, Env};

pub async fn handle_route(req: HttpRequest, env: Env, cf_ctx: Context) -> ApiResponse {
    Ok(
        match Route::try_from_url(&req.uri().to_string(), API_ROOT_PATH) {
            Some(route) => {
                let user = User::try_new(&env, &req, &route).await?;
                let ctx = ApiContext::new(req, env, cf_ctx, user);

                match route {
                    Route::Auth(auth_route) => match auth_route {
                        AuthRoute::RegisterEmail => AuthRegisterEmail::router(ctx).await?,
                        AuthRoute::Check => AuthCheck::router(ctx).await?,
                        AuthRoute::SigninEmail => AuthSigninEmail::router(ctx).await?,
                        AuthRoute::Signout => AuthSignout::router(ctx).await?,
                        _ => unimplemented!()
                    },
                    Route::Admin(admin_route) => match admin_route {
                        AdminRoute::Placeholder => empty_response(None)
                    },
                    Route::Info => ServerInfo::router(ctx).await?,
                }
            }
            None => {
                let ctx = ApiContext::new(req, env, cf_ctx, None);
                NotFoundHandler::new(ctx).handle().await?
            }
        },
    )
}
