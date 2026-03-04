use crate::Route;
use crate::api;
use dioxus::prelude::*;
use wasm_bindgen::JsCast;

const LOGO_LIGHT: Asset = asset!("/assets/icons/floorplan-light.svg");
const LOGO_DARK: Asset = asset!("/assets/icons/floorplan-dark.svg");
const DOTS_JS: &str = include_str!("home.js");
const TYPEWRITER_JS: &str = include_str!("../../js/typewriter.js");
const HOME_CSS: &str = include_str!("home.css");


#[component]
pub fn HomePage() -> Element {
    let nav = navigator();
    
    // Track route to trigger effect on navigation
    let route = use_route::<Route>();
    
    // Inject animation scripts
    use_effect(move || {
        // Read route to create dependency
        let _ = &route;
        
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            // If invite code was stashed by App, redirect to signup
            if let Ok(Some(storage)) = window.session_storage() {
                if let Ok(Some(code)) = storage.get_item("invite_code") {
                    if !code.trim().is_empty() {
                        nav.push(Route::SignupPage {});
                        return;
                    }
                }
            }

            // Reset scroll position (mobile keyboard can leave page scrolled)
            window.scroll_to_with_x_and_y(0.0, 0.0);

            if document.get_element_by_id("bg-dots-script").is_none() {
                let script = document.create_element("script").unwrap();
                script.set_id("bg-dots-script");
                script.set_attribute("type", "text/javascript").ok();
                script.set_text_content(Some(DOTS_JS));
                let _ = document.head().unwrap().append_child(&script);
            }
            
            // Always call init (handles navigation back)
            if let Ok(init_fn) = js_sys::Reflect::get(&window, &"__dotsInit".into()) {
                if let Ok(func) = init_fn.dyn_into::<js_sys::Function>() {
                    let _ = func.call0(&window);
                }
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
        style { {HOME_CSS} }
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
                    span {
                        class: "home-title-line",
                        "Building"
                    }
                    span {
                        class: "home-title-line",
                        "Intelligence."
                    }
                }
                p {
                    class: "home-description",
                    "We're teaching the computer to read plans so you don't have to babysit the paperwork. It's still learning like any good apprentice --and every plan you upload teaches it something new. Give it a crack, see what it can do, and help us build the future of building."
                }
                button {
                    class: "upload-button",
                    onclick: move |_| { nav.push(Route::SignInPage {}); },
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
