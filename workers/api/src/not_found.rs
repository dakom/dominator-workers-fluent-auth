use crate::prelude::*;

pub struct NotFoundHandler {
    _ctx: ApiContext
}

impl NotFoundHandler {
    pub fn new(_ctx: ApiContext) -> Self {
        Self {
            _ctx
        }
    }

    pub async fn handle(&mut self) -> ApiResponse {
        Ok(Response::new_empty_status(404))
    }
}