use std::{future::Future, pin::Pin};

use crate::config::FRONTEND_URL;
use crate::{
    api_ext::*,
    db::{destination::TelegramDestinationDb, user::TelegramAccount},
    json_body_to_any,
    telegram::TelegramBot,
    ApiContext,
};
use action::{ActionDestinationId, ActionDestinationKind};
use async_trait::async_trait;
use shared::{
    api::*,
    backend::result::{ApiError, ApiResult},
};
use telegram::{
    TelegramBotCommand, TelegramBotError, TelegramMessage, TelegramOmiCommand, TelegramWebHook,
    TelegramWebHookRequest,
};
use worker::HttpRequest;

#[async_trait(?Send)]
impl ApiReqExt for TelegramWebHook {
    type Req = <Self as ApiReq>::Req;

    async fn handle(ctx: &ApiContext<TelegramWebHookRequest>) -> ApiResult<()> {
        if let Some(msg) = ctx.req.message.as_ref() {
            match TelegramBotCommand::try_from(msg) {
                Ok(command) => {
                    if msg.from.is_bot {
                        tracing::info!("ignoring bot message");
                        return Ok(());
                    }

                    match command {
                        TelegramBotCommand::Start => return handle_start(ctx, &msg).await,
                        TelegramBotCommand::Omi(omi_command) => match omi_command {
                            TelegramOmiCommand::LinkDm => return handle_link(ctx, &msg).await,
                            TelegramOmiCommand::LinkGroup => return handle_link(ctx, &msg).await,
                        },
                    }
                }
                Err(err) => match err {
                    TelegramBotError::UnknownCommand(text) => {
                        tracing::info!("unknown command: {:#?}", text);
                        Ok(())
                    }
                    TelegramBotError::OmiCommand(text) => {
                        worker::console_warn!("omi command error: {:#?}", text);
                        Ok(())
                    }
                    TelegramBotError::UnsupportedMessage => {
                        tracing::info!("unsupported message");
                        Ok(())
                    }
                    _ => Err(ApiError::Telegram(err)),
                },
            }
        } else {
            Ok(())
        }
    }
}

impl FromHttpRequest for TelegramWebHookRequest {
    fn from_request(
        env: worker::Env,
        req: HttpRequest,
    ) -> Pin<Box<dyn Future<Output = ApiResult<Self>>>> {
        Box::pin(async move {
            match req.headers().get("X-Telegram-Bot-Api-Secret-Token") {
                None => Err(ApiError::Telegram(TelegramBotError::Unauthorized)),
                Some(value) => {
                    let env_secret = env.secret("TELEGRAM_WEBHOOK_SECRET").unwrap().to_string();

                    if env_secret.is_empty() || value.to_str().unwrap_or_default() != &env_secret {
                        Err(ApiError::Telegram(TelegramBotError::Unauthorized))
                    } else {
                        json_body_to_any::<Self>(req.into_body()).await
                    }
                }
            }
        })
    }
}

async fn handle_start(
    ctx: &ApiContext<TelegramWebHookRequest>,
    message: &TelegramMessage,
) -> ApiResult<()> {
    let tg = TelegramBot::new(&ctx.env);

    let text = format!("Welcome to Omi Assist! You can manage your settings at {FRONTEND_URL}");

    let res = tg.send_message(message.chat.id, &text).await?;

    tracing::info!("response id: {}", res.message_id);

    Ok(())
}

async fn handle_link(
    ctx: &ApiContext<TelegramWebHookRequest>,
    message: &TelegramMessage,
) -> ApiResult<()> {
    let tg = TelegramBot::new(&ctx.env);

    let tg_user = match TelegramAccount::load(&ctx.env, message.from.id).await {
        Ok(user) => user,
        Err(_) => {
            let _ = tg
                .send_message(
                    message.chat.id,
                    &format!("you need to first register an account at {FRONTEND_URL}"),
                )
                .await?;
            return Ok(());
        }
    };

    if tg_user.first_name != message.from.first_name || tg_user.username != message.from.username {
        if let Err(err) = TelegramAccount::update_name(
            &ctx.env,
            tg_user.id,
            &message.from.first_name,
            message.from.username.as_deref(),
        )
        .await
        {
            tracing::warn!(
                "failed to update user name: {:#?}, but it was a side-effect optimization anyway",
                err
            );
        }
    }

    if TelegramDestinationDb::exists_by_user_chat_id(&ctx.env, &tg_user.user_id, message.chat.id)
        .await?
    {
        let msg = match message.chat.chat_type {
            telegram::TelegramChatType::Private => "You have already linked this chat",
            _ => "You have already linked this group",
        };
        let _ = tg.send_message(message.chat.id, msg).await?;
        return Ok(());
    }

    let kind = match message.chat.chat_type {
        telegram::TelegramChatType::Private => ActionDestinationKind::TelegramDm {
            chat_id: message.chat.id,
        },
        _ => ActionDestinationKind::TelegramGroup {
            chat_id: message.chat.id,
        },
    };

    let name = match kind {
        ActionDestinationKind::TelegramDm { .. } => match message.from.username.clone() {
            None => message.from.first_name.clone(),
            Some(username) => format!("{} (@{})", message.from.first_name, username),
        },
        ActionDestinationKind::TelegramGroup { .. } => match message.chat.title.clone() {
            None => format!("Group {}", message.chat.id),
            Some(title) => title,
        },
    };

    let destination_id = ActionDestinationId::new(uuid::Uuid::now_v7());
    TelegramDestinationDb::insert(&ctx.env, &destination_id, &tg_user.user_id, &name, kind).await?;

    let msg = match message.chat.chat_type {
        telegram::TelegramChatType::Private => "Chat linked successfully",
        _ => "Group linked successfully",
    };

    let _ = tg.send_message(message.chat.id, msg).await?;

    Ok(())
}
