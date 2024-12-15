use std::future::Future;
use std::pin::Pin;

use crate::prelude::*;
use crate::{ApiContext, ApiResponse};
/// See comments in shared/api
///
/// Bottom line: once an api endpoint is defined and implemented there, the frontend
/// automatically gets the typechecked synchronization for free
///
/// In the backend here, we need to implement the specific extension trait for each api endpoint
/// such that it binds the request and response type to eachother for each route
///
/// However, it's a generic pattern, and could be abstracted into a macro too
/// the key is defining the associated types like:
///
/// ```rust
/// type Req = <Foo as ApiBoth>::Req;
/// type Res = <Foo as ApiBoth>::Res;
/// ```
///
/// (adjusting for the specific struct and trait ofc)
///
/// then, the functions can take concrete types for readability - but it's typechecked
/// and constrained to make sure the request and response types are in sync
///
/// the specific part which can't be generalized is the actual handling logic, of course :)
///
/// Each of the api endpoints are implemented in the corresponding handler.rs
/// in this file we're just defining the traits
///
/// Only *one* of the traits should be implemented for each api endpoint
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use shared::backend::result::ApiResult;

pub trait FromHttpRequest: Sized + DeserializeOwned {
    fn from_request(
        _env: worker::Env,
        req: HttpRequest,
    ) -> Pin<Box<dyn Future<Output = ApiResult<Self>>>> {
        Box::pin(async move { json_body_to_any::<Self>(req.into_body()).await })
    }
}

#[async_trait(?Send)]
pub trait ApiBothExt {
    type Req: FromHttpRequest;
    type Res: Serialize;

    // this is just called from the router... don't override
    async fn router(ctx: ApiContext<HttpRequest>) -> ApiResponse {
        let ctx = ctx.convert_req().await?;
        let response_data = Self::handle(&ctx).await?;
        Ok(Self::response(&ctx, response_data).await)
    }

    // override this for main logic getting from a request to a response data
    async fn handle(ctx: &ApiContext<Self::Req>) -> ApiResult<Self::Res>;

    // and finally, override this to modify the response before returning
    // by default it will just return as json
    async fn response(_ctx: &ApiContext<Self::Req>, res: Self::Res) -> HttpResponse {
        any_to_json_response(&res, None).await
    }
}

#[async_trait(?Send)]
pub trait ApiResExt {
    type Res: Serialize;

    // this is just called from the router... don't override
    async fn router(ctx: ApiContext<HttpRequest>) -> ApiResponse {
        let response_data = Self::handle(&ctx).await?;
        Ok(Self::response(&ctx, response_data).await)
    }

    // override this for main logic to get a response data
    async fn handle(ctx: &ApiContext<HttpRequest>) -> ApiResult<Self::Res>;

    // and finally, override this to modify the response before returning
    // by default it will just return as json
    async fn response(_ctx: &ApiContext<HttpRequest>, res: Self::Res) -> HttpResponse {
        any_to_json_response(&res, None).await
    }
}

#[async_trait(?Send)]
pub trait ApiReqExt {
    type Req: FromHttpRequest;

    // this is just called from the router... don't override
    async fn router(ctx: ApiContext<HttpRequest>) -> ApiResponse {
        let ctx = ctx.convert_req().await?;
        let _ = Self::handle(&ctx).await?;
        Ok(Self::response(&ctx).await)
    }

    // override this for main logic handling the request data
    async fn handle(ctx: &ApiContext<Self::Req>) -> ApiResult<()>;

    // and finally, override this to modify the response before returning
    // by default it will just return empty
    async fn response(_ctx: &ApiContext<Self::Req>) -> HttpResponse {
        empty_response(None)
    }
}

#[async_trait(?Send)]
pub trait ApiEmptyExt {
    // this is just called from the router... don't override
    async fn router(ctx: ApiContext<HttpRequest>) -> ApiResponse {
        let _ = Self::handle(&ctx).await?;
        Ok(Self::response(&ctx).await)
    }

    // override this for main logic handling the request
    async fn handle(ctx: &ApiContext<HttpRequest>) -> ApiResult<()>;

    // and finally, override this to modify the response before returning
    // by default it will just return empty
    async fn response(_ctx: &ApiContext<HttpRequest>) -> HttpResponse {
        empty_response(None)
    }
}

// rarely used, extends ApiBoth and allows passing Extra data from the handle
// useful for dealing with cookies in the response when it's not derived
// from the response data
#[async_trait(?Send)]
pub trait ApiBothWithExtraExt {
    type Req: FromHttpRequest;
    type Res: Serialize;
    type Extra;

    // this is just called from the router... don't override
    async fn router(ctx: ApiContext<HttpRequest>) -> ApiResponse {
        let ctx = ctx.convert_req().await?;
        let (response_data, extra) = Self::handle(&ctx).await?;
        Ok(Self::response(&ctx, response_data, extra).await)
    }

    // override this for main logic getting from a request to a response
    async fn handle(ctx: &ApiContext<Self::Req>) -> ApiResult<(Self::Res, Self::Extra)>;

    // and finally, override this to modify the response before returning
    async fn response(
        ctx: &ApiContext<Self::Req>,
        res: Self::Res,
        extra: Self::Extra,
    ) -> HttpResponse;
}

// rarely used, extends ApiResDynRoute and allows passing Extra data from the handle
// useful for dealing with cookies in the response when it's not derived
// from the response data *and* where the route is dynamic (phew)
#[allow(unused)]
#[async_trait(?Send)]
pub trait ApiResDynRouteWithExtraExt {
    type Res: Serialize;
    type Extra;

    // this is just called from the router... don't override
    async fn router(&self, ctx: ApiContext<HttpRequest>) -> ApiResponse {
        let (response_data, extra) = self.handle(&ctx).await?;
        Ok(self.response(&ctx, response_data, extra).await)
    }

    // override this for main logic getting from a request to a response data
    async fn handle(&self, ctx: &ApiContext<HttpRequest>) -> ApiResult<(Self::Res, Self::Extra)>;

    // and finally, override this to modify the response before returning
    async fn response(
        &self,
        ctx: &ApiContext<HttpRequest>,
        res: Self::Res,
        extra: Self::Extra,
    ) -> HttpResponse;
}

// rarely used, extends ApiEmptyDynRoute and allows passing Extra data from the handle
// useful for dealing with cookies in the response when it's not derived
// from the response data *and* where the route is dynamic (phew)
#[async_trait(?Send)]
pub trait _ApiEmptyDynRouteWithExtraExt {
    type Extra;

    // this is just called from the router... don't override
    async fn router(&self, ctx: ApiContext<HttpRequest>) -> ApiResponse {
        let extra = self.handle(&ctx).await?;
        Ok(self.response(&ctx, extra).await)
    }

    // override this for main logic getting from a request to a response data
    async fn handle(&self, ctx: &ApiContext<HttpRequest>) -> ApiResult<Self::Extra>;

    // and finally, override this to modify the response before returning
    async fn response(&self, ctx: &ApiContext<HttpRequest>, extra: Self::Extra) -> HttpResponse;
}
