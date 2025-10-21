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

fn polys_key(project_id: &str, block_id: &str, image_id: &str) -> String {
    format!("ann:{}:{}:{}:polys", project_id, block_id, image_id)
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

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let h = hex.trim_start_matches('#');
    if h.len() == 6 {
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
        (r, g, b)
    } else { (0,0,0) }
}

pub fn redraw_saved_polygons(
    ctx: &CanvasRenderingContext2d,
    project_id: &str,
    block_id: &str,
    image_id: &str,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) {
    // Clear first
    if let Some(canvas) = ctx.canvas() {
        let window = web_sys::window().expect("Should get window");
        let dpr = window.device_pixel_ratio();
        let css_w = canvas.width() as f64 / dpr;
        let css_h = canvas.height() as f64 / dpr;
        ctx.clear_rect(0.0, 0.0, css_w, css_h);
    }

    let polys = load_polygons(project_id, block_id, image_id);
let classes: Vec<crate::canvas::sidebar::ClassItem> =
        crate::canvas::sidebar::storage::load_json(&format!("classes:{}", project_id)).unwrap_or_default();
    let active: Option<String> = crate::canvas::sidebar::storage::load_json(&format!("active_class:{}", project_id)).unwrap_or(None);

    for sp in polys {
        // color per class
        let hex = classes.iter().find(|c| c.id == sp.class_id).map(|c| c.color.clone()).unwrap_or("#00C2FF".to_string());
        let (r,g,b) = hex_to_rgb(&hex);
        let is_active = active.as_ref().map(|a| a == &sp.class_id).unwrap_or(false);
        let fill_alpha = if is_active { 0.35 } else { 0.15 };
        let stroke_alpha = if is_active { 1.0 } else { 0.5 };
        let fill = format!("rgba({},{},{},{})", r, g, b, fill_alpha);
        let stroke = format!("rgba({},{},{},{})", r, g, b, stroke_alpha);

        // transform points
        let mut screen: Vec<(f64,f64)> = Vec::new();
        for p in &sp.points {
            screen.push((p.x * zoom + pan_x, p.y * zoom + pan_y));
        }
        if screen.len() < 2 { continue; }

        ctx.set_line_width(3.0);
        ctx.set_stroke_style(&JsValue::from_str(&stroke));
        ctx.set_fill_style(&JsValue::from_str(&fill));
        ctx.set_line_cap("round");
        ctx.set_line_join("round");
        ctx.begin_path();
        ctx.move_to(screen[0].0, screen[0].1);
        for (x,y) in &screen[1..] {
            ctx.line_to(*x, *y);
        }
        // close and fill
        ctx.close_path();
        ctx.fill();
        ctx.stroke();
    }
}
