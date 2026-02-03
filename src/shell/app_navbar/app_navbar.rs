use dioxus::prelude::*;
use crate::shell::{THEME, Theme};
use crate::Route;
use crate::atoms::tasks::state::TASKS;
use crate::atoms::media::Image;
use crate::users::state::{USER, USER_LOADING, load_user};
use super::status_bar::StatusBar;
use crate::api;
use crate::blocks::dashboard::state::state_load_blocks;


// const D_FLAG2: Asset = asset!("/assets/icons/d-flag2.svg");
const LOGO_LIGHT: Asset = asset!("/assets/icons/dx-walker-navbar-light.svg");
const LOGO_DARK: Asset = asset!("/assets/icons/dx-walker-navbar-dark.svg");
const CHEVRON_LEFT: Asset = asset!("/assets/icons/chevron-left.svg");
const CHEVRON_RIGHT: Asset = asset!("/assets/icons/chevron-right.svg");

#[component]
pub fn AppNavbar(children:Element) -> Element {
    let route = use_route::<Route>();
    let nav = use_navigator();
    let is_dark = THEME() == Theme::Dark;
    let mut logo_menu_open = use_signal(|| false);

    // use_hook(||{
    //     if api::get_access_token().is_some() // token exists
    //     && USER.read().is_none() // User is none
    //     && !*USER_LOADING.read() // User is not currently loading - prevents starting multiple user calls
    //     {
    //         spawn(async move {
    //             load_user().await;
    //         });
    //     }
    // });
    
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
                // User name - only clickable/underlined when not on dashboard
                if let Some(user) = &*USER.read() {
                    if block_name.is_some() {
                        // Not on dashboard - show clickable username
                        span {
                            class: "app-navbar-username clickable",
                            onclick: move |_| {
                                spawn(async move {
                                    state_load_blocks().await;
                                });
                                nav.push(Route::DashboardPage {});
                            },
                            "@{user.user_name}"
                        }
                    } else {
                        // On dashboard - no underline
                        span {
                            class: "app-navbar-username",
                            "@{user.user_name}"
                        }
                    }
                }
                // Breadcrumb
                // Block name - clickable to go back to dashboard (to load all blocks)
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
                    if block_name.is_some() || USER.read().is_some() {
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

        }
    }
}

