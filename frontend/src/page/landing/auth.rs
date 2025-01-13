pub mod actions;
mod login;
mod register;

use crate::prelude::*;
use login::Signin;
use register::Register;

pub fn render(auth_route: AuthRoute) -> Dom {
    match auth_route {
        AuthRoute::Register => Register::new().render(),
        AuthRoute::Signin => Signin::new().render(),
        _ => html!("div", {
            .text("404")
        }),
    }
}
