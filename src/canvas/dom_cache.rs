//! Centralized cache for DOM elements and canvas contexts.
//!
//! WHY THIS EXISTS:
//! - DOM queries (getElementById, etc.) are relatively expensive (~0.05-0.2ms each)
//! - We query the same elements repeatedly on every mouse event
//! - Caching them means we query once, reuse forever
//!
//! HOW IT WORKS:
//! - thread_local! stores a cached reference to each DOM element
//! - First call: queries DOM, stores reference, returns it
//! - Subsequent calls: returns cached reference (no DOM query!)
//! - The reference points to the LIVE DOM element, so updates still work
//!
//! PATTERN:
//! Every getter follows this pattern:
//! 1. Check cache
//! 2. If cached, return it
//! 3. If not cached, query DOM, store in cache, return it

use std::cell::RefCell;
use std::sync::RwLock;
use wasm_bindgen::JsCast;
use web_sys::{
    window, CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement, HtmlImageElement,
    Window,
};

// Thread-local storage for cached DOM references
//
// Why thread_local!?
// - In WASM we're single-threaded, but Rust doesn't allow mutable statics
// - thread_local! gives us mutable storage that's safe in single-threaded context
// - Each thread (just one in WASM) gets its own copy of the data
thread_local! {
    static WINDOW: RefCell<Option<Window>> = RefCell::new(None);
    static DOCUMENT: RefCell<Option<Document>> = RefCell::new(None);
    static IMAGE_ELEMENT: RefCell<Option<HtmlImageElement>> = RefCell::new(None);
    static CANVAS_CONTAINER: RefCell<Option<HtmlElement>> = RefCell::new(None);
    static OVERLAY_CTX: RefCell<Option<CanvasRenderingContext2d>> = RefCell::new(None);
    static SAVED_CTX: RefCell<Option<CanvasRenderingContext2d>> = RefCell::new(None);
}

pub static IMAGE_WORLD_BOUNDS: RwLock<Option<(f64, f64, f64, f64)>> = RwLock::new(None);

/// Used for getting DPR, document, scheduling requestAnimationFrame, etc.
pub fn get_window() -> Option<Window> {
    WINDOW.with(|cell| {
        let mut cached = cell.borrow_mut();

        // If we already have it cached, return it
        if let Some(win) = cached.as_ref() {
            return Some(win.clone());
        }

        // Cache miss - query once and store
        let win = window()?;
        *cached = Some(win.clone());
        Some(win)
    })
}

/// Used for querying elements by ID
pub fn get_document() -> Option<Document> {
    DOCUMENT.with(|cell| {
        let mut cached = cell.borrow_mut();

        if let Some(doc) = cached.as_ref() {
            return Some(doc.clone());
        }

        // Get document through window (also uses cache)
        let doc = get_window()?.document()?;
        *cached = Some(doc.clone());
        Some(doc)
    })
}

pub fn get_image_element() -> Option<HtmlImageElement> {
    IMAGE_ELEMENT.with(|cell| {
        if let Some(img) = cell.borrow().as_ref() {
            return Some(img.clone());
        }
        // Query DOM and cast to HtmlImageElement (needed for .style())
        let img = get_document()?
            .query_selector(".canvas-image img")
            .ok()
            .flatten()?
            .dyn_into::<HtmlImageElement>()
            .ok()?;

        *cell.borrow_mut() = Some(img.clone());
        Some(img)
    })
}


// Get the #canvas-container element (cached after first call)
/// PERFORMANCE: Without caching, we'd query this ~120 times/sec at 120Hz mouse
pub fn get_canvas_container() -> Option<HtmlElement> {
    CANVAS_CONTAINER.with(|cell| {
        let mut cached = cell.borrow_mut();

        if let Some(elem) = cached.as_ref() {
            return Some(elem.clone());
        }

        // Query DOM and cast to HtmlElement (needed for .style())
        let elem = get_document()?
            .get_element_by_id("canvas-container")?
            .dyn_into::<HtmlElement>()
            .ok()?;

        *cached = Some(elem.clone());
        Some(elem)
    })
}

// getters (DOM queries happen only here on first call)
pub fn get_overlay_context() -> Option<CanvasRenderingContext2d> {
    OVERLAY_CTX.with(|cell| {
        let mut cached = cell.borrow_mut();

        if let Some(ctx) = cached.as_ref() {
            return Some(ctx.clone());
        }

        // Query canvas element and get 2D context
        let ctx = get_document()?
            .get_element_by_id("canvas-overlay")?
            .dyn_into::<HtmlCanvasElement>()
            .ok()?
            .get_context("2d")
            .ok()?
            .and_then(|ctx| ctx.dyn_into::<CanvasRenderingContext2d>().ok())?;

        *cached = Some(ctx.clone());
        Some(ctx)
    })
}

/// Get the saved canvas 2D context (cached after first call)
/// COMPLETED ANNOTATIONS
/// The saved canvas is used for drawing completed/saved annotations
pub fn get_saved_context() -> Option<CanvasRenderingContext2d> {
    SAVED_CTX.with(|cell| {
        let mut cached = cell.borrow_mut();

        if let Some(ctx) = cached.as_ref() {
            return Some(ctx.clone());
        }

        let ctx = get_document()?
            .get_element_by_id("canvas-saved")?
            .dyn_into::<HtmlCanvasElement>()
            .ok()?
            .get_context("2d")
            .ok()?
            .and_then(|ctx| ctx.dyn_into::<CanvasRenderingContext2d>().ok())?;

        *cached = Some(ctx.clone());
        Some(ctx)
    })
}

// Convert DOM rect -> WORLD once, using current zoom/pan
pub fn cache_image_world_bounds_from_dom(zoom: f64, pan_x: f64, pan_y: f64) -> Option<()> {
    let img = get_image_element()?;
    let container = get_canvas_container()?;
    let img_bounding_rect = img.get_bounding_client_rect();
    let container_bounding_rect = container.get_bounding_client_rect();

    // container-relative screen coords; We dont want it from screen we want it from container
    let left_s = img_bounding_rect.left() - container_bounding_rect.left();
    let top_s = img_bounding_rect.top() - container_bounding_rect.top();
    let right_s = img_bounding_rect.right() - container_bounding_rect.left();
    let bottom_s = img_bounding_rect.bottom() - container_bounding_rect.top();

    // container-relative screen coords
    let left = (left_s - pan_x) / zoom;
    let top = (top_s - pan_y) / zoom;
    let right = (right_s - pan_x) / zoom;
    let bottom = (bottom_s - pan_y) / zoom;
    if let Ok(mut w) = IMAGE_WORLD_BOUNDS.write() {
        *w = Some((left, top, right, bottom));
    }
    Some(())
}

pub fn get_image_world_bounds_cached() -> Option<(f64, f64, f64, f64)> {
    IMAGE_WORLD_BOUNDS.read().ok().and_then(|v| *v)
}

// Centralize canvas sizing so only dom_cache queries DOM
pub fn size_canvases_to_container() -> Option<()> {
    let win = get_window()?;
    let dpr = win.device_pixel_ratio();
    let (css_w, css_h) = if let Some(container) = get_canvas_container() {
        let r = container.get_bounding_client_rect();
        (r.width(), r.height())
    } else {
        let w = win.inner_width().ok()?.as_f64()?;
        let h = win.inner_height().ok()?.as_f64()?;
        (w, h)
    };

    for id in ["canvas-saved", "canvas-overlay"] {
        if let Some(el) = get_document()?.get_element_by_id(id) {
            if let Ok(canvas) = el.dyn_into::<HtmlCanvasElement>() {
                canvas.set_width((css_w * dpr).round() as u32);
                canvas.set_height((css_h * dpr).round() as u32);
                let _ = canvas
                    .style()
                    .set_property("width", &format!("{}px", css_w));
                let _ = canvas
                    .style()
                    .set_property("height", &format!("{}px", css_h));
            }
        }
    }
    Some(())
}

/// Clear all caches (useful if DOM is recreated, though rare)
pub fn clear_all_caches() {
    WINDOW.with(|cell| *cell.borrow_mut() = None);
    DOCUMENT.with(|cell| *cell.borrow_mut() = None);
    CANVAS_CONTAINER.with(|cell| *cell.borrow_mut() = None);
    IMAGE_ELEMENT.with(|cell| *cell.borrow_mut() = None);
    OVERLAY_CTX.with(|cell| *cell.borrow_mut() = None);
    SAVED_CTX.with(|cell| *cell.borrow_mut() = None);
}
