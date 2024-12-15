use crate::{page::header::Header, prelude::*};

#[derive(Debug, Clone, PartialEq)]
pub struct NotFoundPage {}

impl NotFoundPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self) -> Dom {
        html!("div", {
            .style("display", "flex")
            .style("flex-direction", "column")
            .style("min-height", "100%")
            .style("padding", "1.56rem 2.5rem")
            .child(html!("div", {
                .style("flex", "1")
                .child(Header::new().render())
                .child(html!("div", {
                    .class([FontSize::H1.class()])
                    .style("margin-top", "20px")
                    .style("text-align", "center")

                    .text_signal(Route::signal().map(|route| {
                        match route {
                            Route::NotFound(reason) => {
                                match reason {
                                    NotFoundReason::NoAuth => "No Auth",
                                    NotFoundReason::BadUrl => "Bad Url",
                                }
                            }
                            _ => unreachable!("Landing route signal should only emit Landing routes!")
                        }
                    }))
                }))
            }))
        })
    }
}
