use serde::{Deserialize, Serialize};
use crate::canvas::sidebar::storage::{load_json, save_json};
use super::polygon::Polygon;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedPoint { pub x: f64, pub y: f64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedPolygon {
    pub class_id: String,
    pub points: Vec<SavedPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedBBox {
    pub class_id: String,
    pub start: SavedPoint,
    pub end: SavedPoint,
}

fn polys_key(project_id: &str, block_id: &str, image_id: &str) -> String {
    format!("ann:{}:{}:{}:polys", project_id, block_id, image_id)
}

fn bboxes_key(project_id: &str, block_id: &str, image_id: &str) -> String {
    format!("ann:{}:{}:{}:bboxes", project_id, block_id, image_id)
}

pub fn save_polygon(project_id: &str, block_id: &str, image_id: &str, poly: &Polygon, class_id: &str) {
    let mut all: Vec<SavedPolygon> = load_json(&polys_key(project_id, block_id, image_id)).unwrap_or_default();
    let pts = poly.points.iter().map(|p| SavedPoint { x: p.x, y: p.y }).collect();
    all.push(SavedPolygon { class_id: class_id.to_string(), points: pts });
    save_json(&polys_key(project_id, block_id, image_id), &all);
}

pub fn load_polygons(project_id: &str, block_id: &str, image_id: &str) -> Vec<SavedPolygon> {
    load_json(&polys_key(project_id, block_id, image_id)).unwrap_or_default()
}

pub fn save_bbox(project_id: &str, block_id: &str, image_id: &str, start: &SavedPoint, end: &SavedPoint, class_id: &str) {
    let mut all: Vec<SavedBBox> = load_json(&bboxes_key(project_id, block_id, image_id)).unwrap_or_default();
    all.push(SavedBBox { class_id: class_id.to_string(), start: start.clone(), end: end.clone() });
    save_json(&bboxes_key(project_id, block_id, image_id), &all);
}

pub fn load_bboxes(project_id: &str, block_id: &str, image_id: &str) -> Vec<SavedBBox> {
    load_json(&bboxes_key(project_id, block_id, image_id)).unwrap_or_default()
}

// --- Update/Delete helpers for annotations ---
pub fn update_polygon_class(project_id: &str, block_id: &str, image_id: &str, index: usize, new_class_id: &str) -> bool {
    let mut all: Vec<SavedPolygon> = load_polygons(project_id, block_id, image_id);
    if index >= all.len() { return false; }
    all[index].class_id = new_class_id.to_string();
    save_json(&polys_key(project_id, block_id, image_id), &all);
    true
}

pub fn delete_polygon_at(project_id: &str, block_id: &str, image_id: &str, index: usize) -> bool {
    let mut all: Vec<SavedPolygon> = load_polygons(project_id, block_id, image_id);
    if index >= all.len() { return false; }
    all.remove(index);
    save_json(&polys_key(project_id, block_id, image_id), &all);
    true
}

pub fn update_bbox_class(project_id: &str, block_id: &str, image_id: &str, index: usize, new_class_id: &str) -> bool {
    let mut all: Vec<SavedBBox> = load_bboxes(project_id, block_id, image_id);
    if index >= all.len() { return false; }
    all[index].class_id = new_class_id.to_string();
    save_json(&bboxes_key(project_id, block_id, image_id), &all);
    true
}

pub fn delete_bbox_at(project_id: &str, block_id: &str, image_id: &str, index: usize) -> bool {
    let mut all: Vec<SavedBBox> = load_bboxes(project_id, block_id, image_id);
    if index >= all.len() { return false; }
    all.remove(index);
    save_json(&bboxes_key(project_id, block_id, image_id), &all);
    true
}

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let h = hex.trim_start_matches('#');
    if h.len() == 6 {
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
        (r, g, b)
    } else { (0,0,0) }
}

// NOTE: kept for backward compatibility if any caller still uses this; prefer saved_canvas.rs
pub fn redraw_saved_polygons(
    ctx: &CanvasRenderingContext2d,
    project_id: &str,
    block_id: &str,
    image_id: &str,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) {
    // Clear full device buffer
    if let Some(canvas) = ctx.canvas() {
        let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    }

    let polys = load_polygons(project_id, block_id, image_id);
    let classes: Vec<crate::canvas::sidebar::ClassItem> =
        crate::canvas::sidebar::storage::load_json(&format!("classes:{}", project_id)).unwrap_or_default();
    let active: Option<String> = crate::canvas::sidebar::storage::load_json(&format!("active_class:{}", project_id)).unwrap_or(None);

    // Apply transform once
    if let Some(win) = web_sys::window() { let dpr = win.device_pixel_ratio(); let _ = ctx.set_transform(zoom * dpr, 0.0, 0.0, zoom * dpr, pan_x * dpr, pan_y * dpr); }

    for sp in polys {
        let hex = classes.iter().find(|c| c.id == sp.class_id).map(|c| c.color.clone()).unwrap_or("#00C2FF".to_string());
        let (r,g,b) = hex_to_rgb(&hex);
        let is_active = active.as_ref().map(|a| a == &sp.class_id).unwrap_or(false);
        let fill_alpha = if is_active { 0.35 } else { 0.15 };
        let stroke_alpha = if is_active { 1.0 } else { 0.5 };
        let fill = format!("rgba({},{},{},{})", r, g, b, fill_alpha);
        let stroke = format!("rgba({},{},{},{})", r, g, b, stroke_alpha);

        if sp.points.len() < 2 { continue; }

        ctx.set_line_width(3.0);
        ctx.set_stroke_style(&JsValue::from_str(&stroke));
        ctx.set_fill_style(&JsValue::from_str(&fill));
        ctx.set_line_cap("round");
        ctx.set_line_join("round");
        ctx.begin_path();
        // World coords, transform handles zoom/pan
        ctx.move_to(sp.points[0].x, sp.points[0].y);
        for p in &sp.points[1..] { ctx.line_to(p.x, p.y); }
        ctx.close_path();
        ctx.fill();
        ctx.stroke();
    }

    // Reset transform
    let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
}
