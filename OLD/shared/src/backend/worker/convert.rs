use worker::worker_sys::web_sys::Response;
use crate::backend::result::ApiError;

use super::ext::ResponseExt;

impl From<worker::Error> for ApiError {
    fn from(err: worker::Error) -> Self {
        Self::Unknown(err.to_string())
    }
}

impl From<&ApiError> for Response {
    fn from(err: &ApiError) -> Self {
        let status_code = match err {
            // just a nice helper to debug things
            // it's up to the frontend to decide what to do with this
            ApiError::Auth(_) => 401,
            ApiError::Unknown(_) => 500
        };

        Response::new_json_status(err, status_code)
    }
}
impl From<ApiError> for Response {
    fn from(err: ApiError) -> Self {
        (&err).into()
    }
}