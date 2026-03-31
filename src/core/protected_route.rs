use dioxus::prelude::*;
use crate::Route;
use crate::core::client::{self, ApiError};
use crate::users::api::User;
use crate::users::state::USER;

#[component]
pub fn ProtectedRoute(children: Element) -> Element {
    let nav = navigator();
    let has_user = USER.read().is_some();
    let mut auth_checked = use_signal(move || has_user);
    let mut authorized = use_signal(move || has_user);

    // Validate session on mount
    use_hook(move || {
        if USER.read().is_some() {
            auth_checked.set(true);
            authorized.set(true);
            return;
        }
        let nav = nav.clone();
        let mut auth_checked = auth_checked.clone();
        let mut authorized = authorized.clone();

        spawn(async move {
            match client::get_typed::<User>("/users/me").await {
                Ok(_user) => {
                    authorized.set(true);
                }
                Err(ApiError::Unauthorized(_)) => {
                    tracing::info!("No valid session - redirecting to sign-in");
                    nav.push(Route::SignInPage {});
                }
                Err(ApiError::Other(e)) => {
                    // Network/server error — don't sign out, just allow through
                    tracing::warn!("Auth check failed (non-401): {} — allowing through", e);
                    authorized.set(true);
                }
            }
            auth_checked.set(true);
        });
    });

    if !auth_checked() {
        return rsx! {
            div {
                style: "min-height: 100vh; background: var(--bg-primary);"
            }
        };
    }

    if authorized() {
        rsx! { {children} }
    } else {
        rsx! { div {} }
    }
}



