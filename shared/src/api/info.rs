use serde::{Deserialize, Serialize};

use crate::backend::route::Route;

use super::{ApiRes, Method};

pub struct ServerInfo {}

impl ApiRes for ServerInfo {
    const ROUTE: Route = Route::Info;
    const METHOD: Method = Method::GET;

    type Res = ServerInfoResponse;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ServerInfoResponse {
    pub version: String,
}
