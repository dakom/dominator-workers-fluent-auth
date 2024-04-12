use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum Route {
    Auth(AuthRoute),
}

#[derive(Debug, Clone)]
pub enum AuthRoute {
    Register,
    Signin,
    Check,
    Signout,
    SendEmailValidation,
    ConfirmEmailValidation,
    SendPasswordResetAny,
    SendPasswordResetMe,
    ConfirmPasswordReset,
    CheckPasswordReset,
    OpenIdConnect,
    OpenIdAccessTokenHook(OpenIdProvider),
    OpenIdFinalizeExec,
    OpenIdFinalizeQuery,
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
                AuthRoute::Register => RouteAuthKind::CookiesOnly,
                AuthRoute::Signin => RouteAuthKind::CookiesOnly,
                // signout only needs the auth token, as that is the only thing it destroys
                AuthRoute::Signout => RouteAuthKind::PartialAuthTokenOnly,
                // sending an email validation requires that the user is signed in
                // but not that their email is valid (that's the purpose of sending a link in the first place)
                AuthRoute::SendEmailValidation => RouteAuthKind::PartialAuthAndUserTokenOnly,
                AuthRoute::SendPasswordResetAny => RouteAuthKind::None,
                AuthRoute::SendPasswordResetMe => RouteAuthKind::Full,
                // these use OOB tokens, so no auth token is needed, it's just a click from email
                AuthRoute::ConfirmEmailValidation => RouteAuthKind::None,
                // well, actually, this one signs the user in too :P
                AuthRoute::ConfirmPasswordReset => RouteAuthKind::CookiesOnly,
                AuthRoute::CheckPasswordReset => RouteAuthKind::None,
                AuthRoute::Check => RouteAuthKind::Full,
                AuthRoute::OpenIdConnect => RouteAuthKind::None,
                AuthRoute::OpenIdAccessTokenHook(_) => RouteAuthKind::None,
                AuthRoute::OpenIdFinalizeExec => RouteAuthKind::CookiesOnly,
                AuthRoute::OpenIdFinalizeQuery => RouteAuthKind::None,
            },
        }
    }
}


impl AuthRoute {
    pub fn try_from_paths(paths: &[&str]) -> Option<Self> {
        match *paths {
            ["register"] => Some(Self::Register),
            ["signin"] => Some(Self::Signin),
            ["signout"] => Some(Self::Signout),
            ["check"] => Some(Self::Check),
            ["send-email-validation"] => Some(Self::SendEmailValidation),
            ["confirm-email-validation"] => Some(Self::ConfirmEmailValidation),
            ["send-password-reset-any"] => Some(Self::SendPasswordResetAny),
            ["send-password-reset-me"] => Some(Self::SendPasswordResetMe),
            ["confirm-password-reset"] => Some(Self::ConfirmPasswordReset),
            ["check-password-reset"] => Some(Self::CheckPasswordReset),
            ["openid-connect"] => Some(Self::OpenIdConnect),
            ["openid-access-token-hook", provider] => OpenIdProvider::try_from_str(provider).map(Self::OpenIdAccessTokenHook),
            ["openid-finalize-exec"] => Some(Self::OpenIdFinalizeExec),
            ["openid-finalize-query"] => Some(Self::OpenIdFinalizeQuery),
            _ => None
        }
    }


}

impl std::fmt::Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            Self::Auth(auth_route) => {
                format!("auth/{}", auth_route)
            }
        };

        write!(f, "{}", s)
    }
}
impl std::fmt::Display for AuthRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            Self::Register => "register".to_string(),
            Self::Signin => "signin".to_string(),
            Self::Signout => "signout".to_string(),
            Self::Check => "check".to_string(),
            Self::SendEmailValidation => "send-email-validation".to_string(),
            Self::ConfirmEmailValidation => "confirm-email-validation".to_string(),
            Self::SendPasswordResetAny => "send-password-reset-any".to_string(),
            Self::SendPasswordResetMe => "send-password-reset-me".to_string(),
            Self::ConfirmPasswordReset => "confirm-password-reset".to_string(),
            Self::CheckPasswordReset => "check-password-reset".to_string(),
            Self::OpenIdConnect => "openid-connect".to_string(),
            Self::OpenIdAccessTokenHook(provider) => format!("openid-access-token-hook/{}", provider.as_str()),
            Self::OpenIdFinalizeExec => "openid-finalize-exec".to_string(),
            Self::OpenIdFinalizeQuery => "openid-finalize-query".to_string(),
        };

        write!(f, "{}", s)
    }
}

#[derive(PartialEq, Debug)]
pub enum RouteAuthKind {
    /// Full protection
    /// token, user_token, and email must all be validated
    Full,
    /// Just the ability to send and set cookies, tokens aren't checked at all
    /// e.g. for the signin route, which must allow backend to set the cookie
    CookiesOnly,
    /// All credentials are sent, but only auth token is validated 
    /// user_token and email are not checked
    /// e.g. called from signout route so that the auth token can be invalidated on that device
    /// regardless of whether the session is still active across other devices
    PartialAuthTokenOnly,
    /// All credentials are sent, all tokens are verified, but current email is not verified
    /// e.g. for validate email flow itself
    PartialAuthAndUserTokenOnly,
    /// No credentials sent or needed at all, plain ol' full public access
    None
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum OpenIdProvider {
    Google,
    Facebook
}

impl OpenIdProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Facebook => "facebook",
        }
    }

    pub fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "google" => Some(Self::Google),
            "facebook" => Some(Self::Facebook),
            _ => None
        }
    }
}

