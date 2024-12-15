use std::sync::LazyLock;

use wasm_bindgen_futures::spawn_local;

use crate::{prelude::*, util::mixins::handle_on_click};

pub struct Sidebar {}

impl Sidebar {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }

    pub fn render(self: Arc<Self>) -> Dom {
        static CONTAINER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("gap", "1.3125rem")
                .style("align-items", "flex-start")
            }
        });

        static LOGO_CONTAINER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("justify-content", "center")
                .style("align-items", "center")
                .style("padding", "1rem 2rem")
                .style("width", "100%")
                .style("border-bottom", &format!("1px solid {}", ColorRaw::GreyAlt1.value()))
            }
        });
        static LOGO: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("align-items", "center")
            }
        });

        html!("div", {
            .class(&*CONTAINER)
            .child(html!("div", {
                .class(&*LOGO_CONTAINER)
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
                            .text(&get_text!("dashboard-logo-label"))
                        }))
                    }))
                }))
            }))
            .children([
                self.render_button(Dashboard::Profile),
            ])
            .child(html!("div", {
                .class([&*BUTTON_BG_CLASS, FontSize::Xlg.class()])
                .text(&get_text!("dashboard-label-signout"))
                .event(|_: events::Click| {
                    spawn_local(async {
                        if let Err(err) = AUTH.signout(false).await {
                            tracing::error!("signout failed");
                            tracing::error!("{:?}", err);
                        }

                        Route::Landing(Landing::Welcome).go_to_url();
                    });
                })
            }))
        })
    }

    fn render_button(self: &Arc<Self>, dashboard: Dashboard) -> Dom {
        let selected_sig = Route::signal().map(move |route| match route {
            Route::Dashboard(route) => route == dashboard,
            _ => false,
        });

        html!("div", {
            .class([&*BUTTON_BG_CLASS, FontSize::Xlg.class(), &*USER_SELECT_NONE])
            .class_signal([&*BUTTON_BG_SELECTED, FontWeight::Bold.class()] , selected_sig)
            .text(&get_text!(match dashboard {
                Dashboard::Profile => "dashboard-label-profile",
            }))
            .apply(handle_on_click(move || {
                Route::Dashboard(dashboard).go_to_url();
            }))
        })
    }
}

static BUTTON_BG_CLASS: LazyLock<String> = LazyLock::new(|| {
    class! {
            .style("cursor", "pointer")
            .style("display", "flex")
            .style("justify-content", "flex-start")
            .style("align-items", "center")
            .style("gap", "1.5rem")
            .style("width", "100%")
            .style("padding", "1.25rem 2.88rem")
    }
});

static BUTTON_BG_SELECTED: LazyLock<String> = LazyLock::new(|| {
    class! {
            .style("background-color", ColorRaw::GreyAlt1.value())
    }
});
