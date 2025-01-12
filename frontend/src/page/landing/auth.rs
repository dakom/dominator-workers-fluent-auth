mod register;
mod login;
pub mod actions;

use crate::prelude::*;
use register::Register;
use login::Signin;

pub fn render(auth_route: AuthRoute) -> Dom {
    match auth_route {
        AuthRoute::Register => Register::new().render(),
        AuthRoute::Signin => Signin::new().render(),
        _ => html!("div", {
            .text("404")
        })
    }
}
