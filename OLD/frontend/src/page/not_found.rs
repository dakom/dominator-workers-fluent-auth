use crate::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct NotFoundPage {
}


impl NotFoundPage {
    pub fn new() -> Self {
        Self {
        }
    }

    pub fn render(&self) -> Dom {
        html!("div", {
            .class([&*TEXT_SIZE_H1])
            .style("margin-top", "20px")
            .style("text-align", "center")
            .text_signal(Route::signal().map(|route| {
                match route {
                    Route::NotFound(reason) => {
                        match reason {
                            NotFoundReason::NoAuth => "No Auth",
                            NotFoundReason::BadUrl => "Bad Url",
                            NotFoundReason::NoOobCode => "No Oob Code",
                        }
                    }
                    _ => unreachable!("Landing route signal should only emit Landing routes!") 
                }
            }))
        })
    }
}