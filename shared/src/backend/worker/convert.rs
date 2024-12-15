use crate::backend::result::ApiError;

impl From<worker::Error> for ApiError {
    fn from(err: worker::Error) -> Self {
        Self::Unknown(err.to_string())
    }
}
