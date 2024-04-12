use futures_signals::signal::{option, Mutable, Signal, SignalExt};
use shared::backend::result::{ApiError, AuthError};

use crate::{get_text, LOCALE};

pub trait ApiErrorExt {
    fn get_text(self) -> String;
}

impl ApiErrorExt for ApiError {
    fn get_text(self) -> String {
        // this goes through the fluent bindings
        let (id, args) = match self {
            Self::Auth(auth_error) => match auth_error {
                AuthError::EmailAlreadyExists => ("error-api-register-email-already-exists", None),
                AuthError::EmailNotVerified => ("error-api-register-email-unverified", None),
                AuthError::NotAuthorized => ("error-api-not-authorized", None),
                AuthError::InvalidSignin => ("error-api-signin-invalid", None),
                AuthError::NoUserPasswordReset => ("error-api-password-reset-no-user", None),
            },
            Self::Unknown(_) => ("error-api-unknown", None),
        };

        get_text!(id, args)
    }
}

// A component that makes it convenient to handle API errors for display
#[derive(Clone)]
pub struct ApiErrorDisplay {
    inner: Mutable<Option<ApiError>>
}

impl ApiErrorDisplay {
    pub fn new() -> Self {
        Self {
            inner: Mutable::new(None)
        }
    }

    pub fn set(&self, error: ApiError) {
        self.inner.set(Some(error));
    }

    pub fn clear(&self) {
        self.inner.set(None);
    }

    pub fn text_signal(&self) -> impl Signal<Item = String> {
        self.inner.signal_cloned().map(|err| {
            err.map(|err| err.get_text())
                .unwrap_or_default()
        })
    }
}