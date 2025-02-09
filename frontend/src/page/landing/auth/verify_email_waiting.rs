use dominator_helpers::futures::AsyncLoader;

use crate::{atoms::buttons::{Button, ButtonStyle}, page::landing::auth::actions::resend_verification_email, prelude::*};

pub struct VerifyEmailWaiting {
    pub loader: AsyncLoader,
    pub error: ApiErrorDisplay,
    pub sent: Mutable<bool>,
}

impl VerifyEmailWaiting {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            loader: AsyncLoader::new(),
            error: ApiErrorDisplay::new(),
            sent: Mutable::new(false),
        })
    }

    pub fn render(self: &Arc<Self>) -> Dom {
        let state = self;

        static CONTAINER:LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("align-items", "center")
                .style("justify-content", "center")
                .style("text-align", "center")
                .style("margin-top", "3rem")
                .style("gap", "2rem")
            }
        });

        html!("div", {
            .class([&*CONTAINER, FontSize::H3.class()])
            .child(html!("div", {
                .text(&get_text!("landing-verify-email-waiting-header"))
            }))
            .child(Button::new()
                .with_text(get_text!("landing-verify-email-resend"))
                .with_style(ButtonStyle::Solid)
                .with_on_click(clone!(state => move || {
                    state.loader.load(clone!(state => async move {
                        state.sent.set_neq(false);
                        match resend_verification_email().await {
                            Ok(_) => {
                                state.sent.set_neq(true);
                            },
                            Err(err) => {
                                state.error.set(err);
                            }
                        }
                    }));
                }))
                .render()
            )
            .child_signal(state.sent.signal().map(|sent| {
                if sent {
                    Some(html!("div", {
                        .text(&get_text!("landing-verify-email-waiting-sent"))
                    }))
                } else {
                    None
                }
            }))
            .child_signal(state.error.text_signal().map(|err| {
                if err.is_empty() {
                    None
                } else {
                    Some(html!("div", {
                        .class(ColorText::Error.class())
                        .text(&err)
                    }))
                }
            }))
        })
    }
}