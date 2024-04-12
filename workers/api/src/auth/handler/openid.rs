use std::{borrow::Cow, str::FromStr};

use openidconnect::{
    core::{CoreClient, CoreIdTokenClaims, CoreIdTokenVerifier, CoreProviderMetadata, CoreResponseType}, http::{self, HeaderMap, HeaderName, HeaderValue}, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, IssuerUrl, Nonce, OAuth2TokenResponse, RedirectUrl, Scope, TokenUrl
};
use shared::backend::route::{AuthRoute, OpenIdProvider, Route};
use worker::{js_sys::{self, try_iter}, wasm_bindgen_futures::JsFuture};
use web_sys::WorkerGlobalScope;
use crate::{auth::durable_objects::openid::OpenIdSessionNonce, config::{API_DOMAIN, API_ROOT_PATH}, prelude::*};

use super::super::durable_objects::openid::{OpenIdSession, OpenIdSessionDO};

pub struct OpenIdProcessor {
    pub provider: OpenIdProvider
}
impl OpenIdProcessor {
    pub fn new(provider: OpenIdProvider) -> Self {
        Self {
            provider
        }
    }

    // example: https://github.com/ramosbugs/openidconnect-rs/blob/main/examples/google.rs
    pub async fn get_auth_url(&self, env: &worker::Env) -> ApiResult<String> {
        let client_id = self.client_id(env)?;
        let client_secret = self.client_secret(env)?;
        let provider_metadata = self.provider_metadata().await?;

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            client_id,
            Some(client_secret),
        ).set_redirect_uri(RedirectUrl::new(Route::Auth(AuthRoute::OpenIdAccessTokenHook(self.provider)).link(API_DOMAIN, API_ROOT_PATH)).map_err(|err| err.to_string())?)
        .set_auth_type(openidconnect::AuthType::RequestBody);

        // have to create the session _before_ we can set the nonce (since we need the csrf_token first, which is synonymous with session (and state))
        let session = OpenIdSessionDO::create(&env, self.provider).await?;
        let object_id = session.id.clone();

        // Generate the authorization URL to which we'll redirect the user.
        // this will come back to the server and pick up the state from session durable object
        let (authorize_url, _, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            move || session.to_csrf_token(),
            Nonce::new_random,
        )
        .add_scope(Scope::new("email".to_string()))
        // this doesn't work in a test scenario, maybe requires publishing / sensitive verification?
        //.add_scope(Scope::new("profile".to_string()))
        .url();

        // store the nonce in the session durable object for later validation
        OpenIdSessionDO::set_nonce(&env, &object_id, nonce.secret().to_string()).await?;

        Ok(authorize_url.to_string())
    }

    // this came via the redirect from the openid provider
    pub async fn validate_token_claims(&self, ctx: &ApiContext, code: String, state: String) -> ApiResult<(OpenIdSession, CoreIdTokenClaims) > {
        let code = AuthorizationCode::new(code);
        let session = OpenIdSession::try_from_str(&state).map_err(|err| err.to_string())?;

        // pick up the state from the "state" parameter
        // validation gets us the originally set provider and nonce
        // the nonce will be used to validate the claims below
        let OpenIdSessionNonce {provider, nonce} = OpenIdSessionDO::get_nonce(&ctx.env, session.clone()).await?;

        // very unlikely to happen, but, simple sanity check that can help debugging
        if provider != self.provider {
            return Err("mismatched provider".into());
        }

        let client_id = self.client_id(&ctx.env)?;
        let client_secret = self.client_secret(&ctx.env)?;
        let provider_metadata = self.provider_metadata().await?;


        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            client_id,
            Some(client_secret),
        ).set_auth_type(openidconnect::AuthType::RequestBody);


        let redirect_uri = Route::Auth(AuthRoute::OpenIdAccessTokenHook(self.provider)).link(API_DOMAIN, API_ROOT_PATH);

        let token_response = client
            .exchange_code(code)
            .set_redirect_uri(Cow::Owned(RedirectUrl::new(redirect_uri).map_err(|err| err.to_string())?))
            .request_async(openid_http_client).await.map_err(|err| err.to_string())?;

        
        let id_token_verifier: CoreIdTokenVerifier = client.id_token_verifier();
        let claims: CoreIdTokenClaims = token_response
            .extra_fields()
            .id_token()
            .expect("Server did not return an ID token")
            .claims(&id_token_verifier, &nonce)
            .map_err(|err| err.to_string())?
            .clone();

        let email = claims.email().ok_or("no email in claims")?.to_string();
        // google supports the email_verified claim and we should use that
        // facebook doesn't support email_verified, and we default to false
        // see here for facebook's recommendation to verify ourselves: https://developers.facebook.com/docs/facebook-login/guides/advanced/existing-system#postfb1
        let email_verified = claims.email_verified().unwrap_or(false);
        let access_token = token_response.access_token().secret().to_string();

        // all is good, now we can set the access token so this session can be finalized
        let _ = OpenIdSessionDO::set_access_token(&ctx.env, &session.id, access_token, email, email_verified).await?;
        Ok((session, claims))
    }

    fn client_id(&self, env: &worker::Env) -> ApiResult<ClientId> {
        match self.provider {
            OpenIdProvider::Google => Ok(ClientId::new(get_secret(env, "OAUTH_GOOGLE_CLIENT_ID")?)),
            OpenIdProvider::Facebook => Ok(ClientId::new(get_secret(env, "OAUTH_FACEBOOK_CLIENT_ID")?))

        }
    }

    fn client_secret(&self, env: &worker::Env) -> ApiResult<ClientSecret> {
        match self.provider {
            OpenIdProvider::Google => Ok(ClientSecret::new(get_secret(env, "OAUTH_GOOGLE_CLIENT_SECRET")?)),
            OpenIdProvider::Facebook => Ok(ClientSecret::new(get_secret(env, "OAUTH_FACEBOOK_CLIENT_SECRET")?))
        }
    }

    fn issuer_url(&self) -> ApiResult<IssuerUrl> {
        match self.provider {
            OpenIdProvider::Google => IssuerUrl::new("https://accounts.google.com".to_string()).map_err(|err| err.to_string().into()),
            OpenIdProvider::Facebook => IssuerUrl::new("https://www.facebook.com".to_string()).map_err(|err| err.to_string().into())
        }
    }

    // see: https://github.com/ramosbugs/openidconnect-rs/issues/155#issuecomment-2044618322
    async fn provider_metadata(&self) -> ApiResult<CoreProviderMetadata> {
        let issuer_url = self.issuer_url()?;
        let mut provider_metadata = CoreProviderMetadata::discover_async(issuer_url, openid_http_client).await.map_err(|err| ApiError::from(err.to_string()))?;

        if provider_metadata.token_endpoint().is_none() {
            match self.provider {
                OpenIdProvider::Facebook => {
                    provider_metadata = provider_metadata.set_token_endpoint(Some(TokenUrl::new("https://graph.facebook.com/oauth/access_token".to_string()).map_err(|err| err.to_string())?));
                },
                OpenIdProvider::Google => {
                    provider_metadata = provider_metadata.set_token_endpoint(Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).map_err(|err| err.to_string())?));
                },
            }
        }

        if provider_metadata.token_endpoint_auth_methods_supported().is_none() {
            provider_metadata = provider_metadata.set_token_endpoint_auth_methods_supported(Some(vec![openidconnect::core::CoreClientAuthMethod::ClientSecretPost]));
        }

        Ok(provider_metadata)
    }
}

async fn openid_http_client(req: openidconnect::HttpRequest) -> ApiResult<openidconnect::HttpResponse> {
    fn map_request(req: openidconnect::HttpRequest) -> ApiResult<web_sys::Request> {
        let mut init:web_sys::RequestInit = web_sys::RequestInit::new();

        match req.method {
            http::Method::GET => {
                init.method("GET");
            },
            http::Method::POST => {
                init.method("POST");
            },
            http::Method::PUT => {
                init.method("PUT");
            },
            http::Method::DELETE => {
                init.method("DELETE");
            },
            http::Method::PATCH => {
                init.method("PATCH");
            },
            http::Method::HEAD => {
                init.method("HEAD");
            },
            http::Method::OPTIONS => {
                init.method("OPTIONS");
            },
            http::Method::CONNECT => {
                init.method("CONNECT");
            },
            http::Method::TRACE => {
                init.method("TRACE");
            },
            _ => {
                return Err("unsupported openid connect method".into());
            },
        }

        let headers = web_sys::Headers::new().unwrap();

        for header in req.headers.iter() {
            let header_value:&str = header.1.to_str().map_err(|err| err.to_string())?;
            headers.set(header.0.as_str(), header_value)?;
        }
        init.headers(&headers);

        if !req.body.is_empty() {
            init.body(Some(&js_sys::Uint8Array::from(req.body.as_slice())));
        }

        Ok(web_sys::Request::new_with_str_and_init(req.url.as_str(), &init)?)
    }

    async fn map_response(web_sys_res: web_sys::Response) -> ApiResult<openidconnect::HttpResponse> {

        let status_code = http::StatusCode::from_u16(web_sys_res.status()).map_err(|err| err.to_string())?;

        let mut headers = HeaderMap::new();


        if let Some(values) = try_iter(&web_sys_res.headers())? {
            for arr in values {
                if let Ok(arr) = arr.map(|arr| arr.unchecked_into::<js_sys::Array>()) { 
                    match (arr.get(0).as_string(), arr.get(1).as_string()) {
                        (Some(name), Some(value)) => {
                            let name = HeaderName::from_str(name.as_str()).map_err(|err| err.to_string())?;
                            let value = HeaderValue::from_str(value.as_str()).map_err(|err| err.to_string())?;
                            headers.insert(name, value);
                        },
                        _ => {
                            worker::console_warn!("invalid header in open id conversion: {:?} = {:?}", arr.get(0), arr.get(1));
                        }
                    }
                }
            }
        }

        let body = JsFuture::from(web_sys_res.text()?).await?.as_string().unwrap_or_default().into_bytes();

        Ok(openidconnect::HttpResponse {
            status_code,
            headers,
            body
        })
    }

    let web_sys_req = map_request(req)?;
    // worker::console_log!("\n");
    // let web_sys_req = web_sys_debug::debug_request(web_sys_req).await;
    let promise = js_sys::global().unchecked_into::<WorkerGlobalScope>().fetch_with_request(&web_sys_req);
    let web_sys_resp = JsFuture::from(promise).await?.unchecked_into::<web_sys::Response>();

    // worker::console_log!("\n");
    // let web_sys_resp = web_sys_debug::debug_response(web_sys_resp).await;
    map_response(web_sys_resp).await
}

