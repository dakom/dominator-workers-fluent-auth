#[derive(Debug, Clone)]
pub enum Route {
    Landing(Landing),
    Dashboard(Dashboard),
    NotFound(NotFoundReason),
    TermsOfService,
    PrivacyPolicy,
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

        // if we need, we can get query params like:
        //let uid = url.search_params().get("uid");

        match paths {
            [""] => Self::Landing(Landing::Welcome),
            ["no-auth"] => Self::NotFound(NotFoundReason::NoAuth),
            ["auth", auth_path @ ..] => AuthRoute::try_from_paths(auth_path).map(|auth| Self::Landing(Landing::Auth(auth)))
                .unwrap_or(Self::NotFound(NotFoundReason::BadUrl)),
            ["dashboard", dashboard_path @ ..] => Dashboard::try_from_paths(dashboard_path).map(Self::Dashboard)
                .unwrap_or(Self::NotFound(NotFoundReason::BadUrl)),
            ["register"] => Self::Landing(Landing::Auth(AuthRoute::Register)),
            ["login"] => Self::Landing(Landing::Auth(AuthRoute::Signin)),
            ["terms-of-service"] => Self::TermsOfService,
            ["privacy-policy"] => Self::PrivacyPolicy,
            // these usually aren't visited directly, but can be helpful for debugging
            _ => Self::NotFound(NotFoundReason::BadUrl),
        }
    }

    pub fn link_url(&self, _domain: &str, root_path: &str) -> String {

        let s = format!("{}/{}", root_path, self.to_string());

        // let s = if root_path.is_empty() {
        //     format!("{}/{}", domain, self.to_string())
        // } else {
        //     format!("{}/{}/{}", domain, root_path, self.to_string())
        // };

        s.trim_end_matches(r#"//"#).to_string()
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
                Landing::Auth(auth_route) => {
                    format!("auth/{}", auth_route)
                }
            },
            Route::Dashboard(dashboard_route) => {
                format!("dashboard/{}", dashboard_route)
            },
            Route::NotFound(reason) => match reason {
                NotFoundReason::BadUrl => "404".to_string(),
                NotFoundReason::NoAuth => "no-auth".to_string(),
            },
            Route::TermsOfService => "terms-of-service".to_string(),
            Route::PrivacyPolicy => "privacy-policy".to_string(),
        };
        write!(f, "{}", s)
    }
}

impl AuthRoute {
    pub fn try_from_paths(paths: &[&str]) -> Option<Self> {
        match *paths {
            ["login"] => Some(Self::Signin),
            ["register"] => Some(Self::Register),
            ["verify-email-waiting"] => Some(Self::VerifyEmailWaiting),
            ["verify-email-confirm", oob_token_id, oob_token_key] => Some(Self::VerifyEmailConfirm {
                oob_token_id: oob_token_id.to_string(),
                oob_token_key: oob_token_key.to_string(),
            }),
            ["reset-password-confirm", oob_token_id, oob_token_key] => Some(Self::PasswordResetConfirm {
                oob_token_id: oob_token_id.to_string(),
                oob_token_key: oob_token_key.to_string(),
            }),
            ["openid-finalize", session_id, session_key] => Some(Self::OpenIdFinalize {
                session_id: session_id.to_string(),
                session_key: session_key.to_string(),
            }),
            _ => None,
        }
    }
}

impl std::fmt::Display for AuthRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            Self::Signin => "login".to_string(),
            Self::Register => "register".to_string(),
            Self::VerifyEmailWaiting => "verify-email-waiting".to_string(),
            Self::VerifyEmailConfirm { oob_token_id, oob_token_key} => format!("verify-email-confirm/{oob_token_id}/{oob_token_key}"),
            Self::PasswordResetConfirm{ oob_token_id, oob_token_key} => format!("reset-password-confirm/{oob_token_id}/{oob_token_key}"),
            Self::OpenIdFinalize{ session_id, session_key} => format!("openid-finalize/{session_id}/{session_key}"),
        };

        write!(f, "{}", s)
    }
}

impl std::fmt::Display for Dashboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            Dashboard::Profile => "profile".to_string(),
        };
        
        write!(f, "{}", s)
    }
}

impl Dashboard {
    pub fn try_from_paths(paths: &[&str]) -> Option<Self> {
        match *paths {
            ["profile"] => Some(Self::Profile),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Dashboard {
    Profile,
}

#[derive(Debug, Clone)]
pub enum Landing {
    Welcome,
    Auth(AuthRoute),
}

#[derive(Clone, Debug)]
pub enum AuthRoute {
    Register,
    Signin,
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
}
