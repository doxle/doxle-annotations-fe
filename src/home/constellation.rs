use dioxus::prelude::*;

const CONSTELLATION_JS: &str = include_str!("../../js/constellation.js");

#[component]
pub fn ConstellationPage() -> Element {
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            if document.get_element_by_id("constellation-script").is_none() {
                let script = document.create_element("script").unwrap();
                script.set_id("constellation-script");
                script.set_attribute("type", "text/javascript").ok();
                script.set_text_content(Some(CONSTELLATION_JS));
                let _ = document.head().unwrap().append_child(&script);
            }
        }
    });

    rsx! {
        style {
            r"html, body {{ background: #000 !important; }}"
        }
        style {
            r".constellation-page {{
                position: fixed;
                inset: 0;
                background: #000;
                overflow: hidden;
            }}
            #constellation-container {{
                position: absolute;
                inset: 0;
                width: 100%;
                height: 100%;
            }}"
        }
        div {
            class: "constellation-page",
            div {
                id: "constellation-container",
            }
        }
    }
}
