use crate::Route;
use crate::api;
use dioxus::prelude::*;
use wasm_bindgen::JsCast;

const LOGO_LIGHT: Asset = asset!("/assets/icons/floorplan-light.svg");
const LOGO_DARK: Asset = asset!("/assets/icons/floorplan-dark.svg");
const RECTS_JS: &str = include_str!("home.js");
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

            if document.get_element_by_id("bg-rects-script").is_none() {
                let script = document.create_element("script").unwrap();
                script.set_id("bg-rects-script");
                script.set_attribute("type", "text/javascript").ok();
                script.set_text_content(Some(RECTS_JS));
                let _ = document.head().unwrap().append_child(&script);
            }
            
            // Always call init (handles navigation back)
            if let Ok(init_fn) = js_sys::Reflect::get(&window, &"__rectsInit".into()) {
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

            // Rects layer
            div {
                id: "rects-container",
                class: "home-rects-layer",
            }

            // Content layer
            div {
                class: "home-content",
                h1 {
                    class: "home-title",
                    "Building Intelligence."
                }
                p {
                    class: "home-description",
                    "We're teaching the computer to read plans so you don't have to have to babysit the paperwork. It's still learning like any good apprentice --and every plan you upload teaches it something new. Give it a crack, see what it can do, and help us build the future of building."
                }
button {
                    class: "upload-button",
onclick: move |_| { navigator().push(Route::SignInPage {}); },
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
