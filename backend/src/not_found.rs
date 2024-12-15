use http::StatusCode;

use crate::prelude::*;

pub struct NotFoundHandler {
    _ctx: ApiContext<HttpRequest>,
}

impl NotFoundHandler {
    pub fn new(_ctx: ApiContext<HttpRequest>) -> Self {
        Self { _ctx }
    }

    pub async fn handle(&mut self) -> ApiResponse {
        Ok(empty_response(Some(StatusCode::NOT_FOUND)))
    }
}
