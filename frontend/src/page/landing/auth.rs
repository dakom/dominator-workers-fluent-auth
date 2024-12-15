mod register;
mod login;
pub mod actions;

use crate::prelude::*;
use register::Register;
use login::Login;
use std::collections::HashMap;
use web_sys::js_sys::{try_iter, Array};

pub fn render(auth_route: AuthRoute) -> Dom {
    match auth_route {
        AuthRoute::Register => Register::new().render(),
        AuthRoute::Login => Login::new().render(),
        _ => html!("div", {
            .text("404")
        })
    }
}

#[derive(Debug)]
struct TelegramUrlParams {
    pub data_check_string: String,
    pub data_check_hash: String,
    pub user_id: i64,
}

impl TelegramUrlParams {
    fn new() -> Result<Self> {
        let window = web_sys::window()
            .ok_or("No global `window` exists")
            .map_err(|e| anyhow!("{e:?}"))?;
        let location = window.location();

        let href = location.href().map_err(|e| anyhow!("{e:?}"))?;
        let url = web_sys::Url::new(&href).map_err(|e| anyhow!("{e:?}"))?;

        let params = url.search_params();

        let entries = params.entries();

        let iter = try_iter(&entries)
            .map_err(|e| anyhow!("{e:?}"))?
            .ok_or(anyhow!("entries is not iterable"))?;

        let mut params_map = HashMap::new();

        let mut hash = None;
        let mut user_id = None;

        for item in iter {
            let item = item.map_err(|e| anyhow!("{e:?}"))?;
            let pair = Array::from(&item);

            // Extract key and value
            let key = pair.get(0);
            let value = pair.get(1);

            // Convert JsValue to Rust String
            let key_str = key.as_string().ok_or(anyhow!("key is not a string"))?;
            let value_str = value.as_string().ok_or(anyhow!("value is not a string"))?;

            if key_str == "hash" {
                hash = Some(value_str);
            } else {
                if key_str == "id" {
                    user_id = Some(value_str.clone());
                }
                params_map.insert(key_str, value_str);
            }
        }

        let mut entries: Vec<_> = params_map.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));

        let data_check_string = entries
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        let user_id = user_id.map(|s| s.parse::<i64>()).transpose()?;

        Ok(Self {
            data_check_string,
            data_check_hash: hash.ok_or(anyhow!("hash not found"))?,
            user_id: user_id.ok_or(anyhow!("user_id not found"))?,
        })
    }
}
