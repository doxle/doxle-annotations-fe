use crate::Route;
use dioxus::prelude::*;
// const LOGO: Asset = asset!("/assets/icons/d-flag1.svg");
const LOGO: Asset = asset!("/assets/icons/dx-walker-dark.svg");
const LOGO_HOVER: Asset = asset!("/assets/icons/dx-walker-dark.svg");
const NAVBAR_CSS: &str = include_str!("home_navbar.css");

#[component]
pub fn Navbar() -> Element {
    let mut menu_open = use_signal(|| false);
    let nav = navigator();
    let route = use_route::<Route>();
    let is_ourstory = matches!(route, Route::OurStoryPage {});
    let is_vision = matches!(route, Route::VisionPage {});
let is_signin = matches!(route, Route::SignInPage {});

    rsx! {
        style { {NAVBAR_CSS} }

        nav {
            class: "home-navbar",

            // Left side - Logo
            div {
                class: "home-navbar-logo",
                onclick: move |_| { nav.push(Route::HomePage {}); },
                img {
                    class: "home-logo-img home-logo-default",
                    src: LOGO,
                    alt: "Doxle Logo",
                }
                img {
                    class: "home-logo-img home-logo-hover",
                    src: LOGO_HOVER,
                    alt: "Doxle Logo Hover",
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
                        class: if is_vision { "home-nav-btn home-nav-btn--red active" } else { "home-nav-btn home-nav-btn--red" },
                        onclick: move |_| { nav.push(Route::VisionPage {}); },
                        "Vision"
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

        // Mobile menu
        if menu_open() {
            div {
                class: "home-mobile-menu",
                onclick: move |_| menu_open.set(false),

                div {
                    class: "home-mobile-menu-items",

                    button {
                        class: "home-mobile-menu-item",
                        onclick: move |_| { menu_open.set(false); nav.push(Route::OurStoryPage {}); },
                        "Our Story"
                    }

                    button {
                        class: "home-mobile-menu-item",
                        onclick: move |_| { menu_open.set(false); nav.push(Route::VisionPage {}); },
                        "Vision"
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
