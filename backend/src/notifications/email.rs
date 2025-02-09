use shared::{backend::result::ApiResult, frontend::route::{AuthRoute, Landing, Route as FrontendRoute}, user::UserId};
use web_sys::console;
use worker::{console_debug, console_error, console_log, Env};

use crate::{config::{EMAIL_SETTINGS, FRONTEND_URL}, kv::auth::AuthKv};

pub enum EmailNotification {
    VerifyEmail { uid: UserId },
}

impl EmailNotification {
    pub async fn send(self, env: &Env) -> ApiResult<()> {

        match self {
            EmailNotification::VerifyEmail { uid } => {
                let token_data = AuthKv::create_verify_email(
                    env, 
                    uid, 
                ).await?;

                let url = format!("{}/{}", FRONTEND_URL, FrontendRoute::Landing(Landing::Auth(AuthRoute::VerifyEmailConfirm { token_id: token_data.id, token_key: token_data.key })));

                match *EMAIL_SETTINGS {
                    Some(_) => {
                        console_error!("TODO: send email");
                    },
                    None => {
                        console_log!("Click here to verify your email: {}", url);
                    },
                }
            }
        }

        Ok(())
    }
}