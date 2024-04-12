use crate::{config::{DKIM_DOMAIN, DKIM_SELECTOR, FRONTEND_DOMAIN, FRONTEND_ROOT_PATH, MAILER_ADDRESS, MAILER_NAME}, context::ContentLanguage, prelude::*};
use serde::Serialize;
use shared::frontend::route::{Route as FrontendRoute, Landing as FrontendLanding, AuthRoute as FrontendAuthRoute};
use web_sys::{js_sys, Headers, RequestInit, WorkerGlobalScope};
use worker::{console_log, console_warn};

pub enum MailerKind {
    EmailVerification {
        oob_token_id: String,
        oob_token_key: String,
    },
    PasswordReset {
        oob_token_id: String,
        oob_token_key: String,
    }
}

#[cfg(debug_assertions)]
pub async fn send(ctx: &ApiContext, address: &str, kind: MailerKind) -> ApiResult<()> {
    let oob_url = match kind {
        MailerKind::EmailVerification { oob_token_id, oob_token_key } => {
            // and the *frontend* url to email
            FrontendRoute::Landing(FrontendLanding::Auth(FrontendAuthRoute::VerifyEmailConfirm{
                oob_token_id, 
                oob_token_key
            })).link(FRONTEND_DOMAIN, FRONTEND_ROOT_PATH)
        },
        MailerKind::PasswordReset { oob_token_id, oob_token_key } => {
            FrontendRoute::Landing(FrontendLanding::Auth(FrontendAuthRoute::PasswordResetConfirm {
                oob_token_id, 
                oob_token_key
            })).link(FRONTEND_DOMAIN, FRONTEND_ROOT_PATH)
        }
    };

    worker::console_log!("in non-debug, email would be sent to {} with link: {}", address, oob_url);

    Ok(())
}


#[cfg(not(debug_assertions))]
pub async fn send(ctx: &ApiContext, address: &str, kind: MailerKind) -> ApiResult<()> {

    let (subject, content) = match kind {
        MailerKind::EmailVerification { oob_token_id, oob_token_key } => {
            // and the *frontend* url to email
            let oob_url = FrontendRoute::Landing(FrontendLanding::Auth(FrontendAuthRoute::VerifyEmailConfirm{
                oob_token_id, 
                oob_token_key
            })).link(FRONTEND_DOMAIN, FRONTEND_ROOT_PATH);

            let subject = match ctx.lang {
                ContentLanguage::English => "Verify your email".to_string(),
                ContentLanguage::Hebrew => "אמת את האימייל שלך".to_string()
            };

            let content = match ctx.lang {
                ContentLanguage::English => format!("Click here to verify your email: {}", oob_url),
                ContentLanguage::Hebrew => format!("לחץ כאן כדי לאמת את האימייל שלך: {}", oob_url)
            };

            (subject, content)
        },
        MailerKind::PasswordReset { oob_token_id, oob_token_key } => {
            // and the *frontend* url to email
            let oob_url = FrontendRoute::Landing(FrontendLanding::Auth(FrontendAuthRoute::PasswordResetConfirm {
                oob_token_id, 
                oob_token_key
            })).link(FRONTEND_DOMAIN, FRONTEND_ROOT_PATH);

            let subject = match ctx.lang {
                ContentLanguage::English => "Reset your password".to_string(),
                ContentLanguage::Hebrew => "אפס את הסיסמה שלך".to_string()
            };

            let content = match ctx.lang {
                ContentLanguage::English => format!("Click here to reset your password: {}", oob_url),
                ContentLanguage::Hebrew => format!("לחץ כאן כדי לאפס את הסיסמה שלך: {}", oob_url)
            };

            (subject, content)
        }
    };

    let mut init = RequestInit::new();
    init.method("POST");
    let headers = Headers::new().unwrap();
    headers.set("content-type", "application/json")?;
    init.headers(&headers);

    let body = serde_json::to_string_pretty(&MailChannelsRequest::new(ctx, address.to_string(), None, subject, content)).map_err(|err| err.to_string())?;
    init.body(Some(&JsValue::from_str(&body)));

    let req = Request::new_with_str_and_init("https://api.mailchannels.net/tx/v1/send", &init).unwrap();

    let promise = js_sys::global().unchecked_into::<WorkerGlobalScope>().fetch_with_request(&req);
    let resp = JsFuture::from(promise).await.map(|resp| resp.unchecked_into::<Response>());

    match resp {
        Ok(resp) => {
            if resp.ok() {
                console_log!("email sent to {}", address);
            } else {
                console_warn!("email failed to send to {}, status code: {}, status text: {}", address, resp.status(), resp.status_text());
                if let Some(err_body) = JsFuture::from(resp.text()?).await?.as_string() {
                    console_warn!("{}", err_body);
                }
                return Err(format!("email failed to send to {}, status code: {}, status text: {}", address, resp.status(), resp.status_text()).into());
            }
        },
        Err(err) => {
            console_warn!("email failed to send to {}, error: {:?}", address, err);
            return Err(format!("email failed to send to {}, error: {:?}", address, err).into());
        }
    }

    Ok(())
}

#[derive(Serialize, Debug)]
struct MailChannelsRequest {
    pub personalizations: Vec<Personalization>,
    pub from: Person,
    pub subject: String,
    pub content: Vec<Content>
}

impl MailChannelsRequest {
    pub fn new(ctx: &ApiContext, to_email: String, to_name: Option<String>, subject: String, content: String) -> Self {
        Self {
            personalizations: vec![Personalization::new(ctx, to_email, to_name)],
            from: Person { email: MAILER_ADDRESS.to_string(), name: Some(MAILER_NAME.to_string()) },
            subject,
            content: vec![Content::new(ctx, content)]
        }
    }
}

#[derive(Serialize, Debug)]
struct Personalization {
    pub to: Vec<Person>,
    pub dkim_domain: Option<&'static str>,
    pub dkim_selector: Option<&'static str>,
    pub dkim_private_key: Option<String>,
}

impl Personalization {
    pub fn new(ctx: &ApiContext, email: String, name: Option<String>) -> Self {
        Self {
            to: vec![Person { email, name }],
            dkim_domain: DKIM_DOMAIN, 
            dkim_selector: DKIM_SELECTOR,
            dkim_private_key: if DKIM_DOMAIN.is_some() && DKIM_SELECTOR.is_some() { Some(get_secret(&ctx.env, "DKIM_PRIVATE_KEY").unwrap()) } else { None } 
        }
    }
}

#[derive(Serialize, Debug)]
struct Person {
    pub email: String,
    pub name: Option<String>,
}

#[derive(Serialize, Debug)]
struct Content {
    pub r#type: &'static str,
    pub value: String,
}

impl Content {
    pub fn new(ctx: &ApiContext, value: String) -> Self {
        let direction = match ctx.lang {
            ContentLanguage::English => "ltr",
            ContentLanguage::Hebrew => "rtl"
        };

        Self {
            r#type: "text/html; charset=UTF-8",
            value: format!(r#"<html><body dir="{direction}">{value}</body></html>"#) 
        }
    }
}