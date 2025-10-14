use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, CanvasPattern, HtmlCanvasElement};
use std::cell::RefCell;

// Cached pattern state
thread_local! {
    static PATTERN_CACHE: RefCell<Option<PatternCache>> = RefCell::new(None);
}

struct PatternCache {
    pattern: CanvasPattern,
    spacing: f64,
    minor_radius: f64,
    major_radius: f64,
    dpr: f64,
    color_minor: String,
    color_major: String,
}

const MIN_ZOOM: f64 = 0.1;   // 10%
const MAX_ZOOM: f64 = 16.0;  // 1600% (FigJam-like range)

#[component]
pub fn CanvasPage(task_id: ReadOnlySignal<String>) -> Element {
    let mut zoom = use_signal(|| 1.0);
    let mut pan_x = use_signal(|| 0.0);
    let mut pan_y = use_signal(|| 0.0);
    let mut is_panning = use_signal(|| false);
    let mut last_mouse_x = use_signal(|| 0.0);
    let mut last_mouse_y = use_signal(|| 0.0);
    
    // Initialize canvases on mount
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    let dpr = window.device_pixel_ratio();
                    let width = window.inner_width().unwrap().as_f64().unwrap();
                    let height = window.inner_height().unwrap().as_f64().unwrap();
                    tracing::info!("Canvas initialized - dpr: {}, width: {}, height: {}", dpr, width, height);
                    
                    // Setup dots canvas
                    if let Some(dots_canvas) = document.get_element_by_id("dots-canvas") {
                        let canvas: HtmlCanvasElement = dots_canvas.dyn_into().unwrap();
                        canvas.set_width((width * dpr) as u32);
                        canvas.set_height((height * dpr) as u32);
                    }
                    
                    // Setup annotation canvas
                    if let Some(ann_canvas) = document.get_element_by_id("annotation-canvas") {
                        let canvas: HtmlCanvasElement = ann_canvas.dyn_into().unwrap();
                        canvas.set_width((width * dpr) as u32);
                        canvas.set_height((height * dpr) as u32);
                    }
                }
            }
        }
    });
    
    // Redraw on zoom/pan changes
    use_effect(move || {
        let z = zoom();
        let px = pan_x();
        let py = pan_y();
        
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    // Redraw dots
                    if let Some(canvas) = document.get_element_by_id("dots-canvas") {
                        let canvas: HtmlCanvasElement = canvas.dyn_into().unwrap();
                        let context: CanvasRenderingContext2d = canvas
                            .get_context("2d")
                            .unwrap()
                            .unwrap()
                            .dyn_into()
                            .unwrap();
                        draw_dots(&context, &canvas, &window, z, px, py);
                    }
                    
                    // Redraw annotations
                    if let Some(canvas) = document.get_element_by_id("annotation-canvas") {
                        let canvas: HtmlCanvasElement = canvas.dyn_into().unwrap();
                        let context: CanvasRenderingContext2d = canvas
                            .get_context("2d")
                            .unwrap()
                            .unwrap()
                            .dyn_into()
                            .unwrap();
                        draw_annotations(&context, &canvas, &window, z, px, py);
                    }
                }
            }
        }
    });

    rsx! {
        div {
            class: "canvas-container",
            onmousedown: move |evt| {
                is_panning.set(true);
                let coords = evt.data.client_coordinates();
                last_mouse_x.set(coords.x);
                last_mouse_y.set(coords.y);
            },
            onmouseup: move |_| {
                is_panning.set(false);
            },
            onmouseleave: move |_| {
                is_panning.set(false);
            },
            onmousemove: move |evt| {
                if is_panning() {
                    let coords = evt.data.client_coordinates();
                    let dx = coords.x - last_mouse_x();
                    let dy = coords.y - last_mouse_y();
                    pan_x.set(pan_x() + dx);
                    pan_y.set(pan_y() + dy);
                    last_mouse_x.set(coords.x);
                    last_mouse_y.set(coords.y);
                }
            },
onwheel: move |evt| {
                let delta = evt.data.delta().strip_units().y;
                let zoom_factor = if delta < 0.0 { 1.1f64 } else { 0.9f64 };
                let new_zoom = (zoom() * zoom_factor).max(MIN_ZOOM).min(MAX_ZOOM);
                zoom.set(new_zoom);
            },
            
            // Layer 1: Dots canvas (bottom)
            canvas {
                id: "dots-canvas",
                class: "canvas-layer canvas-dots",
            }
            
            // Layer 2: Image (CSS transform for performance)
            div {
                class: "canvas-layer canvas-image",
                style: "transform: translate({pan_x()}px, {pan_y()}px) scale({zoom()});",
                // TODO: Add image element here
            }
            
            // Layer 3: Annotations canvas (top)
            canvas {
                id: "annotation-canvas",
                class: "canvas-layer canvas-annotations",
            }
            
            // UI overlay for zoom controls
            div {
                class: "canvas-ui-overlay",
                
                div {
                    class: "zoom-controls",
                    
                    button {
                        class: "zoom-btn",
onclick: move |_| {
                            let new_zoom = (zoom() * 1.2f64).min(MAX_ZOOM);
                            zoom.set(new_zoom);
                        },
                        "+"
                    }
                    
                    span {
                        class: "zoom-level",
                        "{(zoom() * 100.0) as i32}%"
                    }
                    
                    button {
                        class: "zoom-btn",
onclick: move |_| {
                            let new_zoom = (zoom() * 0.8f64).max(MIN_ZOOM);
                            zoom.set(new_zoom);
                        },
                        "−"
                    }
                    
                    button {
                        class: "zoom-btn",
                        onclick: move |_| {
                            zoom.set(1.0);
                            pan_x.set(0.0);
                            pan_y.set(0.0);
                        },
                        "Reset"
                    }
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn create_dot_pattern(
    window: &web_sys::Window,
    spacing: f64,
    minor_radius: f64,
    major_radius: f64,
    color_minor: &str,
    color_major: &str,
) -> Result<CanvasPattern, JsValue> {
    let document = window.document().unwrap();
    let dpr = window.device_pixel_ratio();
    
    // Build a 4x4 tile so every 4th intersection is a major dot
    let tile_css = (spacing * 4.0).max(1.0);
    let tile_px = (tile_css * dpr).round().max(1.0) as u32;

    let tile_canvas = document.create_element("canvas")?;
    let tile_canvas: HtmlCanvasElement = tile_canvas.dyn_into()?;
    tile_canvas.set_width(tile_px);
    tile_canvas.set_height(tile_px);
    
    let tile_ctx: CanvasRenderingContext2d = tile_canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into()?;
    
    // Draw in CSS pixels by scaling context
    tile_ctx.scale(dpr, dpr)?;

    // Minor dots at every spacing
    tile_ctx.set_fill_style(&JsValue::from_str(color_minor));
    for i in 0..4 {
        for j in 0..4 {
            // Skip the origin; it will host a major dot
            if i == 0 && j == 0 { continue; }
            let x = (i as f64) * spacing;
            let y = (j as f64) * spacing;
            tile_ctx.begin_path();
            tile_ctx.arc(x, y, minor_radius.max(0.25), 0.0, 2.0 * std::f64::consts::PI)?;
            tile_ctx.fill();
        }
    }

    // Major dot at the origin of the tile (repeats every 4 cells)
    tile_ctx.set_fill_style(&JsValue::from_str(color_major));
    tile_ctx.begin_path();
    tile_ctx.arc(0.0, 0.0, major_radius.max(minor_radius), 0.0, 2.0 * std::f64::consts::PI)?;
    tile_ctx.fill();
    
    // Create repeating pattern
    tile_ctx.create_pattern_with_html_canvas_element(&tile_canvas, "repeat")
        .ok()
        .flatten()
        .ok_or_else(|| JsValue::from_str("Failed to create pattern"))
}

#[cfg(target_arch = "wasm32")]
fn should_rebuild_pattern(
    cached: &PatternCache,
    spacing: f64,
    minor_radius: f64,
    major_radius: f64,
    dpr: f64,
    color_minor: &str,
    color_major: &str,
) -> bool {
    // Rebuild only when visuals or DPR change
    (cached.dpr - dpr).abs() > f64::EPSILON ||
    (cached.spacing - spacing).abs() > f64::EPSILON ||
    (cached.minor_radius - minor_radius).abs() > f64::EPSILON ||
    (cached.major_radius - major_radius).abs() > f64::EPSILON ||
    cached.color_minor != color_minor ||
    cached.color_major != color_major
}

#[cfg(target_arch = "wasm32")]
fn draw_dots(
    ctx: &CanvasRenderingContext2d,
    canvas: &HtmlCanvasElement,
    window: &web_sys::Window,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) {
    let dpr = window.device_pixel_ratio();
    let width = canvas.width() as f64;
    let height = canvas.height() as f64;
    
    // FigJam-like: uniform dot grid that scales with zoom
    let spacing = 12.0;  // base spacing in CSS px at zoom=1 (tighter like FigJam)
    let minor_radius = 0.6; // minor dot radius in CSS px
    let major_radius = 1.2; // major dot radius in CSS px (every 4th intersection)
    
    // Get background color from CSS variable (theme-aware)
    let bg_color = window
        .document()
        .and_then(|doc| doc.document_element())
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
        .and_then(|el| {
            window.get_computed_style(&el).ok().flatten()
        })
        .and_then(|style| {
            style.get_property_value("--bg-primary").ok()
        })
        .unwrap_or_else(|| "#ffffff".to_string());
    
    // Clear canvas with theme background
    ctx.set_fill_style(&JsValue::from_str(&bg_color));
    ctx.fill_rect(0.0, 0.0, width, height);
    
    // Get theme colors for minor/major dots
    let (dot_minor_color, dot_major_color) = window
        .document()
        .and_then(|doc| doc.document_element())
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
        .and_then(|el| window.get_computed_style(&el).ok().flatten())
        .map(|style| {
            let minor = style
                .get_property_value("--grid-dot-minor")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .or_else(|| style.get_property_value("--dot-color").ok())
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "rgba(149, 128, 255, 0.25)".to_string());
            let major = style
                .get_property_value("--grid-dot-major")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "rgba(149, 128, 255, 0.45)".to_string());
            (minor, major)
        })
        .unwrap_or(("rgba(149, 128, 255, 0.25)".to_string(), "rgba(149, 128, 255, 0.45)".to_string()));

    // Get or create pattern
    let pattern = PATTERN_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        
        let needs_rebuild = match cache.as_ref() {
            None => true,
            Some(cached) => should_rebuild_pattern(
                cached,
                spacing,
                minor_radius,
                major_radius,
                dpr,
                &dot_minor_color,
                &dot_major_color,
            ),
        };
        
        if needs_rebuild {
            if let Ok(new_pattern) = create_dot_pattern(
                window,
                spacing,
                minor_radius,
                major_radius,
                &dot_minor_color,
                &dot_major_color,
            ) {
                *cache = Some(PatternCache {
                    pattern: new_pattern,
                    spacing,
                    minor_radius,
                    major_radius,
                    dpr,
                    color_minor: dot_minor_color.clone(),
                    color_major: dot_major_color.clone(),
                });
            }
        }
        
        cache.as_ref().map(|c| c.pattern.clone())
    });
    
    if let Some(pattern) = pattern {
        ctx.save();
        
        // Scale context to DPR, then apply world transform so dots scale like FigJam
        ctx.scale(dpr, dpr).ok();
        
        let screen_scale = zoom;
        let screen_pan_x = pan_x;
        let screen_pan_y = pan_y;
        
        ctx.translate(screen_pan_x, screen_pan_y).ok();
        ctx.scale(screen_scale, screen_scale).ok();
        
        // Fill visible world rect
        let world_width = (width / dpr) / zoom;
        let world_height = (height / dpr) / zoom;
        let world_x = -pan_x / zoom;
        let world_y = -pan_y / zoom;
        
        ctx.set_fill_style(&pattern);
        ctx.fill_rect(world_x, world_y, world_width, world_height);
        
        ctx.restore();
    }
}

#[cfg(target_arch = "wasm32")]
fn draw_annotations(
    ctx: &CanvasRenderingContext2d,
    canvas: &HtmlCanvasElement,
    window: &web_sys::Window,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) {
    let dpr = window.device_pixel_ratio();
    let width = canvas.width() as f64;
    let height = canvas.height() as f64;
    
    // Clear canvas
    ctx.clear_rect(0.0, 0.0, width, height);
    
    ctx.save();
    
    // Scale for DPR
    ctx.scale(dpr, dpr).ok();
    
    // Apply world transform
    ctx.translate(pan_x, pan_y).ok();
    ctx.scale(zoom, zoom).ok();
    
    // TODO: Draw annotations here
    // Example: Draw a test rectangle
    // ctx.set_stroke_style(&JsValue::from_str("#ff0000"));
    // ctx.set_line_width(2.0 / zoom);
    // ctx.stroke_rect(100.0, 100.0, 200.0, 150.0);
    
    ctx.restore();
}
