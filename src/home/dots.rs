use dioxus::prelude::*;

const DOTS_JS: &str = include_str!("../../js/dot-animation.js");

#[component]
pub fn DotsPage() -> Element {
    // Inject animation script
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
        }
    });

    rsx! {
        style {
            r"html, body {{ background: #000 !important; }}"
        }
        style {
            r".dot {{
                position: absolute;
                width: 3px;
                height: 3px;
background-color: rgba(255, 255, 255, 0.15);
                border-radius: 50%;
                will-change: transform;
                transform: translate3d(0,0,0);
                pointer-events: none;
                backface-visibility: hidden;
            }}"
        }
        div {
            class: "dots-page",
            div {
                id: "dots-container",
                class: "dots-layer",
            }
        }
    }
}
