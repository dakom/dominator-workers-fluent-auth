use crate::{atoms::buttons::Button, auth::AuthPhase, prelude::*};

pub struct Header {}

impl Header {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }

    pub fn render(self: &Arc<Self>) -> Dom {
        static CLASS: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("justify-content", "space-between")
                .style("align-items", "center")
            }
        });

        static LOGO: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("align-items", "center")
            }
        });
        html!("div", {
            .class(&*CLASS)
            .child(link!(Route::Landing(Landing::Welcome).link_ext(), {
                .child(html!("div", {
                    .class(&*LOGO)
                    .child(html!("img", {
                        .style("width", "3rem")
                        .style("height", "3rem")
                        .attr("src", &CONFIG.app_image_url("logo.svg"))
                    }))
                    .child(html!("div", {
                        .style("font-size", "1.5rem")
                        .style("font-weight", "600")
                        .style("margin-left", "1rem")
                        .text(&get_text!("landing-header-logo-text"))
                    }))
                }))
            }))
            .apply_if(AUTH.phase.get() == AuthPhase::Authenticated, |dom| {
                dom.child(Button::new()
                    .with_text("Dashboard")
                    .with_link(Route::Dashboard(Dashboard::Profile).link_ext())
                    .render()
                )
            })
        })
    }
}
