use dioxus::prelude::*;
use std::cell::RefCell;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast; //CONVERTS JS OBJECTS TO RUST TYPES
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

thread_local! {
    static PATTERN_CACHE:RefCell<Option<PatternCache>> = RefCell::new(None);
}
struct PatternCache {
    pattern: web_sys::CanvasPattern,
    spacing: f64,
    dpr: f64,
    color: String,
}

#[component]
pub fn CanvasPage(task_id: ReadOnlySignal<String>) -> Element {
    let _redraw_theme_trigger = use_signal(|| 0);
    let mut zoom = use_signal(|| 1.0);
    let mut pan_x = use_signal(|| 0.0);
    let mut pan_y = use_signal(|| 0.0);
    

    // Resize and theme listeners removed - using CSS GPU acceleration now


    // CSS dots configuration
    let dot_spacing_px: f64 = 12.0;
    let dot_radius_px: f64 = 1.0;
    let dots_style = format!(
        "pointer-events: none; background: radial-gradient(circle, var(--dot-color) 0px, var(--dot-color) {r}px, transparent {r}px); background-size: {s}px {s}px; background-position: 0 0;",
        r = dot_radius_px,
        s = dot_spacing_px,
    );

    rsx! {
        div{
            class:"canvas-container",
            // Layer 1: GPU-accelerated CSS dots
            div{
                class: "canvas-layer canvas-dots",
                style: "{dots_style}",
            }
            // Layer 2: Image (ADD YOUR IMAGE PATH HERE!)
            div {
                class: "canvas-layer canvas-image",
                style: "transform-origin: 0 0;",
                img {
                    src: "/path/to/your/image.jpg",
                    style: "display: block; width: auto; height: auto; max-width: none;"
                }
            }
            // Layer 3: Transparent canvas for annotations
            canvas{
                id:"dots-canvas",
                class:"canvas-layer",
                style: "background-color: transparent;",
            }
        }
    }
}

// Canvas drawing removed - using CSS GPU acceleration for dots
// Canvas layer kept for future annotations

#[cfg(target_arch = "wasm32")]
fn get_css_variable(window: &web_sys::Window, variable: &str) -> Option<String> {
    let document = window.document()?;
    let element = document.document_element()?;
    let computed_style = window.get_computed_style(&element).ok()??;
    let value = computed_style.get_property_value(variable).ok()?;
    if value.is_empty() {
        None
    } else {
        Some(value.trim().to_string())
    }
}

#[cfg(target_arch = "wasm32")]
fn create_dot_pattern(
    window: &web_sys::Window,
    spacing: f64,
    dot_color: &str,
) -> Result<web_sys::CanvasPattern, JsValue> {
    let document = window.document().unwrap();
    let dpr = window.device_pixel_ratio();
    //Create a small tile canvas (just the spacing size)
    let tile_css = spacing;
    let tile_px = (tile_css * dpr).round() as u32;
    let tile_canvas = document
        .create_element("canvas")
        .expect("Should create canvas element");
    let tile_canvas: HtmlCanvasElement = tile_canvas
        .dyn_into()
        .expect("Should create HtmlCanvasElement");
    tile_canvas.set_width(tile_px);
    tile_canvas.set_height(tile_px);
    let tile_ctx: CanvasRenderingContext2d = tile_canvas
        .get_context("2d")
        .expect("Should get 2d context")
        .expect("Context should exist")
        .dyn_into()
        .expect("Should be CanvasRenderingContext2d");

    //Scale context for DPR
    tile_ctx.scale(dpr, dpr).expect("Should scale for DPR");

    //Draw one dot at the CENTER of the tile (not corner!)
    tile_ctx.set_fill_style(&JsValue::from_str(dot_color));
    tile_ctx.begin_path();
    // Center the dot in the tile so it's visible when tiled
    let dot_x = spacing / 2.0; // 18.0 / 2.0 = 9.0 (center)
    let dot_y = spacing / 2.0; // 18.0 / 2.0 = 9.0 (center)
    tracing::info!(
        "Drawing dot at ({}, {}) with radius 1.0px, color: {}",
        dot_x,
        dot_y,
        dot_color
    );
    tile_ctx
        .arc(dot_x, dot_y, 0.6, 0.0, 2.0 * std::f64::consts::PI)
        .expect("Should draw arc");
    tile_ctx.fill();

    //Create repeating pattern from this tile
    let pattern = tile_ctx
        .create_pattern_with_html_canvas_element(&tile_canvas, "repeat")
        .ok()
        .flatten()
        .ok_or_else(|| JsValue::from_str("Failed to create pattern"))
        .expect("Should have created tile pattern");
    tracing::info!("Pattern created succesfully");
    Ok(pattern)
}

#[cfg(target_arch = "wasm32")]
fn should_rebuild_pattern(
    cached: &PatternCache,
    spacing: f64, //Physical dot spacing (always 18.0)
    dpr: f64,     //Only changes on monitor switch
    color: &str,  //Only changes on theme toggle
) -> bool {
    //Rebuild only when visuals or DPR change
    let dpr_changed = (cached.dpr - dpr).abs() > f64::EPSILON;
    let spacing_changed = (cached.spacing - spacing).abs() > f64::EPSILON;
    let color_changed = cached.color != color;
    
    if dpr_changed || spacing_changed || color_changed {
        tracing::info!("Rebuild reason - DPR: {}, Spacing: {}, Color: {} (cached: '{}', new: '{}')", 
            dpr_changed, spacing_changed, color_changed, cached.color, color);
    }
    
    dpr_changed || spacing_changed || color_changed
}
