use dioxus::prelude::*;
use crate::core::svg_canvas::dot_overlay::DotOverlay;

const CSS: &str = include_str!("building_canvas.css");

/// Stripped-down canvas for building block.
/// Figma-style: scroll = pan, Cmd/Ctrl+scroll = zoom, Space+drag = pan.
/// No annotations, comments, bbox, labels, auto-save.
#[component]
pub fn BuildingCanvas(
    #[props(default)] image_url: Option<String>,
    children: Element,
) -> Element {
    let mut pan_x = use_signal(|| 0.0_f64);
    let mut pan_y = use_signal(|| 0.0_f64);
    let mut zoom = use_signal(|| 1.0_f64);
    let mut image_size = use_signal(|| (0.0_f64, 0.0_f64));
    let mut space_held = use_signal(|| false);
    let mut is_panning = use_signal(|| false);
    let mut last_mouse = use_signal(|| (0.0_f64, 0.0_f64));

    // Track Space key for space+drag panning
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::prelude::*;
            use wasm_bindgen::JsCast;

            let window = web_sys::window().unwrap();

            let keydown = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
                if e.code() == "Space" && !e.repeat() {
                    e.prevent_default();
                    space_held.set(true);
                }
            });

            let keyup = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
                if e.code() == "Space" {
                    space_held.set(false);
                    is_panning.set(false);
                }
            });

            window.add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref()).ok();
            window.add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref()).ok();

            keydown.forget();
            keyup.forget();
        }
    });

    // Dot overlay spacing
    let dot_spacing = 30.0;
    let dot_min_px = 15.0;
    let dot_world = {
        let z = zoom();
        if dot_spacing * z < dot_min_px { dot_min_px / z } else { dot_spacing }
    };

    let has_image = image_url.is_some();
    let (iw, ih) = image_size();

    rsx! {
        style { {CSS} }
        div {
            class: "building-canvas",

            // Wheel: scroll=pan, Cmd/Ctrl+scroll=zoom
            onwheel: move |evt| {
                let raw = evt.data();
                let dx = raw.delta().strip_units().x;
                let dy = raw.delta().strip_units().y;
                let modifiers = raw.modifiers();
                let has_meta = modifiers.contains(dioxus::prelude::Modifiers::META)
                    || modifiers.contains(dioxus::prelude::Modifiers::CONTROL);

                if has_meta {
                    // Zoom at cursor
                    let factor = if dy < 0.0 { 1.08 } else { 1.0 / 1.08 };
                    let coords = raw.element_coordinates();
                    let cx = coords.x;
                    let cy = coords.y;
                    let old_zoom = zoom();
                    let new_zoom = (old_zoom * factor).clamp(0.1, 10.0);
                    pan_x.set(cx - (cx - pan_x()) * (new_zoom / old_zoom));
                    pan_y.set(cy - (cy - pan_y()) * (new_zoom / old_zoom));
                    zoom.set(new_zoom);
                } else {
                    // Pan
                    pan_x -= dx;
                    pan_y -= dy;
                }
            },

            // Space+drag panning
            onmousedown: move |evt| {
                if space_held() {
                    let coords = evt.element_coordinates();
                    last_mouse.set((coords.x, coords.y));
                    is_panning.set(true);
                }
            },
            onmousemove: move |evt| {
                if is_panning() {
                    let coords = evt.element_coordinates();
                    let (lx, ly) = last_mouse();
                    pan_x += coords.x - lx;
                    pan_y += coords.y - ly;
                    last_mouse.set((coords.x, coords.y));
                }
            },
            onmouseup: move |_| {
                is_panning.set(false);
            },
            onmouseleave: move |_| {
                is_panning.set(false);
            },

            // Cursor style
            style: if space_held() { "cursor: grab;" } else if is_panning() { "cursor: grabbing;" } else { "" },

            // Dot overlay
            DotOverlay {
                pan_x: pan_x(),
                pan_y: pan_y(),
                zoom: zoom(),
                step_world: dot_world,
                opacity: 0.4,
                image_width: iw,
                image_height: ih,
            }

            // SVG layer (image + future drawing tools)
            if has_image {
                svg {
                    class: "building-canvas-svg",
                    xmlns: "http://www.w3.org/2000/svg",

                    // Pan/zoom transform group
                    g {
                        transform: "translate({pan_x()},{pan_y()}) scale({zoom()})",

                        image {
                            href: "{image_url.as_deref().unwrap_or_default()}",
                            x: "0",
                            y: "0",
                            width: "{iw}",
                            height: "{ih}",
preserve_aspect_ratio: "xMidYMid meet",
                            onload: move |_| {
                                // Image size detection happens via JS interop
                                // For now we set a default if not already set
                                if iw == 0.0 {
                                    image_size.set((1200.0, 900.0));
                                }
                            },
                        }
                    }
                }
            }

            // Children overlay (wizard steps, side panels, etc.)
            {children}
        }
    }
}
