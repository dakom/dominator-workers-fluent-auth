use dominator_helpers::futures::AsyncLoader;
use shared::{auth::FRONTEND_ROUTE_AFTER_SIGNIN, backend::route::OpenIdProvider};
use super::{send_email_validation, confirm_email_validation};
use crate::{atoms::{buttons::Squareish1Button, input::TextInput}, prelude::*};

pub(super) struct VerifyEmailWaiting {
    pub error: ApiErrorDisplay,
    pub resent: Mutable<bool>,
    pub loader: AsyncLoader,
}

impl VerifyEmailWaiting {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            error: ApiErrorDisplay::new(),
            resent: Mutable::new(false),
            loader: AsyncLoader::new(),
        })
    }

    pub fn render(self: Arc<Self>) -> Dom {
        let state = self;
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

        html!("div", {
            .class(&*CONTAINER)
            .child(state.render_error())
            .child(html!("div", {
                .class(&*TEXT_SIZE_LG)
                .style("margin-bottom", "1rem")
                .text(&get_text!("landing-go-verify-email"))
            }))
            .child(Squareish1Button::new().render(
                get_text!("landing-resend-button"),
                clone!(state => move || {
                    state.error.clear();
                    state.loader.load(clone!(state => async move {
                        match send_email_validation().await {
                            Ok(_) => {
                                state.resent.set_neq(true);
                            },
                            Err(e) => {
                                state.error.set(e);
                            }
                        }
                    }));
                })
            ))
        })
    }

    pub fn render_error(&self) -> Dom {
        static ERROR_MESSAGE:Lazy<String> = Lazy::new(|| {
            class! {
                .style("color", ColorSemantic::Error.to_str())
                .style("padding", "5.19rem 0 4.81rem 0")
            }
        });
        html!("div", {
            .class(&*TEXT_SIZE_LG)
            .class(&*TEXT_WEIGHT_BOLD)
            .class(&*ERROR_MESSAGE)
            .text_signal(self.error.text_signal())
        })
    }
}


pub(super) struct VerifyEmailConfirm {
    pub oob_token_id: String,
    pub oob_token_key: String,
    pub phase: Mutable<VerifyEmailConfirmPhase>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum VerifyEmailConfirmPhase {
    Confirming,
    Success,
    Fail
}

impl VerifyEmailConfirm {
    pub fn new(oob_token_id: String, oob_token_key: String) -> Arc<Self> {
        Arc::new(Self {
            oob_token_id,
            oob_token_key,
            phase: Mutable::new(VerifyEmailConfirmPhase::Confirming),
        })
    }

    pub fn render(self: Arc<Self>) -> Dom {
        let state = self;
        html!("div", {
            .child_signal(state.phase.signal().map(clone!(state => move |phase| {
                Some(match phase {
                    VerifyEmailConfirmPhase::Confirming => {
                        state.render_confirming()
                    },
                    VerifyEmailConfirmPhase::Success => {
                        state.render_success()
                    },
                    VerifyEmailConfirmPhase::Fail => {
                        state.render_fail()
                    },
                })
            })))
        })
    }

    fn render_confirming(self: &Arc<Self>) -> Dom {
        let state = self;
        html!("div", {
            .future(clone!(state => async move {
                match confirm_email_validation(state.oob_token_id.clone(), state.oob_token_key.clone()).await {
                    Ok(_) => {
                        state.phase.set_neq(VerifyEmailConfirmPhase::Success);
                    },
                    Err(_) => {
                        state.phase.set_neq(VerifyEmailConfirmPhase::Fail);
                    }
                }
            }))
        })
    }

    fn render_success(&self) -> Dom {
        html!("div", {
            .future(async move {
                gloo_timers::future::TimeoutFuture::new(3000).await;
                FRONTEND_ROUTE_AFTER_SIGNIN.go_to_url();
            })
            .text(&get_text!("verify-email-success"))
        })
    }

    fn render_fail(&self) -> Dom {
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
                .text(&get_text!("landing-verify-email-error"))
            }))
            .child(Squareish1Button::new().render(
                get_text!("landing-resend-button"),
                clone!(resent, error => move || {
                    error.clear();
                    loader.load(clone!(resent, error => async move {
                        match send_email_validation().await {
                            Ok(_) => {
                                resent.set_neq(true);
                            },
                            Err(e) => {
                                error.set(e);
                            }
                        }
                    }));
                })
            ))
        })
    }

}