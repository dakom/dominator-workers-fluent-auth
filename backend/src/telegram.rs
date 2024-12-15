use serde::{de::DeserializeOwned, Deserialize, Serialize};
use shared::{
    api::telegram::{TelegramBotError, TelegramMessage, TelegramUser, TelegramWebHookInfo},
    backend::result::{ApiError, ApiResult},
};
use web_sys::FormData;
use worker::{Env, Fetch, Request};

use crate::config::{ENV_KEY_TELEGRAM_BOT_TOKEN, ENV_KEY_TELEGRAM_WEBHOOK_SECRET};

pub struct TelegramBot {
    pub token: String,
    pub webhook_secret: String,
}

impl TelegramBot {
    pub fn new(env: &Env) -> Self {
        Self {
            token: env.secret(ENV_KEY_TELEGRAM_BOT_TOKEN).unwrap().to_string(),
            webhook_secret: env
                .secret(ENV_KEY_TELEGRAM_WEBHOOK_SECRET)
                .unwrap()
                .to_string(),
        }
    }

    pub async fn get_me(&self) -> ApiResult<TelegramUser> {
        self.make_request_empty("getMe").await
    }

    pub async fn set_webhook(&self, url: String) -> ApiResult<()> {
        let form_data = FormData::new()?;
        form_data.append_with_str("url", &url)?;
        form_data.append_with_str("secret_token", &self.webhook_secret)?;
        form_data.append_with_str("drop_pending_updates", "true")?;

        let success = self
            .make_request_params::<bool>("setWebhook", form_data)
            .await?;

        if success {
            Ok(())
        } else {
            Err(ApiError::Telegram(TelegramBotError::Internal(
                "Failed to set webhook".to_string(),
            )))
        }
    }

    pub async fn get_webhook(&self) -> ApiResult<TelegramWebHookInfo> {
        self.make_request_empty("getWebhookInfo").await
    }

    pub async fn send_message(&self, chat_id: i64, text: &str) -> ApiResult<TelegramMessage> {
        let form_data = FormData::new()?;
        form_data.append_with_str("chat_id", &chat_id.to_string())?;
        form_data.append_with_str("text", text)?;

        self.make_request_params("sendMessage", form_data).await
    }

    async fn make_request_empty<T: DeserializeOwned>(&self, method: &str) -> ApiResult<T> {
        let url = format!("https://api.telegram.org/bot{}/{}", self.token, method);
        let request = Request::new(&url, worker::Method::Get)
            .map_err(|e| ApiError::Telegram(TelegramBotError::Internal(e.to_string())))?;
        let mut res = Fetch::Request(request)
            .send()
            .await
            .map_err(|e| ApiError::Telegram(TelegramBotError::Internal(e.to_string())))?;
        let text = res.text().await?;

        tracing::info!("Response: {}", text);

        let json: TelegramResult<T> = serde_json::from_str(&text)
            .map_err(|e| ApiError::Telegram(TelegramBotError::Internal(e.to_string())))?;

        if json.ok {
            Ok(json.result)
        } else {
            Err(ApiError::Telegram(TelegramBotError::Internal(
                "Telegram API error".to_string(),
            )))
        }
    }

    async fn make_request_params<T: DeserializeOwned>(
        &self,
        method: &str,
        form_data: FormData,
    ) -> ApiResult<T> {
        let url = format!("https://api.telegram.org/bot{}/{}", self.token, method);

        tracing::info!("Request: {}", url);
        tracing::info!("{:?}", form_data);

        let mut init = worker::RequestInit::new();
        init.with_body(Some(form_data.into()));
        init.with_method(worker::Method::Post);

        let request = Request::new_with_init(&url, &init)
            .map_err(|e| ApiError::Telegram(TelegramBotError::Internal(e.to_string())))?;
        let mut res = Fetch::Request(request)
            .send()
            .await
            .map_err(|e| ApiError::Telegram(TelegramBotError::Internal(e.to_string())))?;

        let text = res.text().await?;

        tracing::info!("Response: {}", text);

        let json: TelegramResult<T> = serde_json::from_str(&text)
            .map_err(|e| ApiError::Telegram(TelegramBotError::Internal(e.to_string())))?;

        if json.ok {
            Ok(json.result)
        } else {
            Err(ApiError::Telegram(TelegramBotError::Internal(
                "Telegram API error".to_string(),
            )))
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct TelegramResult<T> {
    ok: bool,
    result: T,
}
