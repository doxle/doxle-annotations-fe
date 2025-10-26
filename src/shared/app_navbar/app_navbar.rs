use dioxus::prelude::*;
use crate::Route;
use crate::state::USER;

const NAVBAR_CSS: &str = include_str!("app_navbar.css");

#[derive(Clone, Copy, PartialEq)]
pub enum NavbarContext {
    Projects,
    Blocks { project_name: &'static str },
    Canvas { block_name: &'static str },
}

#[component]
pub fn AppNavbar(
    context: NavbarContext,
    sidebar_open: Signal<bool>,
) -> Element {
    let nav = navigator();
    let mut user_dropdown_open = use_signal(|| false);
    
    const SIDEBAR: Asset = asset!("/assets/icons/sidebar.svg");
    const LOGO: Asset = asset!("/assets/icons/dog-dark.svg");
    
    // Get user initial for avatar
    let user_initial = USER.read()
        .as_ref()
        .and_then(|u| u.name.chars().next())
        .unwrap_or('U')
        .to_uppercase()
        .to_string();
    
    rsx! {
        document::Style { {NAVBAR_CSS} }
        
        div {
            class: "app-navbar",
            onclick: move |_| {
                if user_dropdown_open() {
                    user_dropdown_open.set(false);
                }
            },
            
            // Left section - Logo and breadcrumbs
            div {
                class: "app-navbar-left",
                
                // Logo
                div {
                    class: "app-navbar-logo",
                    onclick: move |_| { nav.push(Route::ProjectsPage {}); },
                    img {
                        src: LOGO,
                        alt: "Doxle",
                        class: "app-navbar-logo-img"
                    }
                }
                
                // Breadcrumbs based on context
                div {
                    class: "app-navbar-breadcrumbs",
                    
                    match context {
                        NavbarContext::Projects => rsx! {
                            span { 
                                class: "app-navbar-breadcrumb current",
                                "Projects" 
                            }
                        },
                        NavbarContext::Blocks { project_name } => rsx! {
                            span {
                                class: "app-navbar-breadcrumb",
                                onclick: move |_| { nav.push(Route::ProjectsPage {}); },
                                "Projects"
                            }
                            span { class: "app-navbar-separator", " / " }
                            span { 
                                class: "app-navbar-breadcrumb current",
                                "{project_name}"
                            }
                        },
                        NavbarContext::Canvas { block_name } => rsx! {
                            span {
                                class: "app-navbar-breadcrumb",
                                onclick: move |_| { nav.push(Route::ProjectsPage {}); },
                                "Projects"
                            }
                            span { class: "app-navbar-separator", " / " }
                            span { 
                                class: "app-navbar-breadcrumb current",
                                "{block_name}"
                            }
                        }
                    }
                }
            }
            
            // Center section - Context specific tools
            div {
                class: "app-navbar-center",
                
                // Add context-specific tools here if needed
                // For example, canvas tools when in Canvas context
            }
            
            // Right section - User and sidebar toggle
            div {
                class: "app-navbar-right",
                
                // User avatar with dropdown
                div {
                    class: "app-navbar-user-avatar",
                    title: "User Menu",
                    onclick: move |e| {
                        e.stop_propagation();
                        user_dropdown_open.set(!user_dropdown_open());
                    },
                    "{user_initial}"
                    
                    // User dropdown menu
                    if user_dropdown_open() {
                        div {
                            class: "app-navbar-user-dropdown",
                            onclick: move |e| { e.stop_propagation(); },
                            
                            if let Some(user) = USER.read().as_ref() {
                                div { class: "user-dropdown-name", "{user.name}" }
                                div { class: "user-dropdown-email", "{user.email}" }
                                div { class: "user-dropdown-divider" }
                            }
                            
                            button {
                                class: "user-dropdown-item",
                                onclick: move |_| {
                                    nav.push(Route::ProjectsPage {});
                                    user_dropdown_open.set(false);
                                },
                                "Projects"
                            }
                            
                            button {
                                class: "user-dropdown-item",
                                onclick: move |_| {
                                    crate::api::clear_token();
                                    nav.push(Route::LoginPage {});
                                },
                                "Sign Out"
                            }
                        }
                    }
                }
                
                // Sidebar toggle
                button {
                    class: "app-navbar-sidebar-toggle",
                    title: if sidebar_open() { "Close Sidebar" } else { "Open Sidebar" },
                    onclick: move |_| {
                        sidebar_open.set(!sidebar_open());
                    },
                    img {
                        class: "app-navbar-sidebar-icon",
                        src: "{SIDEBAR}",
                        alt: "Toggle Sidebar"
                    }
                }
            }
        }
    }
}