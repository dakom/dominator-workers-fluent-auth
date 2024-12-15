mod profile;
mod sidebar;

use std::sync::LazyLock;

use profile::ProfileUi;

use crate::prelude::*;

pub struct DashboardPage {}

impl DashboardPage {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }

    pub fn render(self: Arc<Self>) -> Dom {
        static CONTAINER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
            }
        });

        static SIDEBAR: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("flex-shrink", "0")
                .style("min-height", "100vh")
                .style("background-color", ColorBackground::Sidebar.value())
            }
        });

        static CONTENT: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("flex-grow", "1")
                .style("padding", "2rem")
                .style("width", "100%")
            }
        });

        html!("div", {
            .class(&*CONTAINER)
            .child(html!("div", {
                .class(&*SIDEBAR)
                .child(sidebar::Sidebar::new().render())
            }))
            .child_signal(Route::signal().map(|route| {
                match route {
                    Route::Dashboard(dashboard) => {
                        Some(html!("div", {
                            .class(&*CONTENT)
                            .child(match dashboard {
                                Dashboard::Profile => ProfileUi::new().render(),
                            })
                        }))
                    },
                    _ => None
                }
            }))
        })
    }
}
