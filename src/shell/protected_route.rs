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
            let token = api::get_access_token();
            if let Some(t) = token.as_deref() {
                // Access token is expired
                if api::is_access_token_expired(t) {
                     match api::refresh_access_token().await{
                        Ok(_) => {
                            tracing::info!("✅ Token refreshed successfully");
                        }
                        Err(_)=>{
                            tracing::info!("Refresh failed - redirecting to sign-in");
                            api::clear_access_token();
                            nav.push(Route::SignInPage {});
                        }
                     }
                }        
            }
            else {
                // No access token: try refresh using refresh token
                match api::refresh_access_token().await{
                    Ok(_) => {
                        tracing::info!("✅ Token refreshed successfully");
                    }
                    Err(_)=>{
                        tracing::info!("Refresh failed - redirecting to sign-in");
                        api::clear_access_token();
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


    // If we have a valid token, render the protected content
    let token = api::get_access_token();
    if let Some(t) = token.as_deref() {
        if !api::is_access_token_expired(t) {
            rsx! { {children} }
        } else {
            rsx! { div {} }
        }
    } else {
        rsx! { div {} }
    }
}



