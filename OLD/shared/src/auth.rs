use crate::frontend::route::{Route as FrontendRoute, Dashboard};

pub const AUTH_TOKEN_ID_NAME: &str = "X-EXAMPLE-TOKEN-ID";
pub const AUTH_TOKEN_KEY_NAME: &str = "X-EXAMPLE-TOKEN-KEY";

pub const FRONTEND_ROUTE_AFTER_SIGNIN:FrontendRoute = FrontendRoute::Dashboard(Dashboard::Browse);