use dioxus::prelude::*;

const GEOMETRIC_GRID_JS: &str = include_str!("../../js/geometric-grid.js");

#[component]
pub fn GeometricGridPage() -> Element {
    // Inject animation script
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            if document.get_element_by_id("geometric-grid-script").is_none() {
                let script = document.create_element("script").unwrap();
                script.set_id("geometric-grid-script");
                script.set_attribute("type", "text/javascript").ok();
                script.set_text_content(Some(GEOMETRIC_GRID_JS));
                let _ = document.head().unwrap().append_child(&script);
            }
        }
    });

    rsx! {
        style {
            r"html, body {{ background: #000 !important; }}"
        }
        style {
            r".geometric-grid-page {{
                position: fixed;
                inset: 0;
                background: #000;
                display: flex;
                align-items: center;
                justify-content: center;
                overflow: hidden;
            }}
            #geometric-grid-container {{
                width: 100%;
                height: 100%;
                display: flex;
                align-items: center;
                justify-content: center;
            }}"
        }
        div {
            class: "geometric-grid-page",
            div {
                id: "geometric-grid-container",
            }
        }
    }
}
