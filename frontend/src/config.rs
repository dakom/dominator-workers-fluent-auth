use awsm_web::env::env_var;

use crate::prelude::*;

#[derive(Debug)]
pub struct Config {
    // the part of the url that is not the domain
    // e.g. in http://example.com/foo/bar, this would be "foo" if we want
    // all parsing to start from /bar
    // it's helpful in shared hosting environments where the app is not at the root
    pub frontend_domain: &'static str,
    pub frontend_root_path: &'static str,
    pub media_root: &'static str,
    pub default_lang: Option<&'static str>,
    pub api_domain: &'static str,
    pub api_root_path: &'static str,
    // see usage and comments in auth, this is fine
    pub auth_login_key_storage_name: &'static str,
    // see usage and comments in auth, this is fine
    pub argon2_global_salt: &'static [u8],
}

impl Config {
    pub fn app_image_url(&self, path: &str) -> String {
        format!("{}/{}", self.media_root, path)
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "dev")] {
        pub const CONFIG: LazyLock<Config> = LazyLock::new(|| {
            Config {
                frontend_domain: "http://localhost:8080",
                frontend_root_path: "",
                media_root: "http://localhost:9000",
                //default_lang: Some("he-IL")
                default_lang: None,
                api_domain: "http://localhost:8787",
                api_root_path: "",
                auth_login_key_storage_name: "demo_auth_login_key",
                argon2_global_salt: b"demo",
            }
        });
    } else {
        pub const CONFIG: LazyLock<Config> = LazyLock::new(|| {
            Config {
                frontend_domain: "https://dominator-workers-fluent-auth.pages.dev",
                frontend_root_path: "",
                media_root: "/media",
                default_lang: None,
                api_domain: "https://dominator-workers-fluent-auth-api-prod.dakom.workers.dev",
                api_root_path: "",
                auth_login_key_storage_name: "demo_auth_login_key",
                argon2_global_salt: b"demo",
            }
        });
    }
}

#[allow(dead_code)]
fn get_env(name: &str) -> Option<String> {
    match env_var(name) {
        Ok(value) => {
            if value.is_empty() {
                None
            } else {
                Some(value)
            }
        }
        Err(_) => None,
    }
}
