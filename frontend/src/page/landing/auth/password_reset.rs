use dominator_helpers::futures::AsyncLoader;
use super::{check_password_reset, confirm_password_reset};
use crate::{atoms::input::{TextInput, TextInputKind}, prelude::*};

pub(super) struct VerifyPasswordResetConfirm {
    pub oob_token_id: String,
    pub oob_token_key: String,
    pub phase: Mutable<VerifyPasswordResetConfirmPhase>,
    pub email: Mutex<Option<String>>,
    pub password: TextInput,
}

#[derive(Clone, Debug, PartialEq)]
enum VerifyPasswordResetConfirmPhase {
    CheckingValidity,
    Invalid,
    Waiting,
    Confirming,
    Success,
    Fail
}

impl VerifyPasswordResetConfirm {
    pub fn new(oob_token_id: String, oob_token_key: String) -> Arc<Self> {
        Arc::new(Self {
            oob_token_id,
            oob_token_key,
            phase: Mutable::new(VerifyPasswordResetConfirmPhase::CheckingValidity),
            email: Mutex::new(None),
            password: TextInput::new(TextInputKind::Password),
        })
    }

    pub fn render(self: Arc<Self>) -> Dom {
        let state = self;
        html!("div", {
            .future(clone!(state => async move {
                match check_password_reset(state.oob_token_id.clone(), state.oob_token_key.clone()).await {
                    Ok(resp) => {
                        log::info!("resetting password for user {:?}", resp);
                        *state.email.lock().unwrap_ext() = Some(resp.email);
                        state.phase.set_neq(VerifyPasswordResetConfirmPhase::Waiting);
                    },
                    Err(e) => {
                        state.phase.set_neq(VerifyPasswordResetConfirmPhase::Invalid);
                    }
                }
            }))
            .child_signal(state.phase.signal_cloned().map(clone!(state => move |phase| {
                match phase {
                    VerifyPasswordResetConfirmPhase::CheckingValidity => None,
                    VerifyPasswordResetConfirmPhase::Invalid => {
                        Some(state.render_invalid())
                    },
                    _ => {
                        Some(state.render_ready())
                    }
                }
            })))
        })
    }

    fn render_invalid(&self) -> Dom {
        static ERROR_MESSAGE:Lazy<String> = Lazy::new(|| {
            class! {
                .style("color", ColorSemantic::Error.to_str())
                .style("padding", "5.19rem 0 4.81rem 0")
            }
        });

        html!("div", {
            .style("color", ColorSemantic::Error.to_str())
            .style("padding", "5.19rem 0 4.81rem 0")
            .style("text-align", "center")
            .class(&*TEXT_SIZE_LG)
            .text(&get_text!("error-api-password-reset-invalid-link"))
        })
    }
    fn render_ready(self: &Arc<Self>) -> Dom {
        let state = self;
        static ERROR_MESSAGE:Lazy<String> = Lazy::new(|| {
            class! {
                .style("color", ColorSemantic::Error.to_str())
                .style("padding", "5.19rem 0 4.81rem 0")
            }
        });

        static CONTAINER:Lazy<String> = Lazy::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("align-items", "center")
                .style("justify-content", "center")
            }
        });

        static INPUTS:Lazy<String> = Lazy::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("align-items", "center")
                .style("gap", "1.875rem")
            }
        });

        let error = ApiErrorDisplay::new();
        let loader = AsyncLoader::new();
        let resent = Mutable::new(false);

        html!("div", {
            .class(&*CONTAINER)
            .child(html!("div", {
                .class(&*TEXT_SIZE_LG)
                .class(&*TEXT_WEIGHT_BOLD)
                .class(&*ERROR_MESSAGE)
                .text_signal(error.text_signal())
            }))
            .child(html!("div", {
                .text(&get_text!("landing-reset-password-header"))
            }))
            .child(state.password.render(Some(&get_text!("landing-signin-form-password"))))
            .child(html!("button", {
                .text("Submit")
                .event(clone!(state, error => move |_:events::Click| {
                    error.clear();
                    loader.load(clone!(state, resent, error => async move {
                        let password = state.password.value.get_cloned().unwrap_or_default();
                        match confirm_password_reset(state.oob_token_id.clone(), state.oob_token_key.clone(), &state.email.lock().unwrap().as_ref().unwrap(), &password).await {
                            Ok(_) => {
                                resent.set_neq(true);
                            },
                            Err(e) => {
                                error.set(e);
                            }
                        }
                    }));
                }))
            }))
        })
    }

}