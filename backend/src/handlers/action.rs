use crate::{
    api_ext::*,
    db::{action::TelegramActionDb, destination::TelegramDestinationDb},
    ApiContext,
};
use action::{
    Action, ActionId, AddAction, AddActionRequest, AddActionResponse, DeleteAction,
    DeleteActionRequest, ListActionDestinations, ListActionDestinationsRequest,
    ListActionDestinationsResponse, ListActions, ListActionsRequest, ListActionsResponse,
};
use async_trait::async_trait;
use shared::{api::*, backend::result::ApiResult};

#[async_trait(?Send)]
impl ApiBothExt for ListActionDestinations {
    type Res = <Self as ApiBoth>::Res;
    type Req = <Self as ApiBoth>::Req;

    async fn handle(
        ctx: &ApiContext<ListActionDestinationsRequest>,
    ) -> ApiResult<ListActionDestinationsResponse> {
        let uid = ctx.uid_unchecked();

        let destinations = TelegramDestinationDb::list(&ctx.env, &uid).await?;

        Ok(ListActionDestinationsResponse { destinations })
    }
}

impl FromHttpRequest for ListActionDestinationsRequest {}

#[async_trait(?Send)]
impl ApiBothExt for AddAction {
    type Res = <Self as ApiBoth>::Res;
    type Req = <Self as ApiBoth>::Req;

    async fn handle(ctx: &ApiContext<AddActionRequest>) -> ApiResult<AddActionResponse> {
        let uid = ctx.uid_unchecked();

        let destination =
            TelegramDestinationDb::load_with_user_id(&ctx.env, &ctx.req.destination_id, &uid)
                .await?;

        let action_id = ActionId::new(uuid::Uuid::now_v7());

        TelegramActionDb::insert(
            &ctx.env,
            &action_id,
            &ctx.req.destination_id,
            &ctx.req.prompt,
            &ctx.req.message,
        )
        .await?;

        let action = Action {
            id: action_id,
            destination: destination.into(),
            prompt: ctx.req.prompt.clone(),
            message: ctx.req.message.clone(),
        };

        Ok(AddActionResponse { action })
    }
}

impl FromHttpRequest for AddActionRequest {}

#[async_trait(?Send)]
impl ApiReqExt for DeleteAction {
    type Req = <Self as ApiReq>::Req;

    async fn handle(ctx: &ApiContext<DeleteActionRequest>) -> ApiResult<()> {
        let uid = ctx.uid_unchecked();

        TelegramActionDb::delete(&ctx.env, &uid, &ctx.req.id).await?;

        Ok(())
    }
}

impl FromHttpRequest for DeleteActionRequest {}

#[async_trait(?Send)]
impl ApiBothExt for ListActions {
    type Res = <Self as ApiBoth>::Res;
    type Req = <Self as ApiBoth>::Req;

    async fn handle(ctx: &ApiContext<ListActionsRequest>) -> ApiResult<ListActionsResponse> {
        let uid = ctx.uid_unchecked();

        let actions = TelegramActionDb::list(&ctx.env, &uid).await?;

        Ok(ListActionsResponse { actions })
    }
}

impl FromHttpRequest for ListActionsRequest {}
