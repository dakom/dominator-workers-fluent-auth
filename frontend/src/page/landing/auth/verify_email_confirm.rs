use shared::{auth::FRONTEND_ROUTE_AFTER_LOGIN, backend::result::ApiError};

use crate::{page::landing::auth::actions::verify_email, prelude::*};

pub struct VerifyEmailConfirm {
    phase: Mutable<Phase>,
    token_id: String,
    token_key: String,
}

#[derive(Clone, Debug)]
enum Phase {
    Loading,
    Success,
    Error(ApiError),
}

impl VerifyEmailConfirm {
    pub fn new(token_id: String, token_key: String) -> Arc<Self> {
        Arc::new(Self {
            phase: Mutable::new(Phase::Loading),
            token_id,
            token_key,
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
            .future(clone!(state => async move {
                match verify_email(state.token_id.clone(), state.token_key.clone()).await {
                    Ok(_) => {
                        FRONTEND_ROUTE_AFTER_LOGIN.go_to_url();
                        state.phase.set(Phase::Success);
                    },
                    Err(err) => {
                        state.phase.set(Phase::Error(err));
                    }
                }
            }))
            .child_signal(state.phase.signal_cloned().map(|phase| {
                match phase {
                    Phase::Success => {
                        None
                    },
                    Phase::Loading => {
                        Some(html!("div", {
                            .class([&*CONTAINER, FontSize::H3.class()])
                            .text(&get_text!("landing-please-wait"))
                        }))
                    },
                    Phase::Error(err) => {
                        Some(html!("div", {
                            .class([&*CONTAINER, FontSize::H3.class()])
                            .child(html!("div", {
                                .class(ColorText::Error.class())
                                .text(&err.get_text())
                            }))
                        }))
                    }
                }
            }))
        })
    }
}