use crate::frontend::route::{Dashboard, Route as FrontendRoute};

pub const HEADER_AUTH_TOKEN_ID: &str = "X-DEMO-TOKEN-ID";
pub const HEADER_AUTH_TOKEN_KEY: &str = "X-DEMO-TOKEN-KEY";

pub const FRONTEND_ROUTE_AFTER_LOGIN: FrontendRoute = FrontendRoute::Dashboard(Dashboard::Profile);
