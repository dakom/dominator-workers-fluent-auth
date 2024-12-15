use crate::prelude::*;

pub struct TermsOfService {}

impl TermsOfService {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }
    pub fn render(self: Arc<Self>) -> Dom {
        static CONTAINER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("align-items", "center")
                .style("justify-content", "center")
            }
        });

        static CONTENT: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("text-align", "center")
                .style("max-width", "30rem")
            }
        });

        html!("div", {
            .class(&*CONTAINER)
            .child(html!("div", {
                .class([&*CONTENT, FontSize::Md.class()])
                .text("Terms of Service")
            }))
        })
    }
}
