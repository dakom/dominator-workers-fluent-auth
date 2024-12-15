use crate::context::ContentLanguage;

const MS_PER_MIN: u64 = 1000 * 60;
const MS_PER_HOUR: u64 = 60 * MS_PER_MIN;
const MS_PER_DAY: u64 = 24 * MS_PER_HOUR;
const MS_PER_WEEK: u64 = 7 * MS_PER_DAY;

pub const AUTH_TOKEN_SIGNIN_EXPIRES: u64 = MS_PER_WEEK * 2;

// the key is never used in isolation, rather it's used in conjunction with the id
// 16 bytes of randomness is more than enough
pub const AUTH_TOKEN_KEY_LENGTH: usize = 16;

cfg_if::cfg_if! {
    if #[cfg(debug_assertions)] {
        pub const ENV_KEY_TELEGRAM_BOT_TOKEN:&'static str = "TELEGRAM_BOT_TOKEN_DEV";
        pub const ENV_KEY_TELEGRAM_AUTH_TOKEN:&'static str = "TELEGRAM_BOT_TOKEN"; // always prod
        pub const ENV_KEY_TELEGRAM_WEBHOOK_SECRET:&'static str = "TELEGRAM_WEBHOOK_SECRET";
        pub const API_DOMAIN:&'static str = "https://terrier-direct-openly.ngrok-free.app";
        pub const API_ROOT_PATH: &'static str = "";
        pub const DEFAULT_CONTENT_LANG:ContentLanguage = ContentLanguage::English;
        pub const ALLOWED_ORIGINS: &[&'static str] = &["http://localhost:8080", "http://127.0.0.1:8080"];
        pub const DB_BINDING:&'static str = "DB-omi-assist";
        pub const KV_BINDING_AUTH_TOKEN_SIGNIN:&'static str = "KV-omi-auth-token-login";
        pub const FRONTEND_URL:&'static str = "http://localhost::8080";
    } else {
        pub const ENV_KEY_TELEGRAM_BOT_TOKEN:&'static str = "TELEGRAM_BOT_TOKEN";
        pub const ENV_KEY_TELEGRAM_AUTH_TOKEN:&'static str = "TELEGRAM_BOT_TOKEN";
        pub const ENV_KEY_TELEGRAM_WEBHOOK_SECRET:&'static str = "TELEGRAM_WEBHOOK_SECRET";
        pub const ENV_KEY_ADMIN_CODE:&'static str = "ADMIN_CODE";
        pub const API_DOMAIN:&'static str = "https://omi-assist-api-prod.dakom.workers.dev";
        pub const API_ROOT_PATH: &'static str = "";
        pub const DEFAULT_CONTENT_LANG:ContentLanguage = ContentLanguage::English;
        pub const ALLOWED_ORIGINS: &[&'static str] = &["https://omi-assist.pages.dev"];
        pub const DB_BINDING:&'static str = "DB-omi-assist";
        pub const KV_BINDING_AUTH_TOKEN_SIGNIN:&'static str = "KV-omi-auth-token-login";
        pub const FRONTEND_URL:&'static str = "https://omi-assist.pages.dev";
    }
}

pub const DB_TABLE: DbTable = DbTable {
    user_account: "user_account",
    telegram_account: "telegram_account",
    telegram_destination: "telegram_destination",
    telegram_action: "telegram_action",
};

pub struct DbTable {
    pub user_account: &'static str,
    pub telegram_account: &'static str,
    pub telegram_destination: &'static str,
    pub telegram_action: &'static str,
}
