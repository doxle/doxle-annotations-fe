use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;
use std::cell::RefCell;

use crate::canvas::dom_cache::{get_saved_context, get_window};

use super::store::{load_polygons, load_bboxes, SavedPolygon, SavedBBox};
use crate::canvas::sidebar::ClassItem;
use crate::canvas::sidebar::storage::load_json;

// In-memory cache to avoid localStorage reads on every redraw
thread_local! {
    static CACHED_POLYGONS: RefCell<Option<(String, String, String, Vec<SavedPolygon>)>> = RefCell::new(None);
    static CACHED_BBOXES: RefCell<Option<(String, String, String, Vec<SavedBBox>)>> = RefCell::new(None);
    static CACHED_CLASSES: RefCell<Option<(String, Vec<ClassItem>)>> = RefCell::new(None);
    static SAVED_RAF_PENDING: RefCell<bool> = RefCell::new(false);
    static SAVED_CTX: RefCell<Option<CanvasRenderingContext2d>> = RefCell::new(None);
}

pub fn get_saved_canvas_context() -> Option<CanvasRenderingContext2d> {
    // Use centralized dom_cache
    get_saved_context()
}

pub fn invalidate_polygon_cache() {
    CACHED_POLYGONS.with(|cache| {
        *cache.borrow_mut() = None;
    });
}

pub fn invalidate_class_cache() {
    CACHED_CLASSES.with(|cache| {
        *cache.borrow_mut() = None;
    });
}

pub fn invalidate_bbox_cache() {
    CACHED_BBOXES.with(|cache| {
        *cache.borrow_mut() = None;
    });
}

fn get_cached_polygons(project_id: &str, block_id: &str, image_id: &str) -> Vec<SavedPolygon> {
    CACHED_POLYGONS.with(|cache| {
        let mut cache_mut = cache.borrow_mut();
        if let Some((pid, bid, iid, polys)) = cache_mut.as_ref() {
            if pid == project_id && bid == block_id && iid == image_id {
                return polys.clone();
            }
        }
        
        // Cache miss - load from localStorage
        let polys = load_polygons(project_id, block_id, image_id);
        *cache_mut = Some((project_id.to_string(), block_id.to_string(), image_id.to_string(), polys.clone()));
        polys
    })
}

fn get_cached_classes(project_id: &str) -> Vec<ClassItem> {
    CACHED_CLASSES.with(|cache| {
        let mut cache_mut = cache.borrow_mut();
        if let Some((pid, classes)) = cache_mut.as_ref() {
            if pid == project_id {
                return classes.clone();
            }
        }
        
        // Cache miss - load from localStorage
        let classes: Vec<ClassItem> = load_json(&format!("classes:{}", project_id)).unwrap_or_default();
        *cache_mut = Some((project_id.to_string(), classes.clone()));
        classes
    })
}

fn get_cached_bboxes(project_id: &str, block_id: &str, image_id: &str) -> Vec<SavedBBox> {
    CACHED_BBOXES.with(|cache| {
        let mut cache_mut = cache.borrow_mut();
        if let Some((pid, bid, iid, bboxes)) = cache_mut.as_ref() {
            if pid == project_id && bid == block_id && iid == image_id {
                return bboxes.clone();
            }
        }
        let bboxes = load_bboxes(project_id, block_id, image_id);
        *cache_mut = Some((project_id.to_string(), block_id.to_string(), image_id.to_string(), bboxes.clone()));
        bboxes
    })
}

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let h = hex.trim_start_matches('#');
    if h.len() == 6 {
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
        (r, g, b)
    } else {
        (0, 0, 0)
    }
}

fn redraw_saved_now(
    project_id: &str,
    block_id: &str,
    image_id: &str,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) {
    let ctx = match get_saved_canvas_context() {
        Some(c) => c,
        None => return,
    };

    // Clear entire device buffer regardless of current transform
    if let Some(canvas) = ctx.canvas() {
        let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    }

    // Use cached data
    let polys = get_cached_polygons(project_id, block_id, image_id);
    let classes = get_cached_classes(project_id);
    let active: Option<String> = load_json(&format!("active_class:{}", project_id)).unwrap_or(None);

    // Get DPR and apply combined transform (DPR * zoom + pan)
    let window = get_window().expect("Should get window");
    let dpr = window.device_pixel_ratio();
    // Apply transform ONCE - combines DPR scaling with zoom/pan
    let _ = ctx.set_transform(zoom * dpr, 0.0, 0.0, zoom * dpr, pan_x * dpr, pan_y * dpr);

    for sp in polys {
        // Get class color
        let hex = classes
            .iter()
            .find(|c| c.id == sp.class_id)
            .map(|c| c.color.clone())
            .unwrap_or("#00C2FF".to_string());
        let (r, g, b) = hex_to_rgb(&hex);
        let is_active = active.as_ref().map(|a| a == &sp.class_id).unwrap_or(false);
        let fill_alpha = if is_active { 0.35 } else { 0.15 };
        let stroke_alpha = if is_active { 1.0 } else { 0.5 };
        let fill = format!("rgba({},{},{},{})", r, g, b, fill_alpha);
        let stroke = format!("rgba({},{},{},{})", r, g, b, stroke_alpha);

        if sp.points.len() < 2 { continue; }

        ctx.set_line_width(3.0); // Line width in world coords
        let _ = ctx.set_stroke_style_str(&stroke);
        let _ = ctx.set_fill_style_str(&fill);
        ctx.set_line_cap("round");
        ctx.set_line_join("round");
        
        ctx.begin_path();
        // Draw in WORLD coordinates - browser transforms them
        ctx.move_to(sp.points[0].x, sp.points[0].y);
        for p in &sp.points[1..] { ctx.line_to(p.x, p.y); }
        ctx.close_path();
        ctx.fill();
        ctx.stroke();
    }

    // Also draw saved BBoxes
    let bboxes = get_cached_bboxes(project_id, block_id, image_id);
    for bb in bboxes {
        let hex = classes
            .iter()
            .find(|c| c.id == bb.class_id)
            .map(|c| c.color.clone())
            .unwrap_or("#00C2FF".to_string());
        let (r, g, b) = hex_to_rgb(&hex);
        let is_active = active.as_ref().map(|a| a == &bb.class_id).unwrap_or(false);
        let fill_alpha = if is_active { 0.25 } else { 0.12 };
        let stroke_alpha = if is_active { 1.0 } else { 0.5 };
        let fill = format!("rgba({},{},{},{})", r, g, b, fill_alpha);
        let stroke = format!("rgba({},{},{},{})", r, g, b, stroke_alpha);

        let x = bb.start.x.min(bb.end.x);
        let y = bb.start.y.min(bb.end.y);
        let w = (bb.end.x - bb.start.x).abs();
        let h = (bb.end.y - bb.start.y).abs();

        ctx.set_line_width(3.0);
        let _ = ctx.set_stroke_style_str(&stroke);
        let _ = ctx.set_fill_style_str(&fill);
        ctx.set_line_cap("round");
        ctx.set_line_join("round");
        ctx.begin_path();
        ctx.rect(x, y, w, h);
        ctx.fill();
        ctx.stroke();
    }

    // Reset transform
    let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
}

// Public API with rAF throttling
pub fn redraw_saved_annotations(
    project_id: &str,
    block_id: &str,
    image_id: &str,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) {
    // Check if already pending
    let should_schedule = SAVED_RAF_PENDING.with(|flag| {
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

    let pid = project_id.to_string();
    let bid = block_id.to_string();
    let iid = image_id.to_string();

    use wasm_bindgen::{closure::Closure, JsCast};
    let closure = Closure::wrap(Box::new(move || {
        redraw_saved_now(&pid, &bid, &iid, zoom, pan_x, pan_y);
        SAVED_RAF_PENDING.with(|flag| *flag.borrow_mut() = false);
    }) as Box<dyn FnMut()>);

    if let Some(win) = get_window() {
        let _ = win.request_animation_frame(closure.as_ref().unchecked_ref());
    }
    closure.forget();
}
