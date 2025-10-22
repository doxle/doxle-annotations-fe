use super::annotations::comment::Comment;
use super::annotations::overlay_canvas::{clear_polygon_preview, get_overlay_canvas_context};
use super::annotations::shapes::BBox;
use super::annotations::shapes::Polygon;
use super::annotations::Tool;
use super::dom_cache::get_window;
use dioxus::prelude::*;
use web_sys::window;

/// Clears the overlay canvas and resets annotation state when switching tools
pub fn clear_annotations_on_tool_switch(
    selected_tool: Tool,
    mut polygon: Signal<Polygon>,
    mut bbox: Signal<BBox>,
    mut comment: Signal<Comment>,
) {
    if let Some(ctx) = get_overlay_canvas_context() {
        // Reset preview state to avoid any dangling preview lines
        clear_polygon_preview();
        // Clear canvas
        if let Some(canvas) = ctx.canvas() {
            let window = get_window().expect("Should get window");
            let dpr = window.device_pixel_ratio();
            let css_w = canvas.width() as f64 / dpr;
            let css_h = canvas.height() as f64 / dpr;
            ctx.clear_rect(0.0, 0.0, css_w, css_h);
        }

        // When switching tools, clear the previous tool's annotation
        // TODO: Save to database before clearing
        match selected_tool {
            Tool::Select | Tool::Pan => {
                // Clear all annotations when switching to Select or Pan
                polygon.write().points.clear();
                polygon.write().is_closed = false;
                // Preview now handled in thread_local
                bbox.write().reset();
                comment.write().reset();
            }
            Tool::Polygon => {
                // Clear bbox and comment when switching to Polygon
                bbox.write().reset();
                comment.write().reset();
            }
            Tool::BoundingBox => {
                // Clear polygon and comment when switching to BBox
                polygon.write().points.clear();
                polygon.write().is_closed = false;
                // Preview now handled in thread_local
                comment.write().reset();
            }
            Tool::Comment => {
                // Clear polygon and bbox when switching to Comment
                polygon.write().points.clear();
                polygon.write().is_closed = false;
                // Preview now handled in thread_local
                bbox.write().reset();
            }
        }
    }
}
