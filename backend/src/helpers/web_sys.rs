use bytes::BytesMut;
use futures::stream::StreamExt;
use http::StatusCode;
use js_sys::{Object, Reflect, Uint8Array};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use shared::backend::result::{ApiError, ApiResult};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_streams::ReadableStream;
use web_sys::ReadableStreamDefaultController;
use worker::{Body, HttpResponse};

pub async fn stream_to_text(web_sys_stream: web_sys::ReadableStream) -> ApiResult<String> {
    let stream = ReadableStream::from_raw(web_sys_stream);
    let mut stream = stream.into_stream();

    let mut bytes = BytesMut::new();
    while let Some(Ok(chunk)) = stream.next().await {
        if chunk.is_undefined() || chunk.is_null() {
            continue;
        }
        let chunk: Vec<u8> = chunk.unchecked_into::<Uint8Array>().to_vec();
        bytes.extend_from_slice(&chunk);
    }

    if bytes.is_empty() {
        return Err(ApiError::MissingBody("empty body".to_string()));
    }

    let text = String::from_utf8(bytes.to_vec()).map_err(|err| err.to_string())?;

    Ok(text)
}

pub async fn body_to_text(body: Body) -> ApiResult<String> {
    let stream = body.into_inner();

    let stream: web_sys::ReadableStream =
        stream.ok_or(ApiError::MissingBody("none at all".to_string()))?;
    stream_to_text(stream)
        .await
        .map_err(|err| ApiError::MissingBody(err.to_string()))
}

pub async fn json_body_to_any<T: DeserializeOwned>(body: Body) -> ApiResult<T> {
    let text = body_to_text(body).await?;

    let data = serde_json::from_str(&text).map_err(|err| {
        ApiError::ParseBody(format!(
            "failed to parse json. error is {}. body is {}",
            err, text
        ))
    })?;

    Ok(data)
}

pub async fn any_to_json_body(data: &impl Serialize) -> ApiResult<Body> {
    struct StreamWithClosure {
        pub stream: web_sys::ReadableStream,
        _start_closure: Closure<dyn FnMut(JsValue)>,
    }

    impl StreamWithClosure {
        pub fn new(my_struct: &impl Serialize) -> Result<StreamWithClosure, JsValue> {
            // Serialize the struct to JSON bytes
            let json_bytes =
                serde_json::to_vec(my_struct).map_err(|e| JsValue::from_str(&e.to_string()))?;

            // Create a Uint8Array from the bytes
            let array = Uint8Array::from(&json_bytes[..]);

            // Create the underlying source object
            let underlying_source = Object::new();

            // Create a closure for the start method
            let start_closure = Closure::wrap(Box::new(move |controller: JsValue| {
                let controller = controller.unchecked_into::<ReadableStreamDefaultController>();
                // Enqueue the data
                controller.enqueue_with_chunk(&array).unwrap();
                // Close the stream
                controller.close().unwrap();
            }) as Box<dyn FnMut(JsValue)>);

            // Set the "start" property on the underlying source
            Reflect::set(
                &underlying_source,
                &JsValue::from_str("start"),
                start_closure.as_ref().unchecked_ref(),
            )?;

            // Create the ReadableStream with the underlying source
            let stream = web_sys::ReadableStream::new_with_underlying_source(&underlying_source)?;

            Ok(StreamWithClosure {
                stream,
                _start_closure: start_closure, // Keep the closure alive
            })
        }
    }

    let stream_with_closure = StreamWithClosure::new(data)?;

    Ok(Body::new(stream_with_closure.stream))
}

pub async fn any_to_json_response(
    res: &impl Serialize,
    status_code: Option<StatusCode>,
) -> HttpResponse {
    let body = any_to_json_body(res).await.unwrap();

    let mut res = HttpResponse::new(body);
    res.headers_mut()
        .insert("Content-Type", "application/json".parse().unwrap());
    if let Some(status_code) = status_code {
        *res.status_mut() = status_code;
    }
    res
}

pub fn empty_response(status_code: Option<StatusCode>) -> HttpResponse {
    let mut res = HttpResponse::new(Body::empty());
    if let Some(status_code) = status_code {
        *res.status_mut() = status_code;
    }
    res
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

#[cfg(not(debug_assertions))]
#[allow(dead_code)]
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
    use web_sys::js_sys::Uint8Array;
    use worker::{console_log, wasm_bindgen, wasm_bindgen::prelude::*};
    use worker::{
        js_sys::{self, try_iter},
        wasm_bindgen_futures::JsFuture,
    };

    use crate::stream_to_text;

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
                            console_log!(
                                "request header: {:?} = {:?}",
                                name.as_str(),
                                value.as_str()
                            );
                        }
                        _ => {
                            console_log!(
                                "non-string request header: {:?} = {:?}",
                                arr.get(0),
                                arr.get(1)
                            );
                        }
                    }
                }
            }
        }

        if let Some(body) = debug_req.body() {
            if let Ok(text) = stream_to_text(body).await {
                tracing::info!("\nrequest body: {:?}", text);
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
                            console_log!(
                                "response header: {:?} = {:?}",
                                name.as_str(),
                                value.as_str()
                            );
                        }
                        _ => {
                            console_log!(
                                "non-string response header: {:?} = {:?}",
                                arr.get(0),
                                arr.get(1)
                            );
                        }
                    }
                }
            }
        }

        if let Some(body) = debug_res.body() {
            if let Ok(text) = stream_to_text(body).await {
                tracing::info!("\nresponse body: {:?}", text);
            }
        }

        original_res
    }

    fn split_request(req: web_sys::Request) -> (web_sys::Request, web_sys::Request) {
        let init1 = web_sys::RequestInit::new();
        let init2 = web_sys::RequestInit::new();
        let url = req.url();
        init1.set_method(&req.method());
        init2.set_method(&req.method());
        init1.set_headers(&req.headers());
        init2.set_headers(&req.headers());
        if let Some(body) = req.body() {
            let bodies = body.tee();
            init1.set_body(&bodies.get(0));
            init2.set_body(&bodies.get(1));
        }

        let req1 = web_sys::Request::new_with_str_and_init(&url, &init1).unwrap();
        let req2 = web_sys::Request::new_with_str_and_init(&url, &init2).unwrap();

        (req1, req2)
    }

    fn split_response(res: web_sys::Response) -> (web_sys::Response, web_sys::Response) {
        let init1 = web_sys::ResponseInit::new();
        let init2 = web_sys::ResponseInit::new();
        init1.set_status(res.status());
        init2.set_status(res.status());
        init1.set_headers(&res.headers());
        init2.set_headers(&res.headers());
        if let Some(body) = res.body() {
            let bodies = body.tee();
            let res1 = web_sys::Response::new_with_opt_readable_stream_and_init(
                Some(&bodies.get(0).unchecked_into()),
                &init1,
            )
            .unwrap();
            let res2 = web_sys::Response::new_with_opt_readable_stream_and_init(
                Some(&bodies.get(1).unchecked_into()),
                &init2,
            )
            .unwrap();
            (res1, res2)
        } else {
            let res1 =
                web_sys::Response::new_with_opt_readable_stream_and_init(None, &init1).unwrap();
            let res2 =
                web_sys::Response::new_with_opt_readable_stream_and_init(None, &init2).unwrap();
            (res1, res2)
        }
    }
}
