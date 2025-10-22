use super::image_utils::NAVBAR_H;
use super::dom_cache::{get_window, get_document};
use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// Sets up canvas size with device pixel ratio for crisp rendering
/// Measures the canvas container and sizes both saved and overlay canvases accordingly
pub fn setup_canvas_size() {
    if let Some(win) = get_window() {
        if let Some(document) = get_document() {
            // Measure the actual canvas container instead of window to avoid rounding/clipping on the right edge
            let (css_w, css_h) =
                if let Some(container) = document.get_element_by_id("canvas-container") {
                    let rect = container.get_bounding_client_rect();
                    (rect.width(), rect.height())
                } else {
                    // Fallback to window size if container not found
                    (
                        win.inner_width()
                            .ok()
                            .and_then(|w| w.as_f64())
                            .unwrap_or(1920.0),
                        win.inner_height()
                            .ok()
                            .and_then(|h| h.as_f64())
                            .map(|h| h - NAVBAR_H)
                            .unwrap_or(1080.0),
                    )
                };
            let dpr = win.device_pixel_ratio();

            // Size both canvases to match container precisely
            for canvas_id in ["canvas-saved", "canvas-overlay"] {
                if let Some(canvas_el) = document.get_element_by_id(canvas_id) {
                    if let Ok(canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() {
                        // Internal device buffer
                        canvas.set_width((css_w * dpr).round() as u32);
                        canvas.set_height((css_h * dpr).round() as u32);

                        // CSS display size
                        let _ = canvas
                            .style()
                            .set_property("width", &format!("{}px", css_w));
                        let _ = canvas
                            .style()
                            .set_property("height", &format!("{}px", css_h));

                        // // Scale context so drawing uses CSS pixels
                        // if let Ok(Some(ctx_any)) = canvas.get_context("2d") {
                        //     if let Ok(ctx) = ctx_any.dyn_into::<CanvasRenderingContext2d>() {
                        //         let _ = ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);
                        //     }
                        // }
                    }
                }
            }
        }
    }
}
