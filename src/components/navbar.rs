use crate::Route;
use dioxus::prelude::*;
const LOGO: Asset = asset!("/assets/dog-light.svg");
const NAVBAR_CSS: &str = include_str!("navbar.css");

#[component]
pub fn Navbar() -> Element {
    let mut menu_open = use_signal(|| false);
    let nav = navigator();
    let route = use_route::<Route>();
    let is_ourstory = matches!(route, Route::OurStoryPage {});
    let is_join = matches!(route, Route::JoinPage {});
    let is_signin = matches!(route, Route::SigninPage {});

    rsx! {
        document::Style { {NAVBAR_CSS} }

        nav {
            style: "position: absolute; top: 0; left: 0; right: 0; z-index: 1000; display: flex; justify-content: space-between; align-items: center; padding: 20px 40px; background: transparent; background-color: transparent;",

            // Left side - Logo
            div {
                style: "display: flex; align-items: center; cursor: pointer;",
                onclick: move |_| { nav.push(Route::HomePage {}); },
                img {
                    class: "logo-img",
                    src: LOGO,
                    alt: "Doxle Logo",
                    style: "height: 30px;"
                }
            }

            // Right side container
            div {
                style: "display: flex; align-items: center; gap: 20px;",

                // Desktop navigation buttons
                div {
                    class: "nav-buttons",
                    style: "display: flex; gap: 30px; align-items: center;",

                    button {
                        class: if is_ourstory { "nav-btn active" } else { "nav-btn" },
                        style: "background: none; font-weight:300 !important; font-size:16px; border: none; font-family: Helvetica, Arial, sans-serif; cursor: pointer; color: #333; transition: opacity 0.2s;",
                        onclick: move |_| { nav.push(Route::OurStoryPage {}); },
                        "Our Story"
                    }

                    button {
                        class: if is_join { "nav-btn active" } else { "nav-btn" },
                        style: "background: none; font-weight:300 !important; font-size:16px; border: none; font-family: Helvetica, Arial, sans-serif; cursor: pointer; color: #333; transition: opacity 0.2s;",
                        onclick: move |_| { nav.push(Route::JoinPage {}); },
                        "Join"
                    }

                    button {
                        class: if is_signin { "nav-btn active" } else { "nav-btn" },
                        style: "background: none; font-weight:300 !important; font-size:16px; border: none; font-family: Helvetica, Arial, sans-serif; cursor: pointer; color: #333; transition: opacity 0.2s;",
                        onclick: move |_| { nav.push(Route::SigninPage {}); },
                        "Sign In"
                    }

                    button {
                        class: "say-hello-btn",
                        style: "background: #4F5BF8; font-weight:300 !important; width:140px; height:60px; font-size:16px; border: none; font-family: Helvetica, Arial, sans-serif; cursor: pointer; color: white; padding: 10px 24px; border-radius: 0px; transition: opacity 0.2s;",
                        onclick: move |_| { nav.push(Route::SayHelloPage {}); },
                        "Say Hello"
                    }
                }

                // Mobile hamburger button
                button {
                    class: "mobile-menu-btn",
                    style: "display: none; background: none; border: none; cursor: pointer; padding: 8px;",
                    onclick: move |_| menu_open.set(!menu_open()),
                    // Hamburger icon
                    div {
                        style: "display: flex; flex-direction: column; gap: 5px;",
                        div { style: "width: 25px; height: 2px; background-color: #333; border-radius: 2px;" }
                        div { style: "width: 25px; height: 2px; background-color: #333; border-radius: 2px;" }
                        div { style: "width: 25px; height: 2px; background-color: #333; border-radius: 2px;" }
                    }
                }
            }
        }

        // Mobile menu
        if menu_open() {
            div {
                class: "mobile-menu",
                style: "position: fixed; top: 70px; left: 0; right: 0; background: white; box-shadow: 0 4px 6px rgba(0,0,0,0.1); z-index: 999; padding: 20px;",
                onclick: move |_| menu_open.set(false),

                div {
                    style: "display: flex; flex-direction: column; gap: 20px;",

                    button {
                        class: "mobile-menu-item",
                        style: "background: none; border: none; font-family: Helvetica, Arial, sans-serif; font-weight: 300; font-size: 18px; cursor: pointer; color: #333; text-align: left; padding: 16px 10px;",
                        onclick: move |_| { menu_open.set(false); nav.push(Route::OurStoryPage {}); },
                        "Our Story"
                    }

                    button {
                        class: "mobile-menu-item",
                        style: "background: none; border: none; font-family: Helvetica, Arial, sans-serif; font-weight: 300; font-size: 18px; cursor: pointer; color: #333; text-align: left; padding: 16px 10px;",
                        onclick: move |_| { menu_open.set(false); nav.push(Route::JoinPage {}); },
                        "Join"
                    }

                    button {
                        class: "mobile-menu-item",
                        style: "background: none; border: none; font-family: Helvetica, Arial, sans-serif; font-weight: 300; font-size: 18px; cursor: pointer; color: #333; text-align: left; padding: 16px 10px;",
                        onclick: move |_| { menu_open.set(false); nav.push(Route::SigninPage {}); },
                        "Sign In"
                    }

                    button {
                        class: "mobile-menu-item say-hello",
                        style: "background: #4F5BF8; border: none; font-family: Helvetica, Arial, sans-serif; font-weight: 300; font-size: 18px; cursor: pointer; color: white; padding: 12px 24px; border-radius: 0px; width: 100%;",
                        onclick: move |_| { menu_open.set(false); nav.push(Route::SayHelloPage {}); },
                        "Say Hello"
                    }
                }
            }
        }
    }
}
