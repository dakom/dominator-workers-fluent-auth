use openidconnect::{CsrfToken, Nonce};
use serde::{Deserialize, Serialize};
use shared::backend::route::OpenIdProvider;

use crate::{config::{AUTH_OPEN_ID_SESSION_EXPIRES, AUTH_TOKEN_KEY_LENGTH}, prelude::durable_object::*};

#[durable_object]
pub struct OpenIdSessionDO {
    state: State,
    _env: Env
}


#[durable_object]
impl DurableObject for OpenIdSessionDO {
    fn new(state: State, env: Env) -> Self {
        Self {
            state: state,
            _env: env,
        }
    }

    async fn fetch(&mut self, req: Request) -> worker::Result<Response> {
        let action = req.headers().get("action")?.and_then(|x| OpenIdSessionAction::from_string(x).ok()).ok_or("missing action header")?;

        match action {
            OpenIdSessionAction::Create{provider} => {
                let mut headers = Headers::new();
                let key = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&rand::thread_rng().gen::<[u8; AUTH_TOKEN_KEY_LENGTH]>());
                let provider = provider.as_str();

                headers.set("key", &key).unwrap();
                headers.set("provider", provider).unwrap();

                self.state.storage().put("key", key).await?;
                self.state.storage().put("provider", provider).await?;
                self.state.storage().set_alarm(Duration::from_millis(AUTH_OPEN_ID_SESSION_EXPIRES)).await?;

                Ok(Response::empty()?.with_headers(headers))
            }

            OpenIdSessionAction::SetNonce { nonce } => {
                self.state.storage().put("nonce", nonce).await?;
                Response::empty()
            }

            OpenIdSessionAction::GetTokenExchange {key} => {
                if self.storage_get::<String>("key").await? != key {
                    return Err("invalid key".into());
                }
                let provider = OpenIdProvider::try_from_str(&self.storage_get::<String>("provider").await?).ok_or("invalid provider str")?;
                let nonce = Nonce::new(self.storage_get::<String>("nonce").await?);

                let mut headers = Headers::new();

                headers.set("result", &serde_json::to_string(&OpenIdSessionNonce {
                    provider,
                    nonce
                }).unwrap()).unwrap();

                Ok(Response::empty()?.with_headers(headers))
            }

            OpenIdSessionAction::SetAccessToken { access_token, email, email_verified} => {
                self.state.storage().put("access_token", access_token).await?;
                self.state.storage().put("email", email).await?;
                self.state.storage().put("email_verified", email_verified).await?;

                Response::empty()
            }

            OpenIdSessionAction::FinalizeExec{key} => {
                let finalize_info = self.load_finalize_info(&key).await.map_err(|err| err.to_string())?;

                self.state.storage().delete_alarm().await?;
                self.state.storage().delete_all().await?;

                let mut headers = Headers::new();
                headers.set("result", &serde_json::to_string(&finalize_info).unwrap()).unwrap();

                Ok(Response::empty()?.with_headers(headers))
            }
            OpenIdSessionAction::FinalizeQuery{key} => {
                let finalize_info = self.load_finalize_info(&key).await.map_err(|err| err.to_string())?;

                let mut headers = Headers::new();
                headers.set("result", &serde_json::to_string(&finalize_info).unwrap()).unwrap();

                Ok(Response::empty()?.with_headers(headers))
            }
        }
    }

    async fn alarm(&mut self) -> worker::Result<Response> {
        self.state.storage().delete_all().await?;
        Response::empty()
    }
}

impl DurableObjectExt for OpenIdSessionDO {
    fn state(&self) -> &State {
        &self.state
    }
}

impl OpenIdSessionDO {
    #[cfg(debug_assertions)]
    const NAMESPACE: &'static str = "AUTH_OPENID_SESSION_DEV";
    #[cfg(not(debug_assertions))]
    const NAMESPACE: &'static str = "AUTH_OPENID_SESSION_PROD";

    fn stub(env: &Env, id: &str) -> ApiResult<Stub> {
        env.durable_object(Self::NAMESPACE)?.id_from_string(id)?.get_stub().map_err(|err| err.into())
    }

    pub async fn create(env: &Env, provider: OpenIdProvider) -> ApiResult<OpenIdSession> {
        let mut do_headers = Headers::new();
        do_headers.append("action", &OpenIdSessionAction::Create{provider}.to_string()?)?;

        let req = Request::new_with_init("http://internal", RequestInit::new().with_headers(do_headers))?;
        let id = env.durable_object(Self::NAMESPACE)?.unique_id()?.to_string();
        let res = Self::stub(env, &id)?.fetch_with_request(req).await?;
        let key = res.headers().get("key")?.ok_or("missing key header")?;

        Ok(OpenIdSession{ id, key })
    }

    pub async fn set_nonce(env: &Env, id: &str, nonce: String) -> ApiResult<()> {
        let mut do_headers = Headers::new();
        do_headers.append("action", &OpenIdSessionAction::SetNonce { nonce}.to_string()?)?;

        let req = Request::new_with_init("http://internal", RequestInit::new().with_headers(do_headers))?;
        let _ = Self::stub(env, &id)?.fetch_with_request(req).await?;

        Ok(())
    }

    pub async fn get_nonce(env: &Env, state: OpenIdSession) -> ApiResult<OpenIdSessionNonce> {
        let mut do_headers = Headers::new();
        do_headers.append("action", &OpenIdSessionAction::GetTokenExchange { key: state.key}.to_string()?)?;

        let req = Request::new_with_init("http://internal", RequestInit::new().with_headers(do_headers))?;
        let res = Self::stub(env, &state.id)?.fetch_with_request(req).await?;
        res
            .headers()
            .get("result")?
            .ok_or("missing verification header".into())
            .and_then(|x| serde_json::from_str(&x).map_err(|err| err.to_string().into()))
    }

    pub async fn set_access_token(env: &Env, id: &str, access_token: String, email: String, email_verified: bool) -> ApiResult<()> {
        let mut do_headers = Headers::new();
        do_headers.append("action", &OpenIdSessionAction::SetAccessToken {access_token, email, email_verified}.to_string()?)?;

        let req = Request::new_with_init("http://internal", RequestInit::new().with_headers(do_headers))?;
        let _ = Self::stub(env, &id)?.fetch_with_request(req).await?;

        Ok(())
    }

    pub async fn finalize_exec(env: &Env, session: OpenIdSession) -> ApiResult<OpenIdSessionFinalizeInfo> {
        let mut do_headers = Headers::new();
        do_headers.append("action", &OpenIdSessionAction::FinalizeExec { key: session.key}.to_string()?)?;

        let req = Request::new_with_init("http://internal", RequestInit::new().with_headers(do_headers))?;
        let res = Self::stub(env, &session.id)?.fetch_with_request(req).await?;
        res
            .headers()
            .get("result")?
            .ok_or("missing finalization header".into())
            .and_then(|x| serde_json::from_str(&x).map_err(|err| err.to_string().into()))
    }
    pub async fn finalize_query(env: &Env, session: OpenIdSession) -> ApiResult<OpenIdSessionFinalizeInfo> {
        let mut do_headers = Headers::new();
        do_headers.append("action", &OpenIdSessionAction::FinalizeQuery { key: session.key}.to_string()?)?;

        let req = Request::new_with_init("http://internal", RequestInit::new().with_headers(do_headers))?;
        let res = Self::stub(env, &session.id)?.fetch_with_request(req).await?;
        res
            .headers()
            .get("result")?
            .ok_or("missing finalization header".into())
            .and_then(|x| serde_json::from_str(&x).map_err(|err| err.to_string().into()))
    }

    // helpers called from an instantiated instance
    pub async fn load_finalize_info(&self, key: &str) -> ApiResult<OpenIdSessionFinalizeInfo> {
        if self.storage_get::<String>("key").await? != key {
            return Err("invalid key".into());
        }
        let provider = OpenIdProvider::try_from_str(&self.storage_get::<String>("provider").await?).ok_or("invalid provider str")?;
        let access_token = self.storage_get::<String>("access_token").await?;
        let email = self.storage_get::<String>("email").await?;
        let email_verified = self.storage_get::<bool>("email_verified").await?;

        Ok(OpenIdSessionFinalizeInfo {
            provider,
            access_token,
            email,
            email_verified
        })
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub enum OpenIdSessionAction {
    Create {
        provider: OpenIdProvider
    },
    SetNonce {
        nonce: String
    },
    GetTokenExchange {
        key: String
    },
    SetAccessToken {
        access_token: String,
        email: String,
        email_verified: bool,
    },
    FinalizeExec {
        key: String
    },
    FinalizeQuery {
        key: String
    }
}

impl OpenIdSessionAction {
    fn to_string(&self) -> ApiResult<String> {
        serde_json::to_string(self).map_err(|err| err.to_string().into())
    }
    fn from_string(action: String) -> ApiResult<Self> {
        serde_json::from_str(action.as_str()).map_err(|err| err.to_string().into())
    }
}


#[derive(Debug, Clone)]
pub struct OpenIdSession {
    pub id: String,
    pub key: String,
}

impl OpenIdSession {
    // a character that is neither url encoded nor used in the base64 url-safe set
    // see https://datatracker.ietf.org/doc/html/rfc4648#section-5
    const DELIMITER: char = '.';

    pub fn to_csrf_token(&self) -> CsrfToken {
        let s = format!("{}{}{}", self.id, Self::DELIMITER, self.key);
        CsrfToken::new(s)
    }

    pub fn try_from_str(state: &str) -> ApiResult<Self> {
        let mut parts = state.split(Self::DELIMITER);
        Ok(Self {
            id: parts.next().ok_or("missing id")?.to_string(),
            key: parts.next().ok_or("missing key")?.to_string(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenIdSessionNonce {
    pub provider: OpenIdProvider,
    pub nonce: Nonce,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenIdSessionFinalizeInfo {
    pub provider: OpenIdProvider,
    pub access_token: String,
    pub email: String,
    pub email_verified: bool,
}