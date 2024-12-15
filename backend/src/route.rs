use crate::api_ext::*;
use crate::{auth::AuthUser, config::API_ROOT_PATH, not_found::NotFoundHandler, prelude::*};
use shared::{
    api::{
        action::{AddAction, DeleteAction, ListActionDestinations, ListActions},
        auth::{AuthCheck, AuthRegisterEmail, AuthLogin, AuthSignout},
        info::ServerInfo,
        telegram::TelegramWebHook,
    },
    backend::route::{ActionRoute, AdminRoute, AuthRoute, Route},
};
use worker::{Context, Env};

pub async fn handle_route(req: HttpRequest, env: Env, cf_ctx: Context) -> ApiResponse {
    Ok(
        match Route::try_from_url(&req.uri().to_string(), API_ROOT_PATH) {
            Some(route) => {
                let user = AuthUser::try_new(&env, &req, &route).await?;
                let ctx = ApiContext::new(req, env, cf_ctx, user);

                match route {
                    Route::Auth(auth_route) => match auth_route {
                        AuthRoute::RegisterEmail => AuthRegisterEmail::router(ctx).await?,
                        AuthRoute::Check => AuthCheck::router(ctx).await?,
                        AuthRoute::Login => AuthLogin::router(ctx).await?,
                        AuthRoute::Signout => AuthSignout::router(ctx).await?,
                    },
                    Route::Admin(admin_route) => match admin_route {
                        AdminRoute::Placeholder => empty_response(None)
                    },
                    Route::Action(action_route) => match action_route {
                        ActionRoute::ListDestinations => {
                            ListActionDestinations::router(ctx).await?
                        }
                        ActionRoute::AddAction => AddAction::router(ctx).await?,
                        ActionRoute::DeleteAction => DeleteAction::router(ctx).await?,
                        ActionRoute::ListActions => ListActions::router(ctx).await?,
                    },
                    Route::Info => ServerInfo::router(ctx).await?,
                    Route::TelegramWebHook => TelegramWebHook::router(ctx).await?,
                }
            }
            None => {
                let ctx = ApiContext::new(req, env, cf_ctx, None);
                NotFoundHandler::new(ctx).handle().await?
            }
        },
    )
}
