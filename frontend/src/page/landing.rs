use crate::{page::landing::{header::Header, welcome::Welcome}, prelude::*};

mod header;
mod welcome;
pub mod auth;


pub fn section_signal() -> impl Signal<Item = Landing> {
    Route::signal().map(|route| {
        match route {
            Route::Landing(landing) => landing,
            _ => unreachable!("Landing route signal should only emit Landing routes!") 
        }
    })
}


pub struct LandingPage {
}

impl LandingPage {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
        })
    }

    pub fn render(self: Arc<Self>) -> Dom {
        let state = self;
        html!("main", {
            .style("display", "flex")
            .style("flex-direction", "column")
            .style("min-height", "100%")
            .style("padding", "1.56rem 2.5rem")
            .child(html!("div", {
                .style("flex", "1")
                .child(Header::new().render())
                .child_signal(section_signal().map(clone!(state => move |section| {
                    match section {
                        Landing::Welcome => {
                            Some(Welcome::new().render())
                        },
                        Landing::Auth(auth_route)=> {
                            Some(auth::render(auth_route))
                        },
                    }
                })))
            }))
            .child(LanguageSelector::render())
        })
    }
}
