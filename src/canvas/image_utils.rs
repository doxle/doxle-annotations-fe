use super::dom_cache::{get_window, get_document};

pub const NAVBAR_H: f64 = 36.0; // canvas container sits below navbar

pub static IMAGE_SCREEN_BOUNDS: std::sync::RwLock<Option<(f64, f64, f64, f64)>> =
    std::sync::RwLock::new(None);

// Get image bounds in WORLD coordinates using cached screen bounds + zoom/pan
pub fn get_image_bounds_world(zoom: f64, pan_x: f64, pan_y: f64) -> Option<(f64, f64, f64, f64)> {
    //Try to read from cache
    let (screen_left, screen_top, screen_right, screen_bottom) = {
        if let Ok(cache) = IMAGE_SCREEN_BOUNDS.read() {
            if let Some(bounds) = *cache {
                bounds //Found Cache
            } else {
                // Cache miss - fallback to DOM query
                let win = get_window()?;
                let doc = get_document()?;
                let el = doc.query_selector(".canvas-image img").ok().flatten()?;
                let rect = el.get_bounding_client_rect();
                let left = (rect.left() - pan_x) / zoom;
                let top = (rect.top() - NAVBAR_H - pan_y) / zoom;
                let right = (rect.right() - pan_x) / zoom;
                let bottom = (rect.bottom() - NAVBAR_H - pan_y) / zoom;
                return Some((left, top, right, bottom));
            }
        } else {
            tracing::info!("[CACHE ERROR] Failed to acquire read lock");
            return None;
        }
    };

    // Convert cached screen bounds to world coordinates using current zoom/pan
    let left = (screen_left - pan_x) / zoom;
    let top = (screen_top - pan_y) / zoom;
    let right = (screen_right - pan_x) / zoom;
    let bottom = (screen_bottom - pan_y) / zoom;
    Some((left, top, right, bottom))
}

// this is called on every click so we need to cache the image screen bounds
pub fn cache_image_screen_bounds() -> Option<()> {
    let win = get_window()?;
    let doc = get_document()?;
    let el = doc.query_selector(".canvas-image img").ok().flatten()?;
    let rect = el.get_bounding_client_rect();
    let left = rect.left();
    let top = rect.top() - NAVBAR_H;
    let right = rect.right();
    let bottom = rect.bottom() - NAVBAR_H;
    *IMAGE_SCREEN_BOUNDS.write().unwrap() = Some((left, top, right, bottom));

    tracing::info!(
        "Image screen bounds cached {}, {}, {}, {}",
        left,
        top,
        right,
        bottom
    );
    Some(())
}

// Check if a viewport point lies inside the displayed image rect (SCREEN coordinates).
pub fn point_inside_image(screen_x: f64, screen_y: f64) -> bool {
    if let Some(win) = get_window() {
        if let Some(doc) = get_document() {
            if let Ok(Some(el)) = doc.query_selector(".canvas-image img") {
                let r = el.get_bounding_client_rect();
                return screen_x >= r.left()
                    && screen_x <= r.right()
                    && screen_y >= r.top()
                    && screen_y <= r.bottom();
            }
        }
    }
    true // If we can't detect the image, allow input (fallback)
}

pub fn clamp_f64(v: f64, min: f64, max: f64) -> f64 {
    v.max(min).min(max)
}
