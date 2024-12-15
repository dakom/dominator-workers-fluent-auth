use crate::{api_ext::*, telegram::TelegramBot, ApiContext};
use async_trait::async_trait;
use info::{ServerInfo, ServerInfoResponse};
use shared::{api::*, backend::result::ApiResult};
use worker::HttpRequest;

#[async_trait(?Send)]
impl ApiResExt for ServerInfo {
    type Res = <Self as ApiRes>::Res;

    async fn handle(_ctx: &ApiContext<HttpRequest>) -> ApiResult<ServerInfoResponse> {
        let tg = TelegramBot::new(&_ctx.env);

        let telegram_bot = tg.get_me().await?;
        let telegram_webhook = tg.get_webhook().await?;

        let res = ServerInfoResponse {
            version: "0.1.0".to_string(),
            telegram_bot,
            telegram_webhook,
        };
        Ok(res)
    }
}
