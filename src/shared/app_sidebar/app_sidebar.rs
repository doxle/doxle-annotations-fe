use dioxus::prelude::*;
use crate::state::{USER, load_user};

#[component]
pub fn AppSidebar(open: Signal<bool>) -> Element {
    let nav = navigator();
    
    // Load user if not already loaded - runs only once
    use_hook(|| {
        spawn(async move {
            if USER.read().is_none() {
                load_user().await;
            }
        });
    });
    
    // Mock stats - in production, fetch from API
    let earnings = "$1,234.56";
    let approved_count = 42;
    let reviewed_count = 18;
    let pending_count = 7;

    rsx! {
        div {
            class: if open() { "app-sidebar open" } else { "app-sidebar" },
            
            // Stats header
            div {
                class: "app-sidebar-header",
                h3 { "Statistics" }
            }
            
            // Stats section
            div {
                class: "app-sidebar-stats-section",
                
                div {
                    class: "app-sidebar-stat-item",
                    div {
                        class: "app-sidebar-stat-label",
                        "Earnings"
                    }
                    div {
                        class: "app-sidebar-stat-value",
                        "{earnings}"
                    }
                }
                
                div {
                    class: "app-sidebar-stat-item",
                    div {
                        class: "app-sidebar-stat-label",
                        "Approved"
                    }
                    div {
                        class: "app-sidebar-stat-value",
                        "{approved_count}"
                    }
                }
                
                div {
                    class: "app-sidebar-stat-item",
                    div {
                        class: "app-sidebar-stat-label",
                        "Reviewed"
                    }
                    div {
                        class: "app-sidebar-stat-value",
                        "{reviewed_count}"
                    }
                }
                
                div {
                    class: "app-sidebar-stat-item",
                    div {
                        class: "app-sidebar-stat-label",
                        "Pending"
                    }
                    div {
                        class: "app-sidebar-stat-value",
                        "{pending_count}"
                    }
                }
            }
            
            // Bottom section with logout and version
            div {
                class: "app-sidebar-bottom-section",
                
                // Logout button
                div {
                    class: "app-sidebar-logout",
                    onclick: move |_| {
                        // Clear auth token
                        crate::api::clear_token();
                        // Clear user state
                        crate::state::user_state::clear_user();
                        // Navigate to home
                        nav.push(crate::Route::HomePage {});
                    },
                    "Logout"
                }
                
                div {
                    class: "app-sidebar-version",
                    "v1.06"
                }
            }
        }
    }
}
