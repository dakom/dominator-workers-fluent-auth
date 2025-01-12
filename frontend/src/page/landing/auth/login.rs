use crate::{atoms::{buttons::{Button, ButtonStyle}, text_input::{TextInput, TextInputKind}}, page::landing::auth::actions, prelude::*};
use shared::{
    api::auth::OpenIdProvider, auth::FRONTEND_ROUTE_AFTER_LOGIN, backend::result::AuthError
};

use dominator_helpers::futures::AsyncLoader;

pub(super) struct Signin {
    pub error: ApiErrorDisplay,
    pub email: Arc<Mutex<Option<String>>>, 
    pub password: Arc<Mutex<Option<String>>>, 
    pub loader: AsyncLoader,
    notice: Mutable<Option<SigninNotice>>,
}

#[derive(Clone, Debug, Copy, PartialEq)]
enum SigninNotice {
    PasswordReset,
}

impl Signin {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            error: ApiErrorDisplay::new(),
            notice: Mutable::new(None),
            email: Arc::new(Mutex::new(None)), 
            password: Arc::new(Mutex::new(None)), 
            loader: AsyncLoader::new(),
        })
    }
    pub fn render(self: Arc<Self>) -> Dom {
        let state = self;
        static CONTAINER:LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("align-items", "center")
                .style("justify-content", "center")
            }
        });

        static AREA_SPLIT:LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "row")
                .style("gap", "1.875rem")
            }
        });
        static INPUTS:LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("align-items", "center")
            }
        });
        static BUTTONS:LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("gap", "1.875rem")
            }
        });

        html!("div", {
            .class(&*CONTAINER)
            .child(html!("div", {
                .class(FontSize::Xlg.class())
                .style("margin", "2rem 0")
                .text(&get_text!("landing-login-title"))
            }))
            .child(state.render_error())
            .child_signal(state.notice.signal_cloned().map(|notice| {
                notice.map(|notice| match notice {
                    SigninNotice::PasswordReset => {
                        html!("div", {
                            .class(FontSize::Lg.class())
                            .style("margin-bottom", "1.875rem")
                            .text(&get_text!("landing-login-notice-password-reset-sent"))
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
                            .child(TextInput::new()
                                .with_kind(TextInputKind::Email)
                                .with_placeholder(&get_text!("landing-auth-form-email"))
                                .with_on_input(clone!(state => move |val| {
                                    *state.email.lock().unwrap_ext() = val;
                                }))
                                .render()
                            )
                            .child(TextInput::new()
                                .with_kind(TextInputKind::Password)
                                .with_placeholder(&get_text!("landing-auth-form-password"))
                                .with_on_input(clone!(state => move |val| {
                                    *state.password.lock().unwrap_ext() = val;
                                }))
                                .render()
                            )
                        }))
                        .child(html!("div", {
                            .style("width", "100%")
                            .style("margin-top", "1.875rem")
                            .class(&*BUTTONS)
                            .child(Button::new()
                                .with_text(get_text!("landing-login-button"))
                                .with_on_click(clone!(state => move || {
                                    state.error.clear();
                                    state.loader.load(clone!(state => async move {
                                        let email_address = state.email.lock().unwrap_ext().clone().unwrap_or_default();
                                        if email_address.is_empty() {
                                            state.error.set(AuthError::EmailEmpty.into());
                                        } else {
                                            match actions::login_email(&email_address, &state.password.lock().unwrap_ext().clone().unwrap_or_default()).await {
                                                Ok(_) => {
                                                    FRONTEND_ROUTE_AFTER_LOGIN.go_to_url();
                                                },
                                                Err(e) => {
                                                    state.error.set(e);
                                                }
                                            }
                                        }
                                    }));
                                }))
                                .render()
                            )
                        }))
                        .child(html!("div", {
                            .style("width", "100%")
                            .style("margin-top", "3rem")
                            .child(html!("div", {
                                .class([&*USER_SELECT_NONE, FontSize::Lg.class(), &*CURSOR_POINTER, ColorText::Link.class()])
                                .text(&get_text!("landing-login-reset-password-button"))
                                .event(clone!(state => move |_:events::Click| {
                                    state.error.clear();
                                    state.loader.load(clone!(state => async move {
                                        let email_address = state.email.lock().unwrap_ext().clone().unwrap_or_default();
                                        if email_address.is_empty() {
                                            state.error.set(AuthError::EmailEmpty.into());
                                        } else {
                                            match actions::send_password_reset(Some(&email_address)).await {
                                                Ok(_) => {
                                                    state.notice.set_neq(Some(SigninNotice::PasswordReset));
                                                },
                                                Err(e) => {
                                                    state.error.set(e);
                                                }
                                            }
                                        }
                                    }));
                                }))
                            }))
                        }))
                    }))
                    .child(html!("div", {
                        .style("display", "flex")
                        .style("flex-direction", "column")
                        .style("justify-content", "center")
                        .child(html!("div", {
                            .class(FontSize::Lg.class())
                            .text(&format!(" - {} - ", get_text!("landing-or")))
                        }))
                    }))
                    .child(html!("div", {
                        .style("display", "flex")
                        .style("flex-direction", "column")
                        .class(&*BUTTONS)
                        .child(Button::new()
                            .with_text(get_text!("landing-login-google-button"))
                            .with_style(ButtonStyle::Outline)
                            .with_content_before(html!("img", {
                                .style("height", "2rem")
                                .attr("src", &CONFIG.app_image_url("google.svg"))
                            }))
                            .with_on_click(clone!(state => move || {
                                state.loader.load(clone!(state => async move {
                                    state.error.clear();
                                    match actions::openid_connect(OpenIdProvider::Google).await {
                                        Ok(_) => {
                                            // openid_connect will redirect
                                        },
                                        Err(e) => {
                                            state.error.set(e);
                                        }
                                    }
                                }));
                            }))
                            .render()
                        )
                        .child(Button::new()
                            .with_text(get_text!("landing-login-facebook-button"))
                            .with_style(ButtonStyle::Outline)
                            .with_content_before(html!("img", {
                                .style("height", "2rem")
                                .attr("src", &CONFIG.app_image_url("facebook.svg"))
                            }))
                            .with_on_click(clone!(state => move || {
                                state.loader.load(clone!(state => async move {
                                    state.error.clear();
                                    match actions::openid_connect(OpenIdProvider::Facebook).await {
                                        Ok(_) => {
                                            // openid_connect will redirect
                                        },
                                        Err(e) => {
                                            state.error.set(e);
                                        }
                                    }
                                }));
                            }))
                            .render()
                        )
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
        static ERROR_MESSAGE:LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", ColorText::Error.value())
                .style("margin-bottom", "2rem")
            }
        });
        html!("div", {
            .class(FontSize::Lg.class())
            .class(FontWeight::Bold.class())
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
                .class(FontSize::Lg.class())
                .text(&get_text!("landing-login-footer-no-account"))
            }))
            .child(html!("div", {
                .child(Button::new()
                    .with_text(get_text!("landing-register-button"))
                    .with_style(ButtonStyle::Outline)
                    .with_link(Route::Landing(Landing::Auth(AuthRoute::Register)).link_ext())
                    .render()
                )
            }))
        })
    }
}