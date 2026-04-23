use crate::Route;
use crate::core::{Theme, THEME};
use dioxus::prelude::*;
const LOGO_LIGHT: Asset = asset!("/assets/icons/dog-light.svg");
const LOGO_DARK: Asset = asset!("/assets/icons/dog-dark.svg");
const NAVBAR_CSS: &str = include_str!("home_navbar.css");

#[cfg(target_arch = "wasm32")]
fn blur_active_element_and_scroll_to_top() {
    use wasm_bindgen::JsCast;

    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(active) = document.active_element() {
                if let Ok(el) = active.dyn_into::<web_sys::HtmlElement>() {
                    let _ = el.blur();
                }
            }
        }

        window.scroll_to_with_x_and_y(0.0, 0.0);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn blur_active_element_and_scroll_to_top() {}

#[component]
pub fn Navbar() -> Element {
    let mut menu_open = use_signal(|| false);
    let nav = navigator();
    let route = use_route::<Route>();
    let is_ourstory = matches!(route, Route::OurStoryPage {});
    let is_vision = matches!(route, Route::VisionPage {});
    let is_signin = matches!(route, Route::SignInPage {});

    // Pages with light bg need black logo (LOGO_LIGHT), dark bg pages need white logo (LOGO_DARK)
    let has_light_bg = matches!(route, Route::LegalConsentPage {} | Route::UploadPlansPage {} | Route::CollectEmailPage {});
    let logo = if has_light_bg && THEME() != Theme::Dark {
        LOGO_LIGHT  // black fill for light bg
    } else {
        LOGO_DARK   // white fill for dark bg
    };

    rsx! {
        style { {NAVBAR_CSS} }

        nav {
            class: "home-navbar",

            // Left side - Logo
            div {
                class: "home-navbar-logo",
                onclick: move |_| {
                    blur_active_element_and_scroll_to_top();
                    nav.push(Route::HomePage {});
                },
                img {
                    class: "home-logo-img",
                    src: logo,
                    alt: "Doxle Logo",
                }
            }

            // Right side container
            div {
                class: "home-navbar-right",

                // Desktop navigation buttons
                div {
                    class: "home-nav-buttons",

                    button {
                        class: if is_ourstory { "home-nav-btn home-nav-btn--red active" } else { "home-nav-btn home-nav-btn--red" },
                        onclick: move |_| { nav.push(Route::OurStoryPage {}); },
                        "Our Story"
                    }

                    button {
                        class: if is_signin { "home-nav-btn home-nav-btn--red active" } else { "home-nav-btn home-nav-btn--red" },
                        onclick: move |_| { nav.push(Route::SignInPage {}); },
                        "Sign In"
                    }

                    button {
                        class: "home-say-hello-btn",
                        onclick: move |_| { nav.push(Route::SayHelloPage {}); },
                        "Say Hello"
                    }
                }

                // Mobile hamburger button
                button {
                    class: "home-mobile-menu-btn",
                    onclick: move |_| menu_open.set(!menu_open()),
                    // Hamburger icon
                    div {
                        class: "home-hamburger",
                        div { class: "home-hamburger-line" }
                        div { class: "home-hamburger-line" }
                        div { class: "home-hamburger-line" }
                    }
                }
            }
        }

        // Mobile menu backdrop (click to close)
        if menu_open() {
            div {
                class: "home-mobile-menu-backdrop",
                onclick: move |_| menu_open.set(false),
            }
        }

        // Mobile menu
        if menu_open() {
            div {
                class: "home-mobile-menu",

                div {
                    class: "home-mobile-menu-items",

                    button {
                        class: "home-mobile-menu-item",
                        onclick: move |_| { menu_open.set(false); nav.push(Route::OurStoryPage {}); },
                        "Our Story"
                    }

                    button {
                        class: "home-mobile-menu-item",
                        onclick: move |_| { menu_open.set(false); nav.push(Route::SignInPage {}); },
                        "Sign In"
                    }

                    button {
                        class: "home-mobile-menu-item home-say-hello",
                        onclick: move |_| { menu_open.set(false); nav.push(Route::SayHelloPage {}); },
                        "Say Hello"
                    }
                }
            }
        }
    }
}
