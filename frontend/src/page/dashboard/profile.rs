use crate::prelude::*;

pub struct ProfileUi {}

impl ProfileUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }

    pub fn render(self: &Arc<Self>) -> Dom {
        let state = self;

        html!("div", {
            .text("TODO")
        })
    }
}
