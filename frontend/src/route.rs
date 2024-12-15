use crate::{
    atoms::modal::Modal,
    auth::AuthPhase,
    page::{
        dashboard::DashboardPage, landing::LandingPage, not_found::NotFoundPage,
        privacy_policy::PrivacyPolicy, terms_of_service::TermsOfService,
    },
    prelude::*,
};
use futures_signals::signal::Signal;
use shared::frontend::route::{NotFoundReason, Route};
use std::fmt::Debug;

pub trait RouteExt {
    fn link_ext(&self) -> String;
    fn signal() -> impl Signal<Item = Route>;
    fn go_to_url(&self) {
        dominator::routing::go_to_url(&self.link_ext());
    }
    fn hard_redirect(&self) {
        let location = web_sys::window().unwrap_ext().location();
        let s: String = self.link_ext();
        location.set_href(&s).unwrap_ext();
    }
}

impl RouteExt for Route {
    fn link_ext(&self) -> String {
        self.link_url(&CONFIG.frontend_domain, &CONFIG.frontend_root_path)
    }
    fn signal() -> impl Signal<Item = Route> {
        dominator::routing::url()
            .signal_cloned()
            .map(|url| Route::from_url(&url, CONFIG.frontend_root_path))
    }
}

pub fn render() -> Dom {
    // just a local mapping, to avoid re-rendering the entire page
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum TopLevelRoute {
        Landing,
        Dashboard,
        NotFound,
        TermsOfService,
        PrivacyPolicy,
    }

    let top_level_route_sig = Route::signal()
        .map(|route| {
            let requires_auth = route.requires_auth();
            let top_level = match route {
                Route::Landing(_) => TopLevelRoute::Landing,
                Route::Dashboard(_) => TopLevelRoute::Dashboard,
                Route::NotFound(_) => TopLevelRoute::NotFound,
                Route::TermsOfService => TopLevelRoute::TermsOfService,
                Route::PrivacyPolicy => TopLevelRoute::PrivacyPolicy,
            };

            (top_level, requires_auth)
        })
        .dedupe();

    let top_level_sig = map_ref! {
        let route_and_requires_auth = top_level_route_sig,
        // this will cause AUTH to be evaluated on first load
        // and transition from Init to some other state
        let auth = AUTH.phase.signal_cloned(),
        // passed so page re-renders if language is changed
        let lang = LOCALE.current.signal_cloned()
        => {
            (route_and_requires_auth.clone(), auth.clone(), lang.clone())
        }
    };

    html!("div", {
        .style("width", "100%")
        .style("height", "100%")
        .child_signal(top_level_sig.map(|((route, requires_auth), auth, _)| {
            // Gate auth access
            match auth {
                AuthPhase::Init => {
                    // Auth is still loading for the first time, so don't render anything
                    return None;
                },
                // User is not logged in at all, so any page that requires auth is gated
                AuthPhase::Unauthenticated => {
                    // ofc if the page doesn't require auth, then it's accessible
                    if requires_auth {
                        Route::NotFound(NotFoundReason::NoAuth).go_to_url();
                        return None;
                    }
                },
                // User is logged in - but email has not been confirmed. Force them to verify for gated pages 
                AuthPhase::EmailNotVerified => {
                    if requires_auth {
                        Route::Landing(Landing::Auth(AuthRoute::VerifyEmailWaiting)).go_to_url();
                        return None;
                    }
                },
                // User is logged in, so all pages are accessible
                AuthPhase::Authenticated => {}
            }
            Some(match route {
                TopLevelRoute::Landing => LandingPage::new().render(),
                TopLevelRoute::Dashboard => DashboardPage::new().render(),
                TopLevelRoute::NotFound => NotFoundPage::new().render(),
                TopLevelRoute::TermsOfService => TermsOfService::new().render(),
                TopLevelRoute::PrivacyPolicy => PrivacyPolicy::new().render(),
            })
        }))
        .fragment(&Modal::render())
    })
}
