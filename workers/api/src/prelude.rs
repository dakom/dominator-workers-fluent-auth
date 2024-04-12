pub use shared::backend::{
    result::*,
    worker::*,
};
pub use worker::{
    Env,
    DurableObject,
    wasm_bindgen::prelude::*,
    wasm_bindgen_futures::JsFuture,
};
pub use crate::{
    helpers::*,
    context::ApiContext,
};

pub use web_sys::{Response, Request};
pub type ApiResponse = ApiResult<Response>;

// different for now due to Result/Response conflict...
// maybe one day they can be unified
pub mod durable_object {
    pub use shared::{
        user::UserId,
        backend::result::*
    };
    pub use worker::{
        async_trait, durable_object, wasm_bindgen, wasm_bindgen_futures, Env, Headers, Request, Response, State,
        Stub, RequestInit, Storage,
        wasm_bindgen::prelude::*,
    };

    pub use base64::Engine;
    pub use rand::Rng;
    pub use std::time::Duration;
    pub use crate::helpers::*;
}