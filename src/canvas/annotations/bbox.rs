use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::shared::shapes::{BBox, Point};
use crate::canvas::dom_cache::get_window;

/* ---- styles (same as polygon) ---- */
const POINT_STROKE: &str = "rgb(51, 66, 255)";
const POINT_FILL: &str = "rgba(51, 66, 255, 0.4)";
const POINT_RADIUS: f64 = 18.0;
const ZOOMED_IN_RADIUS: f64 = 9.0;
const ZOOMED_OUT_RADIUS: f64 = 3.0;
const LINE_COLOR: &str = "rgb(51, 66, 255)";
const LINE_WIDTH: f64 = 1.5;
const FILL_COLOR: &str = "rgba(51, 66, 255, 0.5)";
const PREVIEW_LINE_COLOR: &str = "rgba(51, 66, 255, 1)";
const PREVIEW_FILL_COLOR: &str = "rgba(51, 66, 255, 0.3)";

fn clear_canvas(ctx: &CanvasRenderingContext2d) {
    if let Some(canvas) = ctx.canvas() {
        // Clear entire device buffer regardless of current transform
        let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    }
}

// Draw a point with 40% opacity fill
fn draw_point(ctx: &CanvasRenderingContext2d, p: Point, radius: f64) {
    ctx.set_fill_style(&JsValue::from_str(POINT_FILL));
    ctx.set_stroke_style(&JsValue::from_str(POINT_STROKE));
    ctx.set_line_width(2.0);

    ctx.begin_path();
    let _ = ctx.arc(p.x, p.y, radius, 0.0, std::f64::consts::PI * 2.0);
    ctx.fill();
    ctx.stroke();
}

// Draw bounding box outline
fn draw_bbox_outline(ctx: &CanvasRenderingContext2d, start: Point, end: Point, is_preview: bool) {
    let x = start.x.min(end.x);
    let y = start.y.min(end.y);
    let width = (end.x - start.x).abs();
    let height = (end.y - start.y).abs();

    if is_preview {
        ctx.set_stroke_style(&JsValue::from_str(PREVIEW_LINE_COLOR));
    } else {
        ctx.set_stroke_style(&JsValue::from_str(LINE_COLOR));
    }
    ctx.set_line_width(LINE_WIDTH);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");

    ctx.begin_path();
    ctx.rect(x, y, width, height);
    ctx.stroke();
}

// Draw filled bounding box
fn fill_bbox(ctx: &CanvasRenderingContext2d, start: Point, end: Point, is_preview: bool) {
    let x = start.x.min(end.x);
    let y = start.y.min(end.y);
    let width = (end.x - start.x).abs();
    let height = (end.y - start.y).abs();

    // Set fill style
    if is_preview {
        ctx.set_fill_style(&JsValue::from_str(PREVIEW_FILL_COLOR));
        ctx.set_stroke_style(&JsValue::from_str(PREVIEW_LINE_COLOR));
    } else {
        ctx.set_fill_style(&JsValue::from_str(FILL_COLOR));
        ctx.set_stroke_style(&JsValue::from_str(LINE_COLOR));
    }
    ctx.set_line_width(LINE_WIDTH);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");

    ctx.begin_path();
    ctx.rect(x, y, width, height);
    ctx.fill();
    ctx.stroke();
}

pub fn redraw_bbox(ctx: &CanvasRenderingContext2d, bbox: &BBox, zoom: f64, pan_x: f64, pan_y: f64) {
    clear_canvas(ctx);

    // Apply combined DPR*zoom transform once
    if let Some(win) = get_window() {
        let dpr = win.device_pixel_ratio();
        let _ = ctx.set_transform(zoom * dpr, 0.0, 0.0, zoom * dpr, pan_x * dpr, pan_y * dpr);
    }

    // Draw in WORLD coordinates; transform handles screen mapping
    if let Some(start) = bbox.start_point {
        // Adjust radius to keep roughly constant on screen
        let radius = (POINT_RADIUS / zoom).clamp(ZOOMED_OUT_RADIUS / zoom, ZOOMED_IN_RADIUS / zoom);

        // Draw start point
        draw_point(ctx, start, radius);

        if bbox.is_complete {
            if let Some(end) = bbox.end_point {
                fill_bbox(ctx, start, end, false);
                draw_point(ctx, end, radius);
            }
        }
        // Note: Preview is now handled via mouse_handlers + thread_local
    }

    // Reset transform to identity to avoid leaking state
    let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
}

// Handle bbox click
pub fn on_bbox_click(
    world_x: f64,
    world_y: f64,
    bbox: &mut BBox,
    ctx: &CanvasRenderingContext2d,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) {
    tracing::info!("BBox clicked world x: {} and y: {}", world_x, world_y);

    if bbox.start_point.is_none() {
        // First click - set start point
        bbox.set_start(Point {
            x: world_x,
            y: world_y,
        });
    } else if !bbox.is_complete {
        // Second click - set end point and complete
        bbox.set_end(Point {
            x: world_x,
            y: world_y,
        });
    } else {
        // Box is complete, start a new one
        bbox.reset();
        bbox.set_start(Point {
            x: world_x,
            y: world_y,
        });
    }

    // Immediate visual feedback on the overlay canvas
    redraw_bbox(ctx, bbox, zoom, pan_x, pan_y);
}
