
use serde::{Deserialize, Serialize};

use crate::{config::AUTH_TOKEN_KEY_LENGTH, prelude::durable_object::*};

#[durable_object]
pub struct AuthTokenDO {
    state: State,
    _env: Env
}


#[durable_object]
impl DurableObject for AuthTokenDO {
    fn new(state: State, env: Env) -> Self {
        Self {
            state: state,
            _env: env,
        }
    }

    async fn fetch(&mut self, req: Request) -> worker::Result<Response> {
        let action = req.headers().get("action")?.and_then(|x| AuthTokenAction::from_string(x).ok()).ok_or("missing action header")?;

        match action {
            AuthTokenAction::Create {user_token, uid, expires_ms, kind} => {
                let key = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&rand::thread_rng().gen::<[u8; AUTH_TOKEN_KEY_LENGTH]>());

                let mut headers = Headers::new();
                headers.set("key", &key).unwrap();

                AuthTokenStorage{
                    user_token,
                    uid,
                    key,
                    kind
                }.save(&mut self.state.storage()).await?;
                self.state.storage().set_alarm(Duration::from_millis(expires_ms)).await?;

                Ok(Response::empty()?.with_headers(headers))
            }
            AuthTokenAction::Validate { key, after, kind } => {
                let stored = AuthTokenStorage::load(&self.state.storage()).await?;

                if kind != stored.kind {
                    return Err("invalid kind".into());
                }

                if key != stored.key {
                    return Err("invalid key".into());
                }


                match after {
                    AuthTokenAfterValidation::Delete => {
                        self.state.storage().delete_alarm().await?;
                        self.state.storage().delete_all().await?;
                    },
                    AuthTokenAfterValidation::ExtendExpiresMs(expires_ms) => {
                        self.state.storage().set_alarm(Duration::from_millis(expires_ms)).await?;
                    }
                }

                let mut headers = Headers::new();
                headers.set("uid", &stored.uid.to_string()).unwrap();
                headers.set("user-token", &stored.user_token).unwrap();
                Ok(Response::empty()?.with_headers(headers))
            }
            AuthTokenAction::Destroy => {
                self.state.storage().delete_alarm().await?;
                self.state.storage().delete_all().await?;
                Response::empty()
            }
        }
    }

    async fn alarm(&mut self) -> worker::Result<Response> {
        self.state.storage().delete_all().await?;
        Response::empty()
    }
}

impl DurableObjectExt for AuthTokenDO {
    fn state(&self) -> &State {
        &self.state
    }
}

impl AuthTokenDO {
    #[cfg(debug_assertions)]
    const NAMESPACE: &'static str = "AUTH_TOKEN_DEV";
    #[cfg(not(debug_assertions))]
    const NAMESPACE: &'static str = "AUTH_TOKEN_PROD";

    fn stub(env: &Env, id: &str) -> ApiResult<Stub> {
        env.durable_object(Self::NAMESPACE)?.id_from_string(id)?.get_stub().map_err(|err| err.into())
    }

    pub async fn create(env: &Env, kind: AuthTokenKind, uid: UserId, user_token: String, expires_ms: u64) -> ApiResult<AuthTokenCreateResponse> {
        let mut do_headers = Headers::new();
        do_headers.append("action", &AuthTokenAction::Create{
            kind,
            uid,
            user_token,
            expires_ms
        }.to_string()?)?;

        let req = Request::new_with_init("http://internal", RequestInit::new().with_headers(do_headers))?;
        let id = env.durable_object(Self::NAMESPACE)?.unique_id()?.to_string();
        // the stub is from id, not uid
        let res = Self::stub(env, &id)?.fetch_with_request(req).await?;
        let key = res.headers().get("key")?.ok_or("missing key header")?;

        Ok(AuthTokenCreateResponse { id, key })
    }

    pub async fn validate(env: &Env, kind: AuthTokenKind, id: &str, key: String, after: AuthTokenAfterValidation) -> ApiResult<AuthTokenValidateResponse> {
        let mut do_headers = Headers::new();
        do_headers.append("action", &AuthTokenAction::Validate { key, after, kind }.to_string()?)?;

        let req = Request::new_with_init("http://internal", RequestInit::new().with_headers(do_headers))?;
        let res = Self::stub(env, id)?.fetch_with_request(req).await?;
        let uid:UserId = res.headers().get("uid")?.ok_or("missing uid header")?.try_into()?;
        let user_token = res.headers().get("user-token")?.ok_or("missing user-token header")?;

        Ok(AuthTokenValidateResponse { uid, user_token })
    }

    pub async fn destroy(env: &Env, id: &str) -> ApiResult<()> {
        let mut do_headers = Headers::new();
        do_headers.append("action", &AuthTokenAction::Destroy.to_string()?)?;

        let req = Request::new_with_init("http://internal", RequestInit::new().with_headers(do_headers))?;
        Self::stub(env, &id)?.fetch_with_request(req).await?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum AuthTokenAction {
    Create {
        user_token: String,
        uid: UserId,
        expires_ms: u64,
        kind: AuthTokenKind,
    },
    Validate {
        key: String,
        kind: AuthTokenKind,
        after: AuthTokenAfterValidation 
    },
    Destroy,
}
impl AuthTokenAction {
    fn to_string(&self) -> ApiResult<String> {
        serde_json::to_string(self).map_err(|err| err.to_string().into())
    }
    fn from_string(action: String) -> ApiResult<Self> {
        serde_json::from_str(action.as_str()).map_err(|err| err.to_string().into())
    }
}

#[derive(Serialize, Debug)]
struct AuthTokenStorage {
    user_token: String,
    uid: UserId,
    key: String,
    kind: AuthTokenKind
}

impl AuthTokenStorage {
    const KEYS: [&'static str; 4] = ["user_token", "uid", "key", "kind"];

    async fn save(&self, storage: &mut Storage) -> worker::Result<()> {
        storage.put_multiple(self).await.map_err(|err| err.into())
    }

    async fn load(storage: &Storage) -> worker::Result<Self> {
        let map = storage.get_multiple(Self::KEYS.to_vec()).await?;

        let user_token = map.get(&JsValue::from_str("user_token")).as_string().ok_or("missing user-token")?;
        let key = map.get(&JsValue::from_str("key")).as_string().ok_or("missing key")?;
        let uid:UserId = map.get(&JsValue::from_str("uid")).as_string().ok_or("missing uid")?.try_into()?;
        let kind:AuthTokenKind = map.get(&JsValue::from_str("kind")).as_string().ok_or("missing kind")?.try_into()?;

        Ok(Self {
            user_token,
            uid,
            key,
            kind,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthTokenCreateResponse {
    pub id: String,
    pub key: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthTokenValidateResponse {
    pub uid: UserId,
    pub user_token: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub enum AuthTokenAfterValidation {
    Delete,
    ExtendExpiresMs(u64)
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AuthTokenKind {
    Signin,
    PasswordReset,
    VerifyEmail
}

impl TryFrom<String> for AuthTokenKind {
    type Error = &'static str;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "signin" => Ok(Self::Signin),
            "passwordreset" => Ok(Self::PasswordReset),
            "verifyemail" => Ok(Self::VerifyEmail),
            _ => Err("invalid kind")
        }
    }
}
