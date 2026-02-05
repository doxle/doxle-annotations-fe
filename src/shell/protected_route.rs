use dioxus::prelude::*;
use crate::Route;
use crate::api;

#[component]
pub fn ProtectedRoute(children: Element) -> Element {
    let nav = navigator();
    let mut auth_checked = use_signal(|| false);
    let mut authorized = use_signal(|| false);

    // Validate session on mount
    use_hook(move || {
        let nav = nav.clone();
        let mut auth_checked = auth_checked.clone();
        let mut authorized = authorized.clone();

        spawn(async move {
            match api::get_current_user().await {
                Ok(_user) => {
                    authorized.set(true);
                }
                Err(_) => {
                    tracing::info!("No valid session - redirecting to sign-in");
                    nav.push(Route::SignInPage {});
                }
            }
            auth_checked.set(true);
        });
    });

    if !auth_checked() {
        return rsx! { div {} };
    }

    if authorized() {
        rsx! { {children} }
    } else {
        rsx! { div {} }
    }
}



