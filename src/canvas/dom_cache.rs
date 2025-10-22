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
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement, Window};

// Thread-local storage for cached DOM references
//
// Why thread_local!?
// - In WASM we're single-threaded, but Rust doesn't allow mutable statics
// - thread_local! gives us mutable storage that's safe in single-threaded context
// - Each thread (just one in WASM) gets its own copy of the data
thread_local! {
    static WINDOW: RefCell<Option<Window>> = RefCell::new(None);
    static DOCUMENT: RefCell<Option<Document>> = RefCell::new(None);
    static CANVAS_CONTAINER: RefCell<Option<HtmlElement>> = RefCell::new(None);
    static OVERLAY_CTX: RefCell<Option<CanvasRenderingContext2d>> = RefCell::new(None);
    static SAVED_CTX: RefCell<Option<CanvasRenderingContext2d>> = RefCell::new(None);
}

/// Get the browser window object (cached after first call)
///
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

/// Get the document object (cached after first call)
///
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

// Get the #canvas-container element (cached after first call)
///
/// This is the main container that holds the canvases and receives mouse events.
/// We update its CSS variables (--cursor-x, --guide-x, etc.) on mousemove.
///
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

/// Get the overlay canvas 2D context (cached after first call)
/// Only for active drawing -> on finish gets transferred to saved_canvas
/// The overlay canvas is used for drawing in-progress annotations
/// (polygon preview lines, bbox preview, etc.)
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

/// Clear all caches (useful if DOM is recreated, though rare)
pub fn clear_all_caches() {
    WINDOW.with(|cell| *cell.borrow_mut() = None);
    DOCUMENT.with(|cell| *cell.borrow_mut() = None);
    CANVAS_CONTAINER.with(|cell| *cell.borrow_mut() = None);
    OVERLAY_CTX.with(|cell| *cell.borrow_mut() = None);
    SAVED_CTX.with(|cell| *cell.borrow_mut() = None);
}
