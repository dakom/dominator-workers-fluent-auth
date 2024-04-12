use dominator::pseudo;

use crate::{atoms::buttons::Squareish1Button, page::landing::Landing, prelude::*};

pub struct Welcome {
}


impl Welcome {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }
    pub fn render(self: Arc<Self>) -> Dom {
        static LINK_UNDERLINE:Lazy<String> = Lazy::new(|| {
            class! {
                .pseudo!(":hover", {
                    .style("text-decoration", "underline")
                })
            }
        });
        html!("div", {
            .style("margin-top", "5.25rem")
            .style("text-align", "center")
            .style("display", "flex")
            .style("flex-direction", "column")
            .children([
                html!("div", {
                    .class(&*TEXT_SIZE_H1)
                    .class(&*COLOR_HEADER)
                    .text(&get_text!("landing-welcome-header"))
                    .style("margin-bottom", "1rem")
                }),
                html!("a", {
                    .attr("href", "https://github.com/dakom/dominator-workers-fluent-auth")
                    .class(&*TEXT_SIZE_H2)
                    .class(&*COLOR_BYLINE)
                    .class(&*LINK_UNDERLINE)
                    .text(&get_text!("landing-welcome-byline"))
                    .style("margin-bottom", "4rem")
                }),
                html!("div", {
                    .style("display", "inline-block")
                    .child(Squareish1Button::new().render(
                        get_text!("landing-header-nav-start"), 
                        || {
                            Route::Landing(Landing::Auth(AuthRoute::Signin)).go_to_url();
                        }
                    ))
                })
            ])
        })
    }
}