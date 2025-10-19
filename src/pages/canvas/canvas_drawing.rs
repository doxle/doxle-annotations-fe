use super::annotations::AnnotationTool;
use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};

#[component]
pub fn CanvasDrawing(
    zoom: Signal<f64>,
    pan_x: Signal<f64>,
    pan_y: Signal<f64>,
    selected_tool: Signal<Option<AnnotationTool>>,
) -> Element {
    // --- Assets ----
    const CANVAS_IMG: Asset = asset!("/assets/images/test.png");

    let mut canvas_ready = use_signal(|| false);

    // Set canvas internal resolution to match the container size
    use_effect(move || {
        if let Some(window) = window() {
            if let Some(document) = window.document() {
                if let Some(canvas_el) = document.get_element_by_id("canvas-annotations") {
                    if let Ok(canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() {
                        // Get the actual viewport dimensions
                        let css_w: f64 = window
                            .inner_width()
                            .ok()
                            .and_then(|w| w.as_f64())
                            .unwrap_or(1920.0);
                        let css_h = window
                            .inner_height()
                            .ok()
                            .and_then(|h| h.as_f64())
                            .map(|h| h - 36.0) // Subtract navbar height
                            .unwrap_or(1080.0);

                        //get window device pixel ratio
                        let dpr: f64 = window.device_pixel_ratio();

                        let bw: u32 = (css_w * dpr).round() as u32;
                        let bh: u32 = (css_h * dpr).round() as u32;

                        canvas.set_width((css_w * dpr).round() as u32);
                        canvas.set_height((css_h * dpr).round() as u32);

                        //keep visuals size in css pixels
                        let _ = canvas
                            .style()
                            .set_property("width", &format!("{}px", css_w));
                        let _ = canvas
                            .style()
                            .set_property("height", &format!("{}px", css_h));

                        // Scale context so your existing screen coords still work
                        if let Ok(Some(ctx_any)) = canvas.get_context("2d") {
                            if let Ok(ctx) = ctx_any.dyn_into::<CanvasRenderingContext2d>() {
                                // reset then scale (prevents compounding on hot-reloads)
                                let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
                                let _ = ctx.scale(dpr, dpr);
                            }
                        }
                    }
                }
            }
        }
    });

    rsx! {
        // Layer 1: Background Image - GPU-accelerated (starts at 60% width, centered)
        div {
            class: "canvas-layer canvas-image",
            img { src: CANVAS_IMG }
        }

        // Layer 2: Annotations - CPU - Scales with world
        canvas {
            id: "canvas-annotations",
            class: if selected_tool().is_some() {
                "canvas-layer canvas-annotations tool-active"
            } else {
                "canvas-layer canvas-annotations"
            },
            style: "background-color: transparent;",
        }
    }
}
