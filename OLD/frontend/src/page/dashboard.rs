use std::sync::Mutex;

use wasm_bindgen_futures::spawn_local;

use crate::{atoms::buttons::Squareish1Button, prelude::*};

pub struct DashboardPage {
}


impl DashboardPage {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
        })
    }

    pub fn render(self: Arc<Self>) -> Dom {
        html!("div", {
            .style("display", "flex")
            .style("flex-direction", "column")
            .style("min-height", "100%")
            .style("padding", "1.56rem 2.5rem")
            .child(html!("div", {
                .style("flex", "1")
                .style("margin-top", "3rem")
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("align-items", "center")
                .style("gap", "1rem")
                .child(html!("div", {
                    .class(&*TEXT_SIZE_LG)
                    .text(&get_text!("dashboard-user-id", {
                        "userId" => AUTH.try_clone_uid().map(|uid| uid.to_string()).unwrap_or_else(|| "none".to_string())
                    }))
                }))
                .child(Squareish1Button::new().render(
                    get_text!("dashboard-signout-button"),
                    || {
                        spawn_local(async {
                            if let Err(err) = AUTH.signout().await { 
                                log::error!("signout failed");
                                log::error!("{:?}", err);
                            }
                            Route::Landing(Landing::Welcome).go_to_url();
                        });
                    }
                ))
            }))
            .child(LanguageSelector::render())
        })
    }
}