use crate::{api_ext::*, ApiContext};
use async_trait::async_trait;
use info::{ServerInfo, ServerInfoResponse};
use shared::{api::*, backend::result::ApiResult};
use worker::HttpRequest;

#[async_trait(?Send)]
impl ApiResExt for ServerInfo {
    type Res = <Self as ApiRes>::Res;

    async fn handle(_ctx: &ApiContext<HttpRequest>) -> ApiResult<ServerInfoResponse> {
        let res = ServerInfoResponse {
            version: "0.1.0".to_string(),
        };
        Ok(res)
    }
}
