use std::{sync::LazyLock, time::Duration};

use crate::context::ContentLanguage;

const MS_PER_MIN: u64 = 1000 * 60;
const MS_PER_HOUR: u64 = 60 * MS_PER_MIN;
const MS_PER_DAY: u64 = 24 * MS_PER_HOUR;
const MS_PER_WEEK: u64 = 7 * MS_PER_DAY;

pub static AUTH_TOKEN_SIGNIN_EXPIRES_DURATION: LazyLock<Duration> =
    LazyLock::new(|| Duration::from_millis(MS_PER_WEEK * 2));

pub static AUTH_TOKEN_VERIFY_EMAIL_EXPIRES_DURATION: LazyLock<Duration> =
    LazyLock::new(|| Duration::from_millis(MS_PER_WEEK * 2));

// the key is never used in isolation, rather it's used in conjunction with the id
// 16 bytes of randomness is more than enough
pub const AUTH_TOKEN_KEY_LENGTH: usize = 16;

cfg_if::cfg_if! {
    if #[cfg(debug_assertions)] {
        pub const API_DOMAIN:&'static str = "https://terrier-direct-openly.ngrok-free.app";
        pub const API_ROOT_PATH: &'static str = "";
        pub const DEFAULT_CONTENT_LANG:ContentLanguage = ContentLanguage::English;
        pub const ALLOWED_ORIGINS: &[&'static str] = &["http://localhost:8080", "http://127.0.0.1:8080"];
        pub const DB_BINDING:&'static str = "DB-auth-demo";
        pub const KV_BINDING_AUTH_TOKEN:&'static str = "KV-auth-demo-auth-token";
        pub const FRONTEND_URL:&'static str = "http://localhost:8080";
        pub static EMAIL_SETTINGS:LazyLock<Option<EmailSettings>> = LazyLock::new(|| None);
    } else {
        pub const ENV_KEY_ADMIN_CODE:&'static str = "ADMIN_CODE";
        pub const API_DOMAIN:&'static str = "https://omi-assist-api-prod.dakom.workers.dev";
        pub const API_ROOT_PATH: &'static str = "";
        pub const DEFAULT_CONTENT_LANG:ContentLanguage = ContentLanguage::English;
        pub const ALLOWED_ORIGINS: &[&'static str] = &["https://omi-assist.pages.dev"];
        pub const DB_BINDING:&'static str = "DB-auth-demo";
        pub const KV_BINDING_AUTH_TOKEN:&'static str = "KV-auth-demo-auth-token";
        pub const FRONTEND_URL:&'static str = "https://omi-assist.pages.dev";
        pub static EMAIL_SETTINGS:LazyLock<Option<EmailSettings>> = LazyLock::new(|| {
            Some(EmailSettings {
                
            })
        });
    }
}

pub const DB_TABLE: DbTable = DbTable {
    user_account: "user_account",
    user_account_email: "user_account_email",
    user_role_info: "user_role_info",
    user_roles: "user_roles",
};

pub struct DbTable {
    pub user_account: &'static str,
    pub user_account_email: &'static str,
    pub user_role_info: &'static str,
    pub user_roles: &'static str,
}

// TODO - sendgrid etc.
pub struct EmailSettings {
}