use dioxus::prelude::*;
use crate::Route;
use crate::api;
use crate::state::{USER, load_user};

#[derive(Clone, Debug, PartialEq)]
pub enum AppSidebarPage {
    Projects,
    Blocks { project_id: String, project_name: String },
    Canvas,
}

#[component]
pub fn AppSidebar(current_page: AppSidebarPage) -> Element {
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

    let handle_logout = move |_| {
        api::clear_token();
        nav.push(Route::LoginPage {});
    };

    rsx! {
        div {
            class: "app-sidebar",
            
            // User profile section
            div {
                class: "app-sidebar-user-section",
                
                div {
                    class: "app-sidebar-user-profile",
                    
                    if let Some(u) = USER.read().as_ref() {
                        div {
                            style: "display: flex; flex-direction: column; gap: 4px;",
                            
                            div {
                                style: "display: flex; align-items: center; gap: 8px;",
                                
                                div {
                                    class: "app-sidebar-user-avatar",
                                    onclick: handle_logout,
                                    title: "Click to logout",
                                    {u.name.chars().next().unwrap_or('U').to_uppercase().to_string()}
                                }
                                
                                div {
                                    class: "app-sidebar-user-name",
                                    "{u.name}"
                                }
                            }
                            
                            div {
                                class: "app-sidebar-user-email",
                                "{u.email}"
                            }
                        }
                    }
                }
            }
            
            // Divider
            div {
                class: "app-sidebar-divider"
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
            
            // Divider
            div {
                class: "app-sidebar-divider"
            }
            
            // Navigation section
            div {
                class: "app-sidebar-nav-section",
                
                // Show current project if in blocks page
                if let AppSidebarPage::Blocks { project_id, project_name } = &current_page {
                    div {
                        class: "app-sidebar-nav-item active sub-item",
                        "📁 {project_name}"
                    }
                }
            }
            
            // Bottom section with version or additional info
            div {
                class: "app-sidebar-bottom-section",
                
                div {
                    class: "app-sidebar-version",
                    "v1.0.0"
                }
            }
        }
    }
}