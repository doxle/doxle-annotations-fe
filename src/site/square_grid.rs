use dioxus::prelude::*;

const SQUARE_GRID_JS: &str = include_str!("../../js/square-grid.js");

#[component]
pub fn SquareGridPage() -> Element {
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            if document.get_element_by_id("square-grid-script").is_none() {
                let script = document.create_element("script").unwrap();
                script.set_id("square-grid-script");
                script.set_attribute("type", "text/javascript").ok();
                script.set_text_content(Some(SQUARE_GRID_JS));
                let _ = document.head().unwrap().append_child(&script);
            }
        }
    });

    rsx! {
        style {
            r"html, body {{ background: #000 !important; }}"
        }
        style {
            r".square {{
                position: absolute;
                border: 1px solid rgba(255, 255, 255, 0.15);
                transform: translate(-50%, -50%);
                will-change: transform;
                backface-visibility: hidden;
                pointer-events: none;
                transition: transform 0.2s ease-out;
            }}
            .square-grid-page {{
                position: fixed;
                inset: 0;
                background: #000;
                overflow: hidden;
            }}
            #square-grid-container {{
                position: absolute;
                top: 0;
                left: 0;
                width: 100%;
                height: 100%;
                pointer-events: auto;
                contain: layout paint style;
                transform: translateZ(0);
            }}"
        }
        div {
            class: "square-grid-page",
            div {
                id: "square-grid-container",
            }
        }
    }
}
