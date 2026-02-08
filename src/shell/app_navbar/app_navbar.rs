use dioxus::prelude::*;
use crate::shell::{THEME, Theme};
use crate::shell::app_sidebar::SidebarTab;
use crate::Route;
use crate::atoms::tasks::state::TASKS;
use crate::atoms::media::Image;
use crate::users::state::{USER, load_user};
use super::status_bar::StatusBar;
use crate::api;
use crate::blocks::dashboard::state::state_load_blocks;

// Account Panel Component - opens when clicking avatar
#[component]
fn AccountPanel(show: Signal<bool>, user_name: String, user_email: String) -> Element {
    let nav = use_navigator();
    let mut email = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut success_message = use_signal(|| Option::<String>::None);

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        let email_value = email();
        
        if email_value.trim().is_empty() {
            error_message.set(Some("Please enter an email address".to_string()));
            return;
        }

        spawn(async move {
            is_loading.set(true);
            error_message.set(None);
            success_message.set(None);

            match api::create_invite(&email_value).await {
                Ok(_) => {
                    success_message.set(Some(format!("Invite sent to {}", email_value)));
                    email.set(String::new());
                }
                Err(e) => {
                    error_message.set(Some(e));
                }
            }
            is_loading.set(false);
        });
    };

    let close_panel = move |_| {
        show.set(false);
        email.set(String::new());
        error_message.set(None);
        success_message.set(None);
    };

    rsx! {
        div {
            class: "account-panel-overlay",
            onmousedown: close_panel,
            
            div {
                class: "account-panel",
                onmousedown: move |e| e.stop_propagation(),
                
                // Close button
                button {
                    class: "account-panel-close",
                    onclick: close_panel,
                    "×"
                }
                
                // Invite section
                div {
                    class: "account-panel-section",
                    h3 { class: "account-panel-title", "Invite" }
                    
                    form {
                        onsubmit: handle_submit,
                        class: "account-panel-invite-form",
                        
                        input {
                            class: "account-panel-input",
                            r#type: "email",
                            placeholder: "Enter email address",
                            required: true,
                            value: "{email}",
                            disabled: is_loading(),
                            oninput: move |e| email.set(e.value())
                        }
                        
                        button {
                            class: "account-panel-send-btn",
                            r#type: "submit",
                            disabled: is_loading(),
                            if is_loading() { "..." } else { "Send" }
                        }
                    }
                    
                    if let Some(error) = error_message() {
                        div { class: "account-panel-error", "{error}" }
                    }
                    
                    if let Some(success) = success_message() {
                        div { class: "account-panel-success", "{success}" }
                    }
                }
                
                // Bottom bar with user info and logout
                div {
                    class: "account-panel-bottom",
                    
                    // User info (left)
                    div {
                        class: "account-panel-user",
                        span { class: "account-panel-user-name", "{user_name}" }
                        span { class: "account-panel-user-email", "{user_email}" }
                    }
                    
                    // Logout (right)
                    button {
                        class: "account-panel-logout",
                        onclick: move |_| {
                            spawn(async {
                                let _ = api::logout().await;
                            });
                            nav.push(Route::SignInPage {});
                        },
                        "Sign Out"
                    }
                }
            }
        }
    }
}


// const D_FLAG2: Asset = asset!("/assets/icons/d-flag2.svg");
const LOGO_LIGHT: Asset = asset!("/assets/icons/dog-light.svg");
const LOGO_DARK: Asset = asset!("/assets/icons/dog-dark.svg");
const CHEVRON_LEFT: Asset = asset!("/assets/icons/chevron-left.svg");
const CHEVRON_RIGHT: Asset = asset!("/assets/icons/chevron-right.svg");

#[component]
pub fn AppNavbar(
    children: Element,
    #[props(default)] sidebar_tab: Option<Signal<SidebarTab>>,
    #[props(default)] sidebar_open: Option<Signal<bool>>,
) -> Element {
    let route = use_route::<Route>();
    let nav = use_navigator();
    let is_dark = THEME() == Theme::Dark;
    let mut logo_menu_open = use_signal(|| false);
    let mut show_account_panel = use_signal(|| false);

    // Load user once when AppNavbar mounts (no signal reads = runs once, won't loop)
    use_effect(|| {
        spawn(async move {
            load_user().await;
        });
    });

    // Extract data from route
    let (block_id, block_name, task_id, task_name, image_name, prev_img, next_img, current_idx, total_imgs): (
        Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<Image>, Option<Image>, usize, usize
    ) = match &route {
        Route::TasksListPage { block_id, block_name } => {
            (Some(block_id.clone()), Some(block_name.clone()), None, None, None, None, None, 0, 0)
        }
        Route::AnnotationCanvasPage { block_id, block_name, task_id, task_name, image_id, image_name } => {
            let mut prev_img: Option<Image> = None;
            let mut next_img: Option<Image> = None;
            let mut current_idx: usize = 0;
            let mut total_imgs: usize = 0;
            
            if let Some(task) = TASKS.read().iter().find(|t| t.task_id == *task_id) {
                let mut images = task.images.clone();
                images.sort_by_key(|img| img.order.unwrap_or(i32::MAX));
                total_imgs = images.len();
                let mut found = false;
                for (idx, img) in images.iter().enumerate() {
                    if found { next_img = Some(img.clone()); break; }
                    if img.image_id == *image_id { found = true; current_idx = idx + 1; }
                    else { prev_img = Some(img.clone()); }
                }
            }
            (Some(block_id.clone()), Some(block_name.clone()), Some(task_id.clone()), Some(task_name.clone()), Some(image_name.clone()), prev_img, next_img, current_idx, total_imgs)
        }
        _ => (None, None, None, None, None, None, None, 0, 0)
    };
    
    // Clone for use in closures
    let bid = block_id.clone().unwrap_or_default();
    let bname = block_name.clone().unwrap_or_default();
    let tid = task_id.clone().unwrap_or_default();
    let tname = task_name.clone().unwrap_or_default();
    
    // if let Some(user) = &*crate::users::state::USER.read() {
    //     tracing::info!(" USER {:?}", user);
    // };
    rsx!{
        nav{
            class:"app-navbar",
            style: "position: relative;", // Ensure status bar centers relative to navbar
             // Home icon (left) - click to show dropdown
             div {
                 class: "app-navbar-logo-container",
                 img {
                     key: "home-{is_dark}",
                     src: if is_dark { LOGO_DARK } else { LOGO_LIGHT },
                     class: "app-navbar-logo",
                     alt: "Home",
                     onclick: move |_| { logo_menu_open.set(!logo_menu_open()); }
                 }
                 if logo_menu_open() {
                     div {
                         class: "logo-menu-overlay",
                         onclick: move |_| { logo_menu_open.set(false); }
                     }
                     div {
                         class: "logo-menu-dropdown",
                         onclick: move |e| e.stop_propagation(),
                        div {
                            class: "logo-menu-item",
                            onclick: move |_| {
                                spawn(async move {
                                    state_load_blocks().await;
                                });
                                nav.push(Route::DashboardPage {});
                                logo_menu_open.set(false);
                            },
                            "Dashboard"
                        }
                         div {
                             class: "logo-menu-item",
                             onclick: move |_| {
                                 logo_menu_open.set(false);
                             },
                             "Tasks"
                         }
                         div { class: "logo-menu-divider" }
                         div {
                             class: "logo-menu-item",
                             onclick: move |_| {
                                 // Toggle theme
                                 let new_theme = if THEME() == Theme::Dark { Theme::Light } else { Theme::Dark };
                                 *THEME.write() = new_theme;
                                 logo_menu_open.set(false);
                             },
                             "Theme"
                         }
                         div {
                             class: "logo-menu-item",
                             onclick: move |_| {
                                 logo_menu_open.set(false);
                             },
                             "Settings"
                         }
                         div { class: "logo-menu-divider" }
                         div {
                             class: "logo-menu-item",
                             onclick: move |_| {
                                 logo_menu_open.set(false);
                             },
                             "Help"
                         }
                        div {
                             class: "logo-menu-item",
                             onclick: move |_| {
                                 spawn(async {
                                     let _ = api::logout().await;
                                 });
                                 nav.push(Route::SignInPage {});
                                 logo_menu_open.set(false);
                             },
                             "Sign Out"
                         }
                     }
                 }
             }

             div {
                class: "app-navbar-left-section",
                // Breadcrumb - Block name clickable to go back to dashboard
                if let Some(name) = &block_name {
                    div { 
                        class: "app-breadcrumb-item clickable",
                        onclick: move |_| {
                            spawn(async move {
                                state_load_blocks().await;
                            });
                            nav.push(Route::DashboardPage {});
                        },
                        "{name}"
                    }
                }

                // Task name
                if let Some(name) = &task_name {
                    // Add slash separator
                    if block_name.is_some() {
                        span {
                            class: "app-breadcrumb-chevron",
                            "/"
                        }
                    }
                    if image_name.is_some() {
                        // When viewing image - make task name clickable to go back to task list
                        {
                            let bid = bid.clone();
                            let bname = bname.clone();
                            rsx! {
                                div { 
                                    class: "app-breadcrumb-item clickable",
                                    onclick: move |_| {
                                    nav.push(Route::TasksListPage { block_id: bid.clone(), block_name: bname.clone() });
                                    },
                                    "{name}" 
                                }
                            }
                        }
                    } else {
                        div { class: "app-breadcrumb-item current", "{name}" }
                    }
                }

                // Image name
                if let Some(img_name) = &image_name {
                    span {
                        class: "app-breadcrumb-chevron",
                        "/"
                    }
                    span { 
                        class: "app-breadcrumb-item current truncate", 
                        "data-tooltip" : "{img_name}",
                         span {
                            class: "app-breadcrumb-text",
                            "{img_name}"
                        }
                    }
                }
            }

           
            
             // Center content (custom or image navigation or default StatusBar)
            if prev_img.is_some() || next_img.is_some() {
                div {
                    class: "app-navbar-image-nav",
                    // Image counter (format: 02/09)
                    if total_imgs > 0 {
                        span {
                            class: "app-navbar-image-counter",
                            "{current_idx:02}/{total_imgs:02}"
                        }
                    }
                    // Left arrow
                    if let Some(img) = prev_img.clone() {
                        {
                            let bid = bid.clone();
                            let bname = bname.clone();
                            let tid = tid.clone();
                            let tname = tname.clone();
                            let img_id = img.image_id.clone();
                            let img_name = img.url.split('/').last().unwrap_or("").to_string();
                            rsx! {
                                img {
                                    src: CHEVRON_LEFT,
                                    class: "app-nav-chevron",
                                    onclick: move |_| {
                                        nav.push(Route::AnnotationCanvasPage {
                                            block_id: bid.clone(),
                                            block_name: bname.clone(),
                                            task_id: tid.clone(),
                                            task_name: tname.clone(),
                                            image_id: img_id.clone(),
                                            image_name: img_name.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    // Right arrow
                    if let Some(img) = next_img.clone() {
                        {
                            let bid = bid.clone();
                            let bname = bname.clone();
                            let tid = tid.clone();
                            let tname = tname.clone();
                            let img_id = img.image_id.clone();
                            let img_name = img.url.split('/').last().unwrap_or("").to_string();
                            rsx! {
                                img {
                                    src: CHEVRON_RIGHT,
                                    class: "app-nav-chevron",
                                    onclick: move |_| {
                                        nav.push(Route::AnnotationCanvasPage {
                                            block_id: bid.clone(),
                                            block_name: bname.clone(),
                                            task_id: tid.clone(),
                                            task_name: tname.clone(),
                                            image_id: img_id.clone(),
                                            image_name: img_name.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    // Pipe separator
                    span { class: "app-navbar-pipe", "|" }
                }
            } else if children.is_ok() {
                {children}
            } else {
                StatusBar {}
            }

            // Sidebar tabs (only when sidebar is open)
            if let (Some(mut tab_signal), Some(open_signal)) = (sidebar_tab, sidebar_open) {
                if open_signal() {
                    div {
                        class: "navbar-sidebar-tabs",
                        div {
                            class: if tab_signal() == SidebarTab::Labels { "navbar-tab active" } else { "navbar-tab" },
                            onclick: move |_| tab_signal.set(SidebarTab::Labels),
                            "Labels"
                        }
                        div {
                            class: if tab_signal() == SidebarTab::Comments { "navbar-tab active" } else { "navbar-tab" },
                            onclick: move |_| tab_signal.set(SidebarTab::Comments),
                            "Comments"
                        }
                    }
                }
            }

            // Right section - User avatar
            if let Some(user) = &*USER.read() {
                {
                    // Get initials from user name
                    let initials: String = user.user_name
                        .split_whitespace()
                        .filter_map(|word| word.chars().next())
                        .take(2)
                        .collect::<String>()
                        .to_uppercase();
                    let initials = if initials.is_empty() { 
                        user.user_email.chars().next().unwrap_or('U').to_uppercase().to_string() 
                    } else { 
                        initials 
                    };
                    let user_name = user.user_name.clone();
                    let user_email = user.user_email.clone();
                    rsx! {
                        div {
                            class: "app-navbar-right-section",
                            div {
                                class: "user-avatar",
                                onclick: move |_| show_account_panel.set(true),
                                "{initials}"
                            }
                        }
                        
                        // Account Panel
                        if show_account_panel() {
                            AccountPanel { 
                                show: show_account_panel,
                                user_name: user_name,
                                user_email: user_email
                            }
                        }
                    }
                }
            }
        }
    }
}

