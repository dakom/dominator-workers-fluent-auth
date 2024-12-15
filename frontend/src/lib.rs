pub mod api_ext;
pub mod atoms;
pub mod auth;
pub mod config;
pub mod error;
pub mod locale;
pub mod page;
pub mod prelude;
pub mod route;
pub mod theme;
pub mod util;

use shared::logger::init_logger;

use crate::prelude::*;

#[wasm_bindgen(start)]
pub async fn run() -> Result<(), JsValue> {
    init_logger();

    theme::stylesheet::init();

    dominator::append_dom(&dominator::body(), route::render());

    Ok(())
}
