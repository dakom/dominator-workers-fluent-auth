use dominator_helpers::futures::AsyncLoader;
use shared::{auth::FRONTEND_ROUTE_AFTER_SIGNIN, backend::route::OpenIdProvider};
use super::{signin, send_password_reset, openid_connect};
use crate::{atoms::{buttons::{ButtonSize, OutlineButton, Squareish1Button}, input::{TextInput, TextInputKind}}, prelude::*};

pub(super) struct Signin {
    pub error: ApiErrorDisplay,
    pub notice: Mutable<Option<SigninNotice>>,
    pub email: TextInput,
    pub password: TextInput,
    pub loader: AsyncLoader,
}

#[derive(Clone, Debug, Copy, PartialEq)]
enum SigninNotice {
    EmailNotVerified,
    PasswordReset,
}

impl Signin {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            error: ApiErrorDisplay::new(),
            notice: Mutable::new(None),
            email: TextInput::new(TextInputKind::Email),
            password: TextInput::new(TextInputKind::Password),
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

        static AREA_SPLIT:Lazy<String> = Lazy::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "row")
                .style("gap", "1.875rem")
            }
        });
        static INPUTS:Lazy<String> = Lazy::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("align-items", "center")
            }
        });
        static BUTTONS:Lazy<String> = Lazy::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("gap", "1.875rem")
            }
        });

        html!("div", {
            .class(&*CONTAINER)
            .child(state.render_error())
            .child_signal(state.notice.signal_cloned().map(|notice| {
                notice.map(|notice| match notice {
                    SigninNotice::EmailNotVerified => {
                        html!("div", {
                            .class(&*TEXT_SIZE_LG)
                            .style("margin-bottom", "1.875rem")
                            .text(&get_text!("landing-signin-email-not-verified"))
                        })
                    },
                    SigninNotice::PasswordReset => {
                        html!("div", {
                            .class(&*TEXT_SIZE_LG)
                            .style("margin-bottom", "1.875rem")
                            .text(&get_text!("landing-signin-password-reset-sent"))
                        })
                    },
                })
            }))
            .child(html!("div", {
                .child(html!("div", {
                    .class(&*AREA_SPLIT)
                    .child(html!("div", {
                        .class(&*INPUTS)
                        .child(html!("div", {
                            .class(&*INPUTS)
                            .style("gap", "1.875rem")
                            .child(state.email.render(Some(&get_text!("landing-signin-form-email"))))
                            .child(state.password.render(Some(&get_text!("landing-signin-form-password"))))
                        }))

                        .child(html!("div", {
                            .style("width", "100%")
                            .style("margin-top", "1.875rem")
                            .class(&*BUTTONS)
                            .child(Squareish1Button::new().render(
                                get_text!("landing-signin-button"),
                                clone!(state => move || {
                                    state.error.clear();
                                    state.loader.load(clone!(state => async move {
                                        match signin(&state.email.value.get_cloned().unwrap_or_default(), &state.password.value.get_cloned().unwrap_or_default()).await {
                                            Ok(_) => {
                                                FRONTEND_ROUTE_AFTER_SIGNIN.go_to_url();
                                            },
                                            Err(e) => {
                                                state.error.set(e);
                                            }
                                        }
                                    }));
                                })
                            ))
                        }))
                        .child(html!("div", {
                            .style("margin-top", "1.875rem")
                            .style("align-self", "flex-start")
                            .child(OutlineButton::new(true).set_size(ButtonSize::Sm).render(
                                None,
                                get_text!("landing-reset-password-button"),
                                clone!(state => move || {
                                    state.error.clear();
                                    state.loader.load(clone!(state => async move {
                                        match send_password_reset(Some(&state.email.value.get_cloned().unwrap_or_default())).await {
                                            Ok(_) => {
                                                state.notice.set_neq(Some(SigninNotice::PasswordReset));
                                            },
                                            Err(e) => {
                                                state.error.set(e);
                                            }
                                        }
                                    }));
                                })
                            ))
                        }))
                    }))
                    .child(html!("div", {
                        .style("display", "flex")
                        .style("flex-direction", "column")
                        .style("justify-content", "center")
                        .child(html!("div", {
                            .class(&*TEXT_SIZE_LG)
                            .text(&format!(" - {} - ", get_text!("landing-or")))
                        }))
                    }))
                    .child(html!("div", {
                        .style("display", "flex")
                        .style("flex-direction", "column")
                        .style("justify-content", "center")
                        .class(&*BUTTONS)
                        .child(OutlineButton::new(false).render(
                            Some(html!("img", {
                                .style("height", "2rem")
                                .attr("src", &CONFIG.app_image_url("google-logo.svg"))
                            })),
                            get_text!("landing-signin-google-button"),
                            clone!(state => move || {
                                state.loader.load(clone!(state => async move {
                                    state.error.clear();
                                    match openid_connect(OpenIdProvider::Google).await {
                                        Ok(_) => {
                                            // openid_connect will redirect
                                        },
                                        Err(e) => {
                                            state.error.set(e);
                                        }
                                    }
                                }));
                            })
                        ))
                        .child(OutlineButton::new(false).render(
                            Some(html!("img", {
                                .style("height", "2rem")
                                .attr("src", &CONFIG.app_image_url("facebook-logo.svg"))
                            })),
                            get_text!("landing-signin-facebook-button"),
                            clone!(state => move || {
                                state.loader.load(clone!(state => async move {
                                    state.error.clear();
                                    match openid_connect(OpenIdProvider::Facebook).await {
                                        Ok(_) => {
                                            // openid_connect will redirect
                                        },
                                        Err(e) => {
                                            state.error.set(e);
                                        }
                                    }
                                }));
                            })
                        ))
                    }))
                }))
                .child(html!("div", {
                    .style("margin-top", "1.875rem")
                    .style("width", "100%")
                    .child(html!("hr"))
                    .child(html!("div", {
                        .style("margin-top", "1.875rem")
                        .style("display", "flex")
                        .style("flex-direction", "row")
                        .style("justify-content", "center")
                        .style("align-items", "center")
                        .style("gap", "3rem")
                        .child(state.render_create_account_line())
                    }))
                }))
            }))
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

    fn render_create_account_line(self: &Arc<Self>) -> Dom {
        html!("div", {
            .style("display", "flex")
            .style("justify-content", "center")
            .style("align-items", "center")
            .style("gap", "0.625rem")
            .child(html!("div", {
                .class(&*TEXT_SIZE_LG)
                .text(&get_text!("landing-signin-footer-no-account"))
            }))
            .child(html!("div", {
                .child(OutlineButton::new(true).render(
                    None,
                    get_text!("landing-create-account-button"),
                    || {
                        Route::Landing(Landing::Auth(AuthRoute::Register)).go_to_url();
                    }
                ))
            }))
        })
    }
}