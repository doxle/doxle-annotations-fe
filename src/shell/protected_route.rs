use dioxus::prelude::*;
use crate::Route;
use crate::api;


#[component]
pub fn ProtectedRoute(children: Element) -> Element {
    let nav = navigator();
    let mut auth_checked = use_signal(|| false);
    
    // Check for token on mount
    use_hook(move || {
        let nav = nav.clone();
        let mut auth_checked = auth_checked.clone();

        spawn(async move {
            // With httpOnly cookies, we can only check if access_token cookie exists
            // The browser sends cookies automatically with requests
            let has_token = api::get_access_token().is_some();
            
            if !has_token {
                // No access token cookie - try to refresh (refresh token is httpOnly)
                match api::refresh_access_token().await {
                    Ok(_) => {
                        tracing::info!("✅ Token refreshed successfully");
                    }
                    Err(_) => {
                        tracing::info!("No valid session - redirecting to sign-in");
                        nav.push(Route::SignInPage {});
                    }
                }
            }
            auth_checked.set(true);
        });
    });

    if !auth_checked() {
        return rsx! { div {} };
    }

    // If we have a token cookie, render the protected content
    if api::get_access_token().is_some() {
        rsx! { {children} }
    } else {
        rsx! { div {} }
    }
}



