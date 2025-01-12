use dominator_helpers::futures::AsyncLoader;
use shared::api::auth::OpenIdProvider;
use shared::{auth::FRONTEND_ROUTE_AFTER_LOGIN, backend::result::AuthError};
use crate::atoms::buttons::{Button, ButtonStyle};
use crate::atoms::text_input::{TextInput, TextInputKind};
use crate::page::landing::auth::actions;
use crate::prelude::*;
use crate::atoms::checkbox::Checkbox;

pub(super) struct Register {
    pub error: ApiErrorDisplay,
    pub email: Arc<Mutex<Option<String>>>,
    pub password: Arc<Mutex<Option<String>>>,
    pub loader: AsyncLoader,
    pub terms_agreed: Mutable<bool>,
}

impl Register {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            error: ApiErrorDisplay::new(),
            email: Arc::new(Mutex::new(None)),
            password: Arc::new(Mutex::new(None)),
            loader: AsyncLoader::new(),
            terms_agreed: Mutable::new(false),
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
                .style("gap", "1.875rem")
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
                .text(&get_text!("landing-register-title"))
            }))
            .child(state.render_error())
            .child(html!("div", {
                .child(html!("div", {
                    .class(&*AREA_SPLIT)
                    .child(html!("div", {
                        .class(&*INPUTS)

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
                        .child(html!("div", {
                            .style("width", "100%")
                            .class(&*BUTTONS)
                            .child(Button::new()
                                .with_text(get_text!("landing-register-button"))
                                .with_on_click(clone!(state => move || {
                                    if !state.terms_agreed.get() {
                                        state.error.set(AuthError::TermsNotAgreed.into());
                                        return;
                                    }
                                    state.loader.load(clone!(state => async move {
                                        state.error.clear();

                                        let email_address = state.email.lock().unwrap_ext().clone().unwrap_or_default();
                                        if email_address.is_empty() {
                                            state.error.set(AuthError::EmailEmpty.into());
                                        } else {
                                            match actions::register_email(&email_address, &state.password.lock().unwrap_ext().clone().unwrap_or_default()).await {
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
                            .with_text(get_text!("landing-register-google-button"))
                            .with_style(ButtonStyle::Outline)
                            .with_content_before(html!("img", {
                                .style("height", "2rem")
                                .attr("src", &CONFIG.app_image_url("google.svg"))
                            }))
                            .with_on_click(clone!(state => move || {
                                if !state.terms_agreed.get() {
                                    state.error.set(AuthError::TermsNotAgreed.into());
                                    return;
                                }
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
                            .with_text(get_text!("landing-register-facebook-button"))
                            .with_style(ButtonStyle::Outline)
                            .with_content_before(html!("img", {
                                .style("height", "2rem")
                                .attr("src", &CONFIG.app_image_url("facebook.svg"))
                            }))
                            .with_on_click(clone!(state => move || {
                                if !state.terms_agreed.get() {
                                    state.error.set(AuthError::TermsNotAgreed.into());
                                    return;
                                }
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
                    .style("display", "flex")
                    .style("gap", "1rem")
                    .child(
                        Checkbox::new()
                        .with_selected_signal(state.terms_agreed.signal())
                        .with_on_click(clone!(state => move || {
                            state.terms_agreed.set_neq(!state.terms_agreed.get_cloned());
                        }))
                        .with_content_after(html!("div", {
                            .children([
                                html!("span", {
                                    .class(&*SPACE_AFTER)
                                    .text(&get_text!("landing-register-terms-label-agree"))
                                }),
                                link!(Route::TermsOfService.link_ext(), {
                                    .class(&*SPACE_AFTER)
                                    .style("color", ColorRaw::Accent.value())
                                    .text(&get_text!("landing-register-terms-link-tos"))
                                }),
                                html!("span", {
                                    .class(&*SPACE_AFTER)
                                    .text(&get_text!("landing-register-terms-label-and"))
                                }),
                                link!(Route::PrivacyPolicy.link_ext(), {
                                    .style("color", ColorRaw::Accent.value())
                                    .text(&get_text!("landing-register-terms-link-privacy"))
                                }),
                            ])
                        }))
                        .render()
                    )
                }))
                .child(html!("div", {
                    .style("margin-top", "1.875rem")
                    .style("width", "100%")
                    .child(html!("hr"))
                    .child(html!("div", {
                        .style("margin-top", "1.875rem")
                        .style("display", "flex")
                        .style("justify-content", "center")
                        .style("align-items", "center")
                        .style("gap", "0.625rem")
                        .child(html!("div", {
                            .class(FontSize::Lg.class())
                            .text(&get_text!("landing-register-footer"))
                        }))
                        .child(html!("div", {
                            .class(&*BUTTONS)

                            .child(Button::new()
                                .with_text(get_text!("landing-login-button"))
                                .with_style(ButtonStyle::Outline)
                                .with_link(Route::Landing(Landing::Auth(AuthRoute::Signin)).link_ext())
                                .render()
                            )
                        }))
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
}