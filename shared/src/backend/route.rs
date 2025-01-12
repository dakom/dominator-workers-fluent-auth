use crate::api::auth::OpenIdProvider;

#[derive(Debug, Clone)]
pub enum Route {
    Info,
    Auth(AuthRoute),
    Admin(AdminRoute),
}

#[derive(Debug, Clone)]
pub enum AuthRoute {
    RegisterEmail,
    SendVerifyEmail,
    ConfirmEmailValidation,
    SendPasswordResetAny,
    SendPasswordResetMe,
    ConfirmPasswordReset,
    SigninEmail,
    Check,
    Signout,
    OpenIdConnect,
    OpenIdAccessTokenHook(OpenIdProvider),
    OpenIdFinalizeExec,
    OpenIdFinalizeQuery,
}

#[derive(Debug, Clone)]
pub enum AdminRoute {
    Placeholder
}

impl Route {
    pub fn try_from_url(url: &str, root_path: &str) -> Option<Self> {
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
            ["auth", auth_path @ ..] => AuthRoute::try_from_paths(auth_path).map(Self::Auth),
            ["admin", admin_path @ ..] => AdminRoute::try_from_paths(admin_path).map(Self::Admin),
            ["info"] => Some(Self::Info),
            _ => None,
        }
    }

    // in http://example.com/foo/bar/baz
    // domain = http://example.com
    // root_path = foo
    // the route itself would map to bar/baz
    pub fn link(&self, domain: &str, root_path: &str) -> String {
        if root_path.is_empty() {
            format!("{}/{}", domain, self.to_string())
        } else {
            format!("{}/{}/{}", domain, root_path, self.to_string())
        }
    }

    pub fn auth_kind(&self) -> RouteAuthKind {
        match self {
            Route::Auth(auth_route) => match auth_route {
                AuthRoute::Check => RouteAuthKind::Full,
                AuthRoute::SendPasswordResetMe => RouteAuthKind::Full,
                // these just need to set the cookie, no auth checks
                AuthRoute::RegisterEmail => RouteAuthKind::NoAuthCookieSetter,
                AuthRoute::SigninEmail => RouteAuthKind::NoAuthCookieSetter,
                // signout is allowed even if we've already "signed out everywhere"
                AuthRoute::Signout => RouteAuthKind::PartialAuthTokenOnly,
                // sending an email validation requires that the user is fully signed in (i.e. also hasn't been signed out elsewhere)
                // but not that their email is valid (that's the purpose of sending a link in the first place)
                AuthRoute::SendVerifyEmail => RouteAuthKind::PartialAuthAndUserTokenOnly,
                // request for a password reset can be done by anyone
                // because the point is they aren't able to login at all
                AuthRoute::SendPasswordResetAny => RouteAuthKind::None,
                // Uses an OOB token, so no auth token is needed, it's just a click from email
                AuthRoute::ConfirmEmailValidation => RouteAuthKind::None,
                // this is also via an OOB token, but needs to be able to log the user in
                AuthRoute::ConfirmPasswordReset => RouteAuthKind::NoAuthCookieSetter,
                // most of the openid routes are public (user isn't logged in yet at all), but the finalize exec needs to set the cookie
                AuthRoute::OpenIdConnect => RouteAuthKind::None,
                AuthRoute::OpenIdAccessTokenHook(_) => RouteAuthKind::None,
                AuthRoute::OpenIdFinalizeQuery => RouteAuthKind::None,
                AuthRoute::OpenIdFinalizeExec => RouteAuthKind::NoAuthCookieSetter,
            },
            Route::Admin(_) => RouteAuthKind::Admin,
            Route::Info => RouteAuthKind::None,
        }
    }
}

impl AuthRoute {
    pub fn try_from_paths(paths: &[&str]) -> Option<Self> {
        match *paths {
            ["register-email"] => Some(Self::RegisterEmail),
            ["send-verify-email"] => Some(Self::SendVerifyEmail),
            ["confirm-email-validation"] => Some(Self::ConfirmEmailValidation),
            ["send-password-reset-any"] => Some(Self::SendPasswordResetAny),
            ["send-password-reset-me"] => Some(Self::SendPasswordResetMe),
            ["confirm-password-reset"] => Some(Self::ConfirmPasswordReset),
            ["openid-connect"] => Some(Self::OpenIdConnect),
            ["openid-access-token-hook", provider] => OpenIdProvider::try_from_str(provider).map(Self::OpenIdAccessTokenHook),
            ["openid-finalize-exec"] => Some(Self::OpenIdFinalizeExec),
            ["openid-finalize-query"] => Some(Self::OpenIdFinalizeQuery),
            ["check"] => Some(Self::Check),
            ["signout"] => Some(Self::Signout),
            ["signin-email"] => Some(Self::SigninEmail),
            _ => None,
        }
    }
}

impl AdminRoute {
    pub fn try_from_paths(paths: &[&str]) -> Option<Self> {
        match *paths {
            ["placeholder"] => Some(Self::Placeholder),
            _ => None,
        }
    }
}

impl std::fmt::Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            Self::Auth(auth_route) => {
                format!("auth/{}", auth_route)
            }
            Self::Admin(admin_route) => {
                format!("admin/{}", admin_route)
            }
            Self::Info => "info".to_string(),
        };

        write!(f, "{}", s)
    }
}
impl std::fmt::Display for AuthRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            Self::RegisterEmail => "register-email".to_string(),
            Self::Signout => "signout".to_string(),
            Self::Check => "check".to_string(),
            Self::SigninEmail => "signin-email".to_string(),
            Self::SendVerifyEmail => "send-verify-email".to_string(),
            Self::ConfirmEmailValidation => "confirm-email-validation".to_string(),
            Self::SendPasswordResetAny => "send-password-reset-any".to_string(),
            Self::SendPasswordResetMe => "send-password-reset-me".to_string(), 
            Self::ConfirmPasswordReset => "confirm-password-reset".to_string(), 
            Self::OpenIdConnect => "openid-connect".to_string(),
            Self::OpenIdAccessTokenHook(provider) => format!("openid-access-token-hook/{}", provider.as_str()),
            Self::OpenIdFinalizeExec => "openid-finalize-exec".to_string(),
            Self::OpenIdFinalizeQuery => "openid-finalize-query".to_string(),
        };

        write!(f, "{}", s)
    }
}

impl std::fmt::Display for AdminRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            Self::Placeholder => "placeholder".to_string(),
        };

        write!(f, "{}", s)
    }
}

#[derive(PartialEq, Debug)]
pub enum RouteAuthKind {
    /// No credentials sent or needed at all, plain ol' public access
    None,
    /// Admin-only
    Admin,
    /// Full protection
    /// token, user_token, and email must all be validated
    Full,
    /// Just the ability to send and set cookies, tokens aren't checked at all
    /// e.g. for the login/register routes, which must allow backend to set the cookie
    NoAuthCookieSetter,
    /// All credentials are sent, but only auth token is validated
    /// user_token is not checked
    /// e.g. called from signout route so that the auth token can be invalidated on that device
    /// regardless of whether the session is still active across other devices
    /// but we don't want to allow signout for arbitrary users
    PartialAuthTokenOnly,
    // All credentials are sent, all tokens are verified, but current email is not verified
    // e.g. for validate email flow itself (not used if email isn't a thing system-wide)
    PartialAuthAndUserTokenOnly,
}
