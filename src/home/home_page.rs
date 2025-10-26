use crate::Route;
use crate::api;
use dioxus::prelude::*;

const LOGO_LIGHT: Asset = asset!("/assets/icons/floorplan-light.svg");
const LOGO_DARK: Asset = asset!("/assets/icons/floorplan-dark.svg");
const DOTS_JS: &str = include_str!("../../js/dot-animation.js");
const TYPEWRITER_JS: &str = include_str!("../../js/typewriter.js");


#[component]
pub fn HomePage() -> Element {
    let nav = navigator();
    
    // Redirect to projects if already logged in
    use_hook(|| {
        if api::get_token().is_some() {
            tracing::info!("User already logged in - redirecting to projects");
            nav.push(Route::ProjectsPage {});
        }
    });
    
    // Inject animation scripts
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            if document.get_element_by_id("bg-dots-script").is_none() {
                let script = document.create_element("script").unwrap();
                script.set_id("bg-dots-script");
                script.set_attribute("type", "text/javascript").ok();
                script.set_text_content(Some(DOTS_JS));
                let _ = document.head().unwrap().append_child(&script);
            }

            if document.get_element_by_id("typewriter-script").is_none() {
                let script = document.create_element("script").unwrap();
                script.set_id("typewriter-script");
                script.set_attribute("type", "text/javascript").ok();
                script.set_text_content(Some(TYPEWRITER_JS));
                let _ = document.head().unwrap().append_child(&script);
            }
        }
    });

    rsx! {
        // Force light background on the root to prevent tab-switch flicker
        document::Style {
            r"html, body {{ background: #ffffff !important; background-color: #ffffff !important; color-scheme: light !important; }}"
        }
        document::Style {
            r".dot {{
                position: absolute;
                width: 3px;
                height: 3px;
                background-color: rgba(149, 128, 255, 0.3);
                border-radius: 50%;
                will-change: transform;
                transform: translate3d(0,0,0);
                pointer-events: none;
                backface-visibility: hidden;
            }}"
        }
        div {
            class: "home-container",

            // Dots layer
            div {
                id: "dots-container",
                class: "home-dots-layer",
            }

            // Content layer
            div {
                class: "home-content",
                h1 {
                    class: "home-title",
                    "BUILT WITH AI"
                }
                h2 {
                    class: "home-title",
                    "SO YOU "
                    span { "BUILD" }
                }
                h2 {
                    class: "home-subtitle",
                    "WITH AI."
                }
                p {
                    class: "home-description",
                    "We're teaching the computer to read plans so you don't have to have to babysit the paperwork. It's still learning like any good apprentice --and every plan you upload teaches it something new. Give it a crack, see what it can do, and help us build the future of building."
                }
                button {
                    class: "home-button",
                    onclick: move |_| { navigator().push(Route::LoginPage {}); },
                    span {
                        class: "home-button-text",
                        "Upload Plans"
                    }
                    img {
                        class: "upload-logo upload-logo-light",
                        src: "{LOGO_LIGHT}",
                        alt: "Floorplan Logo",
                    }
                    img {
                        class: "upload-logo upload-logo-dark",
                        src: "{LOGO_DARK}",
                        alt: "Floorplan Logo",
                    }
                }
            }
        }
    }
}
