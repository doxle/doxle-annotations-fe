use dioxus::prelude::*;

/// Sets up canvas size with device pixel ratio for crisp rendering
/// Delegates DOM access to dom_cache to avoid extra queries here
pub fn setup_canvas_size() {
    let _ = super::dom_cache::size_canvases_to_container();
}
