
#[derive(Debug, Clone)]
pub enum Route {
    Landing(Landing),
    Dashboard(Dashboard),
    NotFound(NotFoundReason),
}

impl Route {
    pub fn from_url(url: &str, root_path: &str) -> Self {
        let url = web_sys::Url::new(url).unwrap();
        let paths = url.pathname();
        let paths = paths
            .split('/')
            .into_iter()
            // skip all the roots (1 for the domain, 1 for each part of root path)
            .skip(root_path.chars().filter(|c| *c == '/').count() + 1)
            .collect::<Vec<_>>();
        let paths = paths.as_slice();

        match paths {
            [""] => Self::Landing(Landing::Welcome),
            ["register"] => Self::Landing(Landing::Auth(AuthRoute::Register)),
            ["signin"] => Self::Landing(Landing::Auth(AuthRoute::Signin)),
            ["reset-password-confirm", oob_token_id, oob_token_key] => {
                Self::Landing(Landing::Auth(AuthRoute::PasswordResetConfirm { oob_token_id: oob_token_id.to_string(), oob_token_key: oob_token_key.to_string()}))
            },
            ["no-auth"] => Self::NotFound(NotFoundReason::NoAuth),
            ["no-oob-code"] => Self::NotFound(NotFoundReason::NoOobCode),
            ["dashboard", "profile", _section] => {
                Self::NotFound(NotFoundReason::BadUrl)
            },
            ["dashboard"] => {
                Self::Dashboard(Dashboard::Browse)
            },
            ["dashboard", "browse"] => {
                Self::Dashboard(Dashboard::Browse)
            },
            ["verify-email-waiting"] => Self::Landing(Landing::Auth(AuthRoute::VerifyEmailWaiting)),
            ["verify-email-confirm", oob_token_id, oob_token_key] => {
                Self::Landing(Landing::Auth(AuthRoute::VerifyEmailConfirm { oob_token_id: oob_token_id.to_string(), oob_token_key: oob_token_key.to_string()}))
            },
            ["openid-finalize", session_id, session_key] => {
                Self::Landing(Landing::Auth(AuthRoute::OpenIdFinalize { session_id: session_id.to_string(), session_key: session_key.to_string()}))
            },
            // these usually aren't visited directly, but can be helpful for debugging
            _ => Self::NotFound(NotFoundReason::BadUrl),
        }
    }

    pub fn link(&self, domain: &str, root_path: &str) -> String {
        if root_path.is_empty() {
            format!("{}/{}", domain, self.to_string())
        } else {
            format!("{}/{}/{}", domain, root_path, self.to_string())
        }
    }

    // unlike backend auth, this is just a pure yes/no gate for frontend
    // so that it can redirect on unauthenticated pages
    pub fn requires_auth(&self) -> bool {
        match self {
            Self::Dashboard(_) => true,
            _ => false,
        }
    }
}


impl std::fmt::Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            Route::Landing(landing) => match landing {
                Landing::Welcome => "".to_string(),
                Landing::Auth(auth_page) => match auth_page {
                    AuthRoute::Signin => "signin".to_string(),
                    AuthRoute::Register => "register".to_string(),
                    AuthRoute::VerifyEmailWaiting => "verify-email-waiting".to_string(),
                    AuthRoute::VerifyEmailConfirm { oob_token_id, oob_token_key} => format!("verify-email-confirm/{oob_token_id}/{oob_token_key}"),
                    AuthRoute::PasswordResetConfirm{ oob_token_id, oob_token_key} => format!("reset-password-confirm/{oob_token_id}/{oob_token_key}"),
                    AuthRoute::OpenIdFinalize{ session_id, session_key} => format!("openid-finalize/{session_id}/{session_key}"),
                },
            }
            Route::Dashboard(dashboard) => {
                match dashboard {
                    Dashboard::Browse => format!("dashboard/browse"),
                }
            },
            Route::NotFound(reason) => match reason {
                NotFoundReason::BadUrl => "404".to_string(), 
                NotFoundReason::NoAuth => "no-auth".to_string(), 
                NotFoundReason::NoOobCode => "no-oob-code".to_string(), 
            }
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Dashboard {
    Browse,
}

#[derive(Debug, Clone)]
pub enum Landing {
    Welcome,
    Auth(AuthRoute)
}

#[derive(Clone, Debug)]
pub enum AuthRoute {
    Signin,
    Register,
    VerifyEmailWaiting,
    VerifyEmailConfirm {
        oob_token_id: String,
        oob_token_key: String
    },
    PasswordResetConfirm {
        oob_token_id: String,
        oob_token_key: String
    },
    OpenIdFinalize{
        session_id: String,
        session_key: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotFoundReason {
    NoAuth,
    BadUrl,
    NoOobCode
}