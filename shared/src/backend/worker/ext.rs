use std::future::Future;
use serde::de::DeserializeOwned;
use wasm_bindgen::JsValue;
use worker::{
    wasm_bindgen_futures::JsFuture,
    worker_sys::web_sys::{Request, Response, ResponseInit},
    serde_wasm_bindgen
};

pub trait ResponseExt {
    fn new_empty() -> Response {
        Response::new().unwrap()
    }
    fn new_empty_status(status_code: u16) -> Response {
        let mut init = ResponseInit::new();
        init.status(status_code);
        Response::new_with_opt_str_and_init(None, &init).unwrap()
    }
    fn new_json<T: serde::Serialize>(data: T) -> Response {
        let json = serde_json::to_string_pretty(&data).unwrap();
        let req = Response::new_with_opt_str(Some(&json)).unwrap();
        req.headers().set("Content-Type", "application/json").unwrap();
        req
    }

    fn new_json_status<T: serde::Serialize>(data: T, status_code: u16) -> Response {
        let json = serde_json::to_string_pretty(&data).unwrap();
        let mut init = ResponseInit::new();
        init.status(status_code);
        let req = Response::new_with_opt_str_and_init(Some(&json), &init).unwrap();
        req.headers().set("Content-Type", "application/json").unwrap();
        req
    }

    // raw Response::redirect() causes a "cannot write immutable headers" error in CF
    fn new_temp_redirect(url: &str) -> Response {
        let mut init = ResponseInit::new();
        init.status(302);
        let req = Response::new_with_opt_str_and_init(None, &init).unwrap();
        req.headers().set("Location", url).unwrap();
        req
    }

    fn try_into_json<T: DeserializeOwned>(&self) -> impl Future<Output = std::result::Result::<T, JsValue>>;
}

impl ResponseExt for Response {
    async fn try_into_json<T: DeserializeOwned>(&self) -> std::result::Result::<T, JsValue> {
        let data = JsFuture::from(self.json()?).await?;
        serde_wasm_bindgen::from_value(data).map_err(|err| JsValue::from(err))
    }
}

pub trait RequestExt {
    fn try_from_json<T: DeserializeOwned>(&self) -> impl Future<Output = std::result::Result::<T, JsValue>>;
}

impl RequestExt for Request {
    async fn try_from_json<T: DeserializeOwned>(&self) -> std::result::Result::<T, JsValue> {
        let data = JsFuture::from(self.json()?).await?;
        serde_wasm_bindgen::from_value(data).map_err(|err| JsValue::from(err))
    }
}