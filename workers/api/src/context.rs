use shared::user::UserId;
use unic_langid::LanguageIdentifier;
use worker::{Context, Env};
use crate::{auth::AuthUser, config::DEFAULT_CONTENT_LANG};

pub struct ApiContext {
    pub req: web_sys::Request,
    pub env: Env,
    pub cf_ctx: Context,
    pub user: Option<AuthUser>,
    pub lang: ContentLanguage,
}

impl ApiContext {
    pub fn new(req: web_sys::Request, env: Env, cf_ctx: Context, user: Option<AuthUser>) -> Self {
        let lang_header = req.headers()
            .get("Content-Language")
            .unwrap()
            .or_else(|| req.headers().get("Accept-Language").unwrap());

        let lang = lang_header
            .and_then(|lang_header| 
                lang_header
                    .split(|c| c == ',' || c == ';')
                    .next()
                    .and_then(|lang_str| LanguageIdentifier::from_bytes(lang_str.as_bytes()).ok())
            )
            .and_then(ContentLanguage::try_from_lang_id)
            .unwrap_or(DEFAULT_CONTENT_LANG);

        Self {
            req,
            env,
            cf_ctx,
            user,
            lang
        }
    }

    pub fn uid_unchecked(&self) -> UserId {
        self.user.as_ref().unwrap().account.id.clone()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ContentLanguage {
    English,
    Hebrew
}

impl ContentLanguage {
    pub fn to_lang_id(&self) -> LanguageIdentifier {
        match self {
            Self::English => LanguageIdentifier::from_bytes("en".as_bytes()).unwrap(),
            Self::Hebrew => LanguageIdentifier::from_bytes("he".as_bytes()).unwrap()
        }
    }

    const fn all() -> [Self; 2] {
        [Self::English, Self::Hebrew]
    }

    pub fn try_from_lang_id(lang_id: LanguageIdentifier) -> Option<Self> {
        for lang in Self::all().iter() {
            if lang.to_lang_id().matches(&lang_id, true, true) {
                return Some(*lang);
            }
        }
        None
    }
}