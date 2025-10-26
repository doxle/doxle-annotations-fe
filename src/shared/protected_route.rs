use dioxus::prelude::*;
use crate::Route;
use crate::api;

#[component]
pub fn ProtectedRoute(children: Element) -> Element {
    let nav = navigator();
    
    // Check for token on mount
    use_hook(|| {
        let token = api::get_token();
        
        if token.is_none() {
            // No token - redirect to login
            tracing::info!("No auth token found - redirecting to login");
            nav.push(Route::LoginPage {});
        } else {
            tracing::info!("Auth token found - user is authenticated");
        }
    });
    
    // If we have a token, render the protected content
    if api::get_token().is_some() {
        rsx! { {children} }
    } else {
        // Redirect happening, show nothing
        rsx! { div {} }
    }
}
