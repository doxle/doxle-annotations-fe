use dioxus::prelude::*;
use crate::Route;
use crate::api::cognito;

#[component]
pub fn ProjectsPage() -> Element {
    let nav = navigator();

    let handle_logout = move |_| {
        // Clear the auth token
        cognito::clear_token();
        // Redirect to signin
        nav.push(Route::SigninPage {});
    };

    rsx! {
        div {
            style: "min-height: 100vh; padding: 40px 20px; font-family: Helvetica, Arial, sans-serif; background: white;",
            
            // Header with user avatar
            div {
                style: "max-width: 1200px; margin: 0 auto 32px auto; display: flex; justify-content: flex-end;",
                
                // User avatar button
                div {
                    onclick: handle_logout,
                    style: "width: 48px; height: 48px; border-radius: 50%; background: #3342FF; display: flex; align-items: center; justify-content: center; cursor: pointer; transition: opacity 0.2s; color: white; font-size: 18px; font-weight: 500; font-family: Helvetica, Arial, sans-serif;",
                    title: "Click to logout",
                    "S"
                }
            }
            
            div {
                style: "max-width: 1200px; margin: 0 auto;",
                
                h1 {
                    style: "font-size: 32px; font-weight: 500; color: #3342FF; margin-bottom: 32px;",
                    "Projects"
                }
                
                div {
                    style: "padding: 40px; border: 2px dashed #DDD; text-align: center; color: #666;",
                    
                    p {
                        style: "font-size: 18px; margin-bottom: 16px;",
                        "No projects yet"
                    }
                    
                    p {
                        style: "font-size: 14px; color: #999;",
                        "Projects will appear here once created"
                    }
                }
            }
        }
    }
}
