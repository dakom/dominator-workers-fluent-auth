pub mod actions;
mod login;
mod register;
mod verify_email_waiting;
mod verify_email_confirm;

use crate::prelude::*;
use login::Signin;
use register::Register;
use verify_email_waiting::VerifyEmailWaiting;
use verify_email_confirm::VerifyEmailConfirm;

pub fn render(auth_route: AuthRoute) -> Dom {
    match auth_route {
        AuthRoute::Register => Register::new().render(),
        AuthRoute::Signin => Signin::new().render(),
        AuthRoute::VerifyEmailWaiting => VerifyEmailWaiting::new().render(),
        AuthRoute::VerifyEmailConfirm{token_id, token_key} => VerifyEmailConfirm::new(token_id, token_key).render(),
        _ => html!("div", {
            .text("404")
        }),
    }
}
