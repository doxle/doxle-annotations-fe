use crate::Route;
use dioxus::prelude::*;
const LOGO_LIGHT: Asset = asset!("/assets/icons/dog-light.svg");
const LOGO_DARK: Asset = asset!("/assets/icons/dog-dark.svg");
const NAVBAR_CSS: &str = include_str!("home_navbar.css");

#[component]
pub fn Navbar() -> Element {
    let mut menu_open = use_signal(|| false);
    let nav = navigator();
    let route = use_route::<Route>();
    let is_ourstory = matches!(route, Route::OurStoryPage {});
    let is_vision = matches!(route, Route::VisionPage {});
    let is_signin = matches!(route, Route::LoginPage {});

    rsx! {
        document::Style { {NAVBAR_CSS} }

        nav {
            class: "navbar",

            // Left side - Logo
            div {
                class: "navbar-logo",
                onclick: move |_| { nav.push(Route::HomePage {}); },
                img {
                    class: "logo-img logo-light",
                    src: LOGO_LIGHT,
                    alt: "Doxle Logo",
                }
                img {
                    class: "logo-img logo-dark",
                    src: LOGO_DARK,
                    alt: "Doxle Logo",
                }
            }

            // Right side container
            div {
                class: "navbar-right",

                // Desktop navigation buttons
                div {
                    class: "nav-buttons",

                    button {
                        class: if is_ourstory { "nav-btn active" } else { "nav-btn" },
                        onclick: move |_| { nav.push(Route::OurStoryPage {}); },
                        "Our Story"
                    }

                    button {
                        class: if is_vision { "nav-btn active" } else { "nav-btn" },
                        onclick: move |_| { nav.push(Route::VisionPage {}); },
                        "Vision"
                    }

                    button {
                        class: if is_signin { "nav-btn active" } else { "nav-btn" },
                        onclick: move |_| { nav.push(Route::LoginPage {}); },
                        "Sign In"
                    }

                    button {
                        class: "say-hello-btn",
                        onclick: move |_| { nav.push(Route::SayHelloPage {}); },
                        "Say Hello"
                    }
                }

                // Mobile hamburger button
                button {
                    class: "mobile-menu-btn",
                    onclick: move |_| menu_open.set(!menu_open()),
                    // Hamburger icon
                    div {
                        class: "hamburger",
                        div { class: "hamburger-line" }
                        div { class: "hamburger-line" }
                        div { class: "hamburger-line" }
                    }
                }
            }
        }

        // Mobile menu
        if menu_open() {
            div {
                class: "mobile-menu",
                onclick: move |_| menu_open.set(false),

                div {
                    class: "mobile-menu-items",

                    button {
                        class: "mobile-menu-item",
                        onclick: move |_| { menu_open.set(false); nav.push(Route::OurStoryPage {}); },
                        "Our Story"
                    }

                    button {
                        class: "mobile-menu-item",
                        onclick: move |_| { menu_open.set(false); nav.push(Route::VisionPage {}); },
                        "Vision"
                    }

                    button {
                        class: "mobile-menu-item",
                        onclick: move |_| { menu_open.set(false); nav.push(Route::LoginPage {}); },
                        "Sign In"
                    }

                    button {
                        class: "mobile-menu-item say-hello",
                        onclick: move |_| { menu_open.set(false); nav.push(Route::SayHelloPage {}); },
                        "Say Hello"
                    }
                }
            }
        }
    }
}
