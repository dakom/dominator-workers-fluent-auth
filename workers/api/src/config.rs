use crate::context::ContentLanguage;

const MS_PER_MIN:u64 = 1000 * 60;
const MS_PER_HOUR:u64 = 60 * MS_PER_MIN;
const MS_PER_DAY:u64 = 24 * MS_PER_HOUR;
const MS_PER_WEEK:u64 = 7 * MS_PER_DAY;

pub const AUTH_SIGNIN_TOKEN_EXPIRES:u64 = MS_PER_WEEK * 2;
pub const AUTH_RESET_PASSWORD_TOKEN_EXPIRES:u64 = MS_PER_HOUR;
pub const AUTH_VERIFY_EMAIL_TOKEN_EXPIRES:u64 = MS_PER_DAY * 3;
pub const AUTH_OPEN_ID_SESSION_EXPIRES:u64 = MS_PER_HOUR;

// the key is never used in isolation, rather it's used in conjunction with the id
// 16 bytes of randomness is more than enough
pub const AUTH_TOKEN_KEY_LENGTH:usize = 16;

// just a random password when registering oauth users for the first time
pub const OAUTH_REGISTER_PASSWORD_LENGTH:usize = 32;

cfg_if::cfg_if! {
    if #[cfg(debug_assertions)] {
        pub const FRONTEND_DOMAIN:&'static str = "http://localhost:8080";
        pub const FRONTEND_ROOT_PATH: &'static str = "";
        pub const API_DOMAIN:&'static str = "http://localhost:8787";
        pub const API_ROOT_PATH: &'static str = "";
        pub const DKIM_DOMAIN:&'static str = "example.com";
        pub const DKIM_SELECTOR:&'static str = "mailchannels";
        pub const MAILER_ADDRESS:&'static str = "mailer@example.com";
        pub const MAILER_NAME:&'static str = "Demo Mailer";
        pub const DEFAULT_CONTENT_LANG:ContentLanguage = ContentLanguage::English;
        pub const ALLOWED_ORIGINS: &[&'static str] = &["http://localhost:8080", "http://127.0.0.1:8080"];
        pub const SEND_EMAIL: bool = false;
    } else {
        pub const FRONTEND_DOMAIN:&'static str = "https://example.pages.dev";
        pub const FRONTEND_ROOT_PATH: &'static str = "";
        pub const API_DOMAIN:&'static str = "https://api-prod.example.workers.dev";
        pub const API_ROOT_PATH: &'static str = "";
        pub const DKIM_DOMAIN:&'static str = "example.com";
        pub const DKIM_SELECTOR:&'static str = "mailchannels";
        pub const MAILER_ADDRESS:&'static str = "mailer@example.com";
        pub const MAILER_NAME:&'static str = "Demo Mailer";
        pub const DEFAULT_CONTENT_LANG:ContentLanguage = ContentLanguage::English;
        pub const ALLOWED_ORIGINS: &[&'static str] = &["https://example.com"];
        pub const SEND_EMAIL: bool = true;
    }
}

pub const DB_TABLE:DbTable = DbTable {
    user_account: "user_account",
};

pub struct DbTable {
    pub user_account: &'static str,
}