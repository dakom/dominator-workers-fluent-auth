use crate::frontend::route::{Dashboard, Route as FrontendRoute};

pub const HEADER_AUTH_TOKEN_ID: &str = "X-DEMO-TOKEN-ID";
pub const HEADER_AUTH_TOKEN_KEY: &str = "X-DEMO-TOKEN-KEY";
// for admin-only impersonation
pub const HEADER_ADMIN_CODE: &str = "X-DEMO-ADMIN-CODE";
pub const HEADER_ADMIN_AUTH_UID: &str = "X-DEMO-ADMIN-UID";

pub const FRONTEND_ROUTE_AFTER_LOGIN: FrontendRoute = FrontendRoute::Dashboard(Dashboard::Profile);
