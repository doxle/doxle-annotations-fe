use dioxus::prelude::*;
use wasm_bindgen::JsCast;

const OURSTORY_CSS: &str = include_str!("ourstory.css");
const OURSTORY_JS: &str = include_str!("ourstory.js");

#[component]
pub fn OurStoryPage() -> Element {
    // Inject animation script
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            // Always refresh script so changes apply reliably on mobile/HMR
            if let Some(existing) = document.get_element_by_id("ourstory-blocks-script") {
                existing.remove();
            }

            let script = document.create_element("script").unwrap();
            script.set_id("ourstory-blocks-script");
            script.set_attribute("type", "text/javascript").ok();
            script.set_text_content(Some(OURSTORY_JS));
            let _ = document.head().unwrap().append_child(&script);
            
            // Call init
            if let Ok(init_fn) = js_sys::Reflect::get(&window, &"__ourstoryInit".into()) {
                if let Ok(func) = init_fn.dyn_into::<js_sys::Function>() {
                    let _ = func.call0(&window);
                }
            }
        }
    });

    rsx! {
        style { {OURSTORY_CSS} }
        div { 
            class: "ourstory-container",
            
            // Blocks animation layer (JS - desktop only)
            div {
                id: "ourstory-blocks-container",
                class: "ourstory-blocks-layer",
            }
            
            div { 
                class: "ourstory-content",
                h1 { 
                    class: "ourstory-title",
                    "Our Story" 
                }
                p {
                    class: "ourstory-text",
                    "Welcome to Doxle. Building is hard enough — so we make working with plans less painful. We start with measurements, and we're building toward 3D and estimating — no hype, no magic buttons. Curious or want to know more? Email us at help@doxle.com"
                }
            }
            
            // Mobile CSS-only blocks (after text)
            div {
                class: "ourstory-mobile-blocks",
                // Column 1 - 2 blocks
                div { class: "ourstory-mobile-block-col",
                    div { class: "ourstory-mobile-block" }
                    div { class: "ourstory-mobile-block" }
                }
                // Column 2 - 3 blocks
                div { class: "ourstory-mobile-block-col",
                    div { class: "ourstory-mobile-block" }
                    div { class: "ourstory-mobile-block" }
                    div { class: "ourstory-mobile-block" }
                }
                // Column 3 - 1 block
                div { class: "ourstory-mobile-block-col",
                    div { class: "ourstory-mobile-block" }
                }
                // Column 4 - 2 blocks
                div { class: "ourstory-mobile-block-col",
                    div { class: "ourstory-mobile-block" }
                    div { class: "ourstory-mobile-block" }
                }
                // Column 5 - 3 blocks
                div { class: "ourstory-mobile-block-col",
                    div { class: "ourstory-mobile-block" }
                    div { class: "ourstory-mobile-block" }
                    div { class: "ourstory-mobile-block" }
                }
            }
        }
    }
}
