use super::dom_cache::{cache_image_world_bounds_from_dom, get_image_element, get_image_world_bounds_cached};

pub const NAVBAR_H: f64 = 36.0; // canvas container sits below navbar

// Read cached WORLD bounds (zoom/pan params ignored, kept for signature compatibility)
pub fn get_image_bounds_world(
    _zoom: f64,
    _pan_x: f64,
    _pan_y: f64,
) -> Option<(f64, f64, f64, f64)> {
    get_image_world_bounds_cached()
}

// Called once when the image loads (or whenever layout changes) to set WORLD bounds
pub fn cache_image_world_bounds(zoom: f64, pan_x: f64, pan_y: f64) -> Option<()> {
    cache_image_world_bounds_from_dom(zoom, pan_x, pan_y)
}

// Check if a viewport point lies inside the displayed image rect (SCREEN coordinates).
pub fn point_inside_image(screen_x: f64, screen_y: f64) -> bool {
    if let Some(img) = get_image_element() {
        let r = img.get_bounding_client_rect();
        return screen_x >= r.left()
            && screen_x <= r.right()
            && screen_y >= r.top()
            && screen_y <= r.bottom();
    }
    true // If we can't detect the image, allow input (fallback)
}

pub fn clamp_f64(v: f64, min: f64, max: f64) -> f64 {
    v.max(min).min(max)
}
