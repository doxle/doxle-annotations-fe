use super::annotations::shapes::BBox;
use super::annotations::comment::Comment;
use super::annotations::shapes::Polygon;
use super::annotations::saved_canvas::{
    invalidate_bbox_cache, invalidate_polygon_cache, redraw_saved_annotations,
};
use super::annotations::Tool;
use dioxus::prelude::*;

/// Setup effect for redrawing saved annotations when class_counter changes
pub fn setup_annotation_redraw_effect(
    project_id: String,
    block_id: String,
    image_id: String,
    class_counter: Signal<u64>,
    zoom: Signal<f64>,
    pan_x: Signal<f64>,
    pan_y: Signal<f64>,
) {
    use_effect(move || {
        // Invalidate caches BEFORE reading class_counter to ensure fresh state
        invalidate_polygon_cache();
        invalidate_bbox_cache();
        let _ = class_counter(); // depend on class version to redraw after add/delete/reassign
        redraw_saved_annotations(&project_id, &block_id, &image_id, zoom(), pan_x(), pan_y());
    });
}

/// Setup effect for canvas sizing with DPR
pub fn setup_canvas_size_effect() {
    use_effect(move || {
        super::canvas_setup::setup_canvas_size();
    });
}

/// Setup effect to clear annotations when switching tools
pub fn setup_tool_switch_effect(
    selected_tool: Signal<Tool>,
    polygon: Signal<Polygon>,
    bbox: Signal<BBox>,
    comment: Signal<Comment>,
) {
    use_effect(move || {
        super::tool_effects::clear_annotations_on_tool_switch(
            selected_tool(),
            polygon,
            bbox,
            comment,
        );
    });
}
