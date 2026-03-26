use dioxus::prelude::*;
use dioxus::dioxus_core::use_drop;
use super::bindings;
use super::wall::Wall;

const VIEWER3D_JS: &str = include_str!("../../js/viewer3d.js");

const THREE_CDN: &str = "https://esm.sh/three@0.170.0";
const ORBIT_CDN: &str = "https://esm.sh/three@0.170.0/examples/jsm/controls/OrbitControls.js";

/// Inject a <script type="importmap"> so Three.js + OrbitControls resolve as bare imports.
/// Then inject viewer3d.js which uses window.THREE.
fn inject_scripts() {
    #[cfg(target_arch = "wasm32")]
    {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };
        let document = match window.document() {
            Some(d) => d,
            None => return,
        };

        // Skip if already injected
        if document.get_element_by_id("viewer3d-three-loader").is_some() {
            return;
        }

        // 1. Loader script: dynamically import Three.js + OrbitControls from CDN,
        //    attach to window.THREE so our IIFE viewer3d.js can use them.
        let loader_src = format!(
            r#"
            (async function() {{
                const THREE = await import("{}");
                const {{ OrbitControls }} = await import("{}");
                window.THREE = THREE;
                window.DoxleOrbitControls = OrbitControls;
                window.dispatchEvent(new Event("three-ready"));
            }})();
            "#,
            THREE_CDN, ORBIT_CDN
        );

        let loader = document.create_element("script").unwrap();
        loader.set_id("viewer3d-three-loader");
        loader.set_attribute("type", "module").ok();
        loader.set_text_content(Some(&loader_src));
        let _ = document.head().unwrap().append_child(&loader);

        // 2. Inject viewer3d.js (IIFE, registers window.doxleViewer)
        let viewer_script = document.create_element("script").unwrap();
        viewer_script.set_id("viewer3d-script");
        viewer_script.set_attribute("type", "text/javascript").ok();
        viewer_script.set_text_content(Some(VIEWER3D_JS));
        let _ = document.head().unwrap().append_child(&viewer_script);
    }
}

/// Wait for "three-ready" event, then init viewer and add demo wall.
fn init_after_three_loaded() {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };

        // Check if THREE is already loaded
        let three_exists = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("THREE"))
            .map(|v| !v.is_undefined())
            .unwrap_or(false);

        if three_exists {
            setup_demo_wall();
            return;
        }

        // Otherwise wait for the event
        let closure = Closure::once_into_js(move || {
            setup_demo_wall();
        });

        let _ = window.add_event_listener_with_callback(
            "three-ready",
            closure.unchecked_ref(),
        );
    }
}

fn setup_demo_wall() {
    bindings::viewer_init("viewer3d-container");

    // Generate a 6m x 4m room with 2.4m walls, centered at origin
    let json = super::wall::generate_room(6.0, 4.0, 2.4);
    bindings::viewer_add_wall(&json);
}

#[component]
pub fn ViewerPage() -> Element {
    use_effect(move || {
        inject_scripts();
        init_after_three_loaded();
    });

    // Cleanup on unmount
    use_drop(move || {
        bindings::viewer_dispose();
    });

    rsx! {
        div {
            class: "viewer3d-page",

            div {
                id: "viewer3d-container",
            }

            div {
                class: "viewer3d-overlay",
                h3 { "Room Prototype" }
                p { "6000mm × 4000mm × 2400mm" }
                p { "4 walls · studs @ 450mm · nogging · window" }
                p { "Orbit: drag | Zoom: scroll | Pan: right-drag" }
                p { "Tap two points to measure" }
                p {
                    id: "doxle-measure-label",
                    style: "color: #00ff00; font-size: 16px; font-weight: bold; margin-top: 6px;",
                }
            }
        }
    }
}
