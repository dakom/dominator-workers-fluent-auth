/// A given api endpoint must implement exactly ONE of these traits here in the shared crate
/// And in the backend, it must implement the corresponding extension trait.
/// (in the frontend, the extension trait is automatically implemented since there's no distinct logic to fetch/request/response) 
/// 
/// With those two places implemented (here and in backend), the API is fully defined
/// it's guaranteed that the request, response, method, and auth checks (via route.auth_kind())
/// are all in sync, across frontend and backend and generated documentation
/// and that any changes are caught at compile time
pub mod auth;

use serde::{de::DeserializeOwned, Serialize};

use crate::backend::route::Route;

/// has a request type and a response type
pub trait ApiBoth {
    /// The backend route for this endpoint.
    const ROUTE: Route;

    /// The request type for this endpoint.
    type Req: DeserializeOwned + Serialize + 'static;

    /// The response type for this endpoint.
    type Res: DeserializeOwned + Serialize + 'static;

    /// The method used to make a request to the endpoint.
    const METHOD: Method;
}

/// has only a request type, no response type
pub trait ApiReq {
    /// The backend route for this endpoint.
    const ROUTE: Route;

    /// The request type for this endpoint.
    type Req: DeserializeOwned + Serialize + 'static;

    /// The method used to make a request to the endpoint.
    const METHOD: Method;
}

/// has only a response type, no request type
pub trait ApiRes {
    /// The backend route for this endpoint.
    const ROUTE: Route;

    /// The response type for this endpoint.
    type Res: DeserializeOwned + Serialize + 'static;

    /// The method used to make a request to the endpoint.
    const METHOD: Method;
}

/// has neither a request type nor a response type 
pub trait ApiEmpty {
    /// The backend route for this endpoint.
    const ROUTE: Route;

    /// The method used to make a request to the endpoint.
    const METHOD: Method;
}

/// has only a response type, no request type, dynamic route
pub trait ApiResDynRoute {
    /// The backend route for this endpoint.
    fn route(&self) -> Route;

    /// The response type for this endpoint.
    type Res: DeserializeOwned + Serialize + 'static;

    /// The method used to make a request to the endpoint.
    const METHOD: Method;
}

/// no response or request type, dynamic route
pub trait ApiEmptyDynRoute {
    /// The backend route for this endpoint.
    fn route(&self) -> Route;

    /// The method used to make a request to the endpoint.
    const METHOD: Method;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
        }
    }
}