use crate::{
    locale::LOCALE,
    prelude::*,
    theme::{responsive::Breakpoint, typography::FONT_FAMILY_NOTO},
};
use dominator::stylesheet;

pub fn init() {
    stylesheet!(":root", {
        .style("box-sizing", "border-box")
        .style_signal("direction", LOCALE.current.signal_cloned().map(|locale| locale.dir().as_str()))
        .style_signal("font-size", Breakpoint::signal().map(|breakpoint| {
            breakpoint.font_size()
        }))
    });

    stylesheet!("*, ::before, ::after", {
        .style("box-sizing", "inherit")
    });

    stylesheet!("html, body", {
        .style("margin", "0")
        .style("padding", "0")
        .style("width", "100%")
        .style("height", "100%")
        .style("font-family", FONT_FAMILY_NOTO)
    });

    stylesheet!("a", {
        .style("all", "unset")
        .style("cursor", "pointer")
    })
}
