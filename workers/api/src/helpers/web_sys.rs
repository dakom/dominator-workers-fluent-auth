#[cfg(not(debug_assertions))]
pub mod web_sys_debug {
    pub async fn debug_request(req: web_sys::Request) -> web_sys::Request {
        req
    }

    pub async fn debug_response(res: web_sys::Response) -> web_sys::Response {
        res
    }
}

#[cfg(debug_assertions)]
#[allow(dead_code)]
pub mod web_sys_debug {
    use shared::backend::result::ApiResult;
    use worker::{console_log, wasm_bindgen, wasm_bindgen::prelude::*};
    use worker::{js_sys::{self, try_iter}, wasm_bindgen_futures::JsFuture};
    use web_sys::js_sys::Uint8Array;

    // need to consume the request to get the body
    // so internally it's cloned and split, and the untouched clone is returned
    pub async fn debug_request(req: web_sys::Request) -> web_sys::Request {
        let (original_req, debug_req) = split_request(req);

        console_log!("request url: {}", &debug_req.url());
        console_log!("request method: {}", &debug_req.method());
        if let Ok(Some(values)) = try_iter(&debug_req.headers()) {
            for arr in values {
                if let Ok(arr) = arr.map(|arr| arr.unchecked_into::<js_sys::Array>()) { 
                    match (arr.get(0).as_string(), arr.get(1).as_string()) {
                        (Some(name), Some(value)) => {
                            console_log!("request header: {:?} = {:?}", name.as_str(), value.as_str());
                        },
                        _ => {
                            console_log!("non-string request header: {:?} = {:?}", arr.get(0), arr.get(1));
                        }
                    }
                }
            }
        }

        if let Some(body) = debug_req.body() {
            if let Ok(text) = stream_to_text(body).await {
                worker::console_log!("\nrequest body: {:?}", text);
            }
        }

        original_req
    }

    // need to consume the request to get the body
    // so internally it's cloned and split, and the untouched clone is returned
    pub async fn debug_response(res: web_sys::Response) -> web_sys::Response {
        let (original_res, debug_res) = split_response(res);

        console_log!("response status: {}", &debug_res.status());
        console_log!("response status text: {}", &debug_res.status_text());
        if let Ok(Some(values)) = try_iter(&debug_res.headers()) {
            for arr in values {
                if let Ok(arr) = arr.map(|arr| arr.unchecked_into::<js_sys::Array>()) { 
                    match (arr.get(0).as_string(), arr.get(1).as_string()) {
                        (Some(name), Some(value)) => {
                            console_log!("response header: {:?} = {:?}", name.as_str(), value.as_str());
                        },
                        _ => {
                            console_log!("non-string response header: {:?} = {:?}", arr.get(0), arr.get(1));
                        }
                    }
                }
            }
        }

        if let Some(body) = debug_res.body() {
            if let Ok(text) = stream_to_text(body).await {
                worker::console_log!("\nresponse body: {:?}", text);
            }
        }

        original_res
    }


    fn split_request(req: web_sys::Request) -> (web_sys::Request, web_sys::Request) {
        let mut init1 = web_sys::RequestInit::new();
        let mut init2 = web_sys::RequestInit::new();
        let url = req.url();
        init1.method(&req.method());
        init2.method(&req.method());
        init1.headers(&req.headers());
        init2.headers(&req.headers());
        if let Some(body) = req.body() {
            let bodies = body.tee();
            init1.body(Some(&bodies.get(0)));
            init2.body(Some(&bodies.get(1)));
        }

        let req1 = web_sys::Request::new_with_str_and_init(&url, &init1).unwrap();
        let req2 = web_sys::Request::new_with_str_and_init(&url, &init2).unwrap();

        (req1, req2)
    }

    fn split_response(res: web_sys::Response) -> (web_sys::Response, web_sys::Response) {
        let mut init1 = web_sys::ResponseInit::new();
        let mut init2 = web_sys::ResponseInit::new();
        init1.status(res.status());
        init2.status(res.status());
        init1.headers(&res.headers());
        init2.headers(&res.headers());
        if let Some(body) = res.body() {
            let bodies = body.tee();
            let res1 = web_sys::Response::new_with_opt_readable_stream_and_init(Some(&bodies.get(0).unchecked_into()), &init1).unwrap();
            let res2 = web_sys::Response::new_with_opt_readable_stream_and_init(Some(&bodies.get(1).unchecked_into()), &init2).unwrap();
            (res1, res2)
        } else {
            let res1 = web_sys::Response::new_with_opt_readable_stream_and_init(None, &init1).unwrap();
            let res2 = web_sys::Response::new_with_opt_readable_stream_and_init(None, &init2).unwrap();
            (res1, res2)
        }
    }

    async fn stream_to_text(stream: web_sys::ReadableStream) -> ApiResult<String> {
        let reader = stream.get_reader().unchecked_into::<Reader>();
        let value = JsFuture::from(reader.read()).await?.unchecked_into::<ReadContents>().value();
        let bytes:Vec<u8> = value.unchecked_into::<Uint8Array>().to_vec();
        let text = String::from_utf8(bytes).map_err(|err| err.to_string())?;

        Ok(text)
    }
    #[wasm_bindgen]
    extern "C" {
        type Reader;
        #[wasm_bindgen(method, js_class = "Reader")]
        fn read(this: &Reader) -> js_sys::Promise;
    }

    #[wasm_bindgen]
    extern "C" {
        type ReadContents;
        #[wasm_bindgen(method, getter, js_class = "ReadContents")]
        fn value(this: &ReadContents) -> JsValue;
    }
}