use futures_signals::signal::always;

use crate::{atoms::buttons::{Squareish1Button, UnderlineButton}, prelude::*};

use super::Landing;

pub struct Header {
}

impl Header {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
        })
    }
    pub fn render(self: Arc<Self>) -> Dom {
        static CLASS:Lazy<String> = Lazy::new(|| {
            class! {
                .style("display", "flex")
                .style("justify-content", "center")
                .style("align-items", "center")
            }
        });
        html!("div", {
            .class(&*CLASS)
            .child(self.render_nav())
        })
    }

    fn render_nav(self: Arc<Self>) -> Dom {
        let state = self;
        static CLASS:Lazy<String> = Lazy::new(|| {
            class! {
                .style("display", "flex")
                .style("height", "3.4375rem")
                .style("justify-content", "flex-end")
                .style("align-items", "center")
                .style("gap", "2.6875rem")
            }
        });

        html!("nav", {
            .class(&*CLASS)
            .children([
                UnderlineButton::new().render(
                    get_text!("landing-header-nav-welcome"), 
                    || super::section_signal().map(|section| matches!(section, Landing::Welcome)), 
                    clone!(state => move || {
                        Route::Landing(Landing::Welcome).go_to_url();
                    })),
                UnderlineButton::new().render(
                    get_text!("landing-header-nav-start"), 
                    || super::section_signal().map(|section| matches!(section, Landing::Auth(_))), 
                    clone!(state => move || {
                        Route::Landing(Landing::Auth(AuthRoute::Signin)).go_to_url();
                    })),
            ])
        })
    }
}