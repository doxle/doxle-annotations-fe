use std::cell::RefCell;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::CanvasRenderingContext2d;

use super::edit_draw::draw_edit_overlay;
use super::polygon::{
    draw_endpoint, draw_point, draw_polygon, draw_preview_line, fill_polygon, POINT_RADIUS,
    ZOOMED_IN_RADIUS, ZOOMED_OUT_RADIUS,
};
use super::shapes::{Point, Polygon};
use crate::canvas::dom_cache::{get_document, get_overlay_context, get_window};

thread_local! {
    static RAF_PENDING: RefCell<bool> = RefCell::new(false);
    static LATEST_STATE: RefCell<Option<(Polygon, f64, f64, f64)>> = RefCell::new(None);
    static OVERLAY_CTX: RefCell<Option<CanvasRenderingContext2d>> = RefCell::new(None);
    static PREVIEW_POINT: RefCell<Option<Point>> = RefCell::new(None);
}

pub fn get_overlay_canvas_context() -> Option<CanvasRenderingContext2d> {
    // Use centralized dom_cache
    get_overlay_context()
}

fn clear_overlay(ctx: &CanvasRenderingContext2d) {
    if let Some(canvas) = ctx.canvas() {
        let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    }
}

fn render_overlay_once(poly: &Polygon, zoom: f64, pan_x: f64, pan_y: f64) {
    if let Some(ctx) = get_overlay_canvas_context() {
        clear_overlay(&ctx);
        // Get DPR and apply combined transform (DPR * zoom + pan)
        let win = get_window().expect("Should get window");
        let dpr = win.device_pixel_ratio();
        // Apply transform once - combines DPR scaling with zoom/pan
        let _ = ctx.set_transform(zoom * dpr, 0.0, 0.0, zoom * dpr, pan_x * dpr, pan_y * dpr);

        // draw polygon as in-progress overlay (in world coords)
        let points: Vec<Point> = poly.points.clone();

        if poly.is_closed {
            fill_polygon(&ctx, &points, zoom);
        } else {
            draw_polygon(&ctx, &points, false, zoom);
            let preview = PREVIEW_POINT.with(|p| *p.borrow());
            if let Some(preview) = preview {
                if let Some(last) = points.last() {
                    draw_preview_line(&ctx, *last, preview, zoom);
                    if points.len() >= 2 {
                        if let Some(first) = points.first() {
                            // Draw closing line only when cursor is near the first point (screen distance < 30px)
                            let dx = preview.x - first.x;
                            let dy = preview.y - first.y;
                            let distance = ((dx * dx + dy * dy).sqrt() * zoom);
                            if distance < 30.0 {
                                draw_preview_line(&ctx, preview, *first, zoom);
                            }
                        }
                    }
                }
            }
        }

        // Draw points (scale radius inversely to zoom since we're in world coords)
        let adjusted_radius =
            (POINT_RADIUS / zoom).clamp(ZOOMED_OUT_RADIUS / zoom, ZOOMED_IN_RADIUS / zoom);
        for (i, &p) in points.iter().enumerate() {
            // Check if near first point for closing
            let can_close = if i == 0 && points.len() >= 3 {
                if let Some(preview) = PREVIEW_POINT.with(|pp| *pp.borrow()) {
                    let dx = preview.x - p.x;
                    let dy = preview.y - p.y;
                    let distance = ((dx * dx + dy * dy).sqrt() * zoom); // Distance in screen pixels
                    distance < 30.0
                } else {
                    false
                }
            } else {
                false
            };

            if can_close {
                draw_endpoint(&ctx, p, adjusted_radius);
            } else {
                draw_point(&ctx, p, adjusted_radius);
            }
        }

        // Reset
        let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    }
}

pub fn schedule_overlay_redraw(poly: Polygon, zoom: f64, pan_x: f64, pan_y: f64) {
    // Store latest state - rAF will read this, not stale snapshot!
    LATEST_STATE.with(|state| {
        *state.borrow_mut() = Some((poly, zoom, pan_x, pan_y));
    });

    let should_schedule = RAF_PENDING.with(|flag| {
        let mut pending = flag.borrow_mut();
        if *pending {
            false // Already pending, will use latest state
        } else {
            *pending = true;
            true
        }
    });

    if !should_schedule {
        return; // rAF already scheduled, it will use latest state
    }

    let closure = Closure::wrap(Box::new(move || {
        // Read LATEST state at render time
        if let Some((poly, zoom, pan_x, pan_y)) = LATEST_STATE.with(|s| s.borrow().clone()) {
            render_overlay_once(&poly, zoom, pan_x, pan_y);
        }
        RAF_PENDING.with(|flag| *flag.borrow_mut() = false);
    }) as Box<dyn FnMut()>);

    if let Some(win) = get_window() {
        let _ = win.request_animation_frame(closure.as_ref().unchecked_ref());
    }
    closure.forget();
}

pub fn set_polygon_preview(point: Option<Point>) {
    PREVIEW_POINT.with(|p| *p.borrow_mut() = point);
}

pub fn clear_polygon_preview() {
    PREVIEW_POINT.with(|p| *p.borrow_mut() = None);
}

/// Schedule redraw for edit mode
pub fn schedule_edit_redraw(zoom: f64, pan_x: f64, pan_y: f64) {
    let should_schedule = RAF_PENDING.with(|flag| {
        let mut pending = flag.borrow_mut();
        if *pending {
            false
        } else {
            *pending = true;
            true
        }
    });

    if !should_schedule {
        return;
    }

    let closure = Closure::wrap(Box::new(move || {
        if let Some(ctx) = get_overlay_canvas_context() {
            // Clear canvas
            if let Some(canvas) = ctx.canvas() {
                let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
                ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
            }

            // Apply transform
            if let Some(win) = get_window() {
                let dpr = win.device_pixel_ratio();
                let _ =
                    ctx.set_transform(zoom * dpr, 0.0, 0.0, zoom * dpr, pan_x * dpr, pan_y * dpr);
            }

            // Check theme for dark mode
            let is_dark = if let Some(doc) = get_document() {
                if let Some(html) = doc.document_element() {
                    html.class_name().contains("dark")
                } else {
                    false
                }
            } else {
                false
            };

            // Draw edit overlay
            draw_edit_overlay(&ctx, zoom, is_dark);

            // Reset transform
            let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        }
        RAF_PENDING.with(|flag| *flag.borrow_mut() = false);
    }) as Box<dyn FnMut()>);

    if let Some(win) = get_window() {
        let _ = win.request_animation_frame(closure.as_ref().unchecked_ref());
    }
    closure.forget();
}
