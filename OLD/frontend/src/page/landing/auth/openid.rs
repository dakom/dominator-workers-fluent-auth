use std::sync::atomic::AtomicBool;

use super::{openid_session_query, openid_session_finalize};
use crate::{atoms::buttons::Squareish1Button, prelude::*};

pub(super) struct OpenIdFinalize {
    pub session_id: String,
    pub session_key: String,
    pub phase: Mutable<OpenIdFinalizePhase>,
    pub register_terms: AtomicBool,
}

#[derive(Clone, Debug, PartialEq)]
enum OpenIdFinalizePhase {
    Loading,
    Submitting,
    Invalid,
    AskForRegister,
}

impl OpenIdFinalize {
    pub fn new(session_id: String, session_key: String) -> Arc<Self> {
        Arc::new(Self {
            session_id,
            session_key,
            phase: Mutable::new(OpenIdFinalizePhase::Loading),
            register_terms: AtomicBool::new(false),
        })
    }

    pub fn render(self: Arc<Self>) -> Dom {
        let state = self;
        html!("div", {
            .future(clone!(state => async move {
                match openid_session_query(state.session_id.clone(), state.session_key.clone()).await {
                    Ok(resp) => {
                        if resp.user_exists {
                            // user already exists - just sign them in
                            // this distinction is handled automatically in the backend
                            // we're just making sure we get the user to agree to registration terms on the frontend
                            state.phase.set_neq(OpenIdFinalizePhase::Submitting);
                        } else {
                            state.phase.set_neq(OpenIdFinalizePhase::AskForRegister);
                        }
                    },
                    Err(e) => {
                        state.phase.set_neq(OpenIdFinalizePhase::Invalid);
                    }
                }
            }))
            .child_signal(state.phase.signal_cloned().map(clone!(state => move |phase| {
                match phase {
                    OpenIdFinalizePhase::Loading => None,
                    OpenIdFinalizePhase::Submitting => Some(state.render_submitting()),
                    OpenIdFinalizePhase::Invalid => {
                        Some(state.render_invalid())
                    },
                    OpenIdFinalizePhase::AskForRegister => {
                        Some(state.render_ask_for_register())
                    },
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
            .class(&*ERROR_MESSAGE)
            .class(&*TEXT_SIZE_LG)
            .style("text-align", "center")
            .text(&get_text!("error-api-openid-invalid"))
        })
    }

    fn render_submitting(self: &Arc<Self>) -> Dom {
        let state = self;
        html!("div", {
            .future(clone!(state => async move {
                if openid_session_finalize(state.session_id.clone(), state.session_key.clone()).await.is_err() {
                    state.phase.set_neq(OpenIdFinalizePhase::Invalid);
                }
            }))
            .class(&*TEXT_SIZE_LG)
            .text(&get_text!("landing-signing-in"))
        })
    }

    fn render_ask_for_register(self: &Arc<Self>) -> Dom {
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

        html!("div", {
            .class(&*CONTAINER)
            .child(html!("div", {
                .text(&get_text!("landing-agree-terms"))
            }))
            .child(Squareish1Button::new().render(
                get_text!("button-submit"),
                clone!(state => move || {
                    state.phase.set_neq(OpenIdFinalizePhase::Submitting);
                })
            ))
        })
    }

}