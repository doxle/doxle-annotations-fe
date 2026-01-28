use dioxus::prelude::*;

const GRID_CSS:&str = include_str!("grid_overlay.css");
/// Positive modulo for f64 in [0, m)
fn pos_mod(a: f64, m: f64) -> f64 {
    if m <= 0.0 { return 0.0; }
    let r = a % m;
    if r < 0.0 { r + m } else { r }
}

/// Viewport-filling grid. Styling in grid_overlay.css (static).
/// We only pass dynamic numbers via CSS variables.
#[component]
pub fn GridOverlay(
    pan_x: f64,      // Transform.x (screen)
    pan_y: f64,      // Transform.y (screen)
    zoom: f64,       // Transform.zoom
    step_world: f64, // world units per cell (e.g., 25.0)
    opacity: f64,    // 0..1
    image_width: f64,  // world coords
    image_height: f64, // world coords
) -> Element {
    let step_minor_world = step_world;
    let step_major_world = step_world * 5.0;

    // On-screen spacing (px)
    let step_minor_screen = (step_minor_world * zoom).abs();
    let step_major_screen = (step_major_world * zoom).abs();

    // LOD: Hide if major grid is too dense
    if step_major_screen < 6.0 {
        return rsx! { Fragment {} };
    }

    // LOD: Hide minor grid if too dense (increased threshold to 15.0px)
    let show_minor = step_minor_screen >= 15.0;


    // Align grid to world origin so lines stick during pan
    let off_x_minor = pos_mod(pan_x, step_minor_screen);
    let off_y_minor = pos_mod(pan_y, step_minor_screen);
    
    let off_x_major = pos_mod(pan_x, step_major_screen);
    let off_y_major = pos_mod(pan_y, step_major_screen);

    // CSS variables for Minor Grid
    let minor_opacity_val = if show_minor { opacity } else { 0.0 };
    let minor_style = format!(
        "--grid-step: {s}px; --grid-off-x: {ox}px; --grid-off-y: {oy}px; --grid-opacity: {op};",
        s = step_minor_screen, ox = off_x_minor, oy = off_y_minor, op = minor_opacity_val
    );

    // CSS variables for Major Grid - Boost opacity (2x) for better visibility
    let major_opacity_val = (opacity * 1.0).min(1.0);
    let major_style = format!(
        "--grid-step: {s}px; --grid-off-x: {ox}px; --grid-off-y: {oy}px; --grid-opacity: {op};",
        s = step_major_screen, ox = off_x_major, oy = off_y_major, op = major_opacity_val
    );

    // Image bounds in screen coordinates
    let img_left = pan_x;
    let img_top = pan_y;
    let img_w = image_width * zoom;
    let img_h = image_height * zoom;

    // If no image, show full grid
    if image_width <= 0.0 || image_height <= 0.0 {
        return rsx! {
            style{{GRID_CSS}},
            div { class: "grid-overlay grid-minor", style: "{minor_style}" },
            div { class: "grid-overlay grid-major", style: "{major_style}" }
        };
    }

    // 4 regions around the image:
    // Top: full width, from 0 to img_top
    let top_style = format!("top:0; left:0; right:0; height:{h}px;", h = img_top.max(0.0));
    // Bottom: full width, from img_bottom to viewport bottom
    let bottom_style = format!("top:{t}px; left:0; right:0; bottom:0;", t = img_top + img_h);
    // Left: from img_top to img_bottom, 0 to img_left
    let left_style = format!("top:{t}px; left:0; width:{w}px; height:{h}px;", 
        t = img_top.max(0.0), w = img_left.max(0.0), h = img_h);
    // Right: from img_top to img_bottom, img_right to viewport right  
    let right_style = format!("top:{t}px; left:{l}px; right:0; height:{h}px;",
        t = img_top.max(0.0), l = img_left + img_w, h = img_h);

    rsx! {
        style{{GRID_CSS}},
        // Minor Grid - 4 regions
        div { class: "grid-overlay grid-minor grid-region", style: "{minor_style} {top_style}" },
        div { class: "grid-overlay grid-minor grid-region", style: "{minor_style} {bottom_style}" },
        div { class: "grid-overlay grid-minor grid-region", style: "{minor_style} {left_style}" },
        div { class: "grid-overlay grid-minor grid-region", style: "{minor_style} {right_style}" },
        // Major Grid - 4 regions
        div { class: "grid-overlay grid-major grid-region", style: "{major_style} {top_style}" },
        div { class: "grid-overlay grid-major grid-region", style: "{major_style} {bottom_style}" },
        div { class: "grid-overlay grid-major grid-region", style: "{major_style} {left_style}" },
        div { class: "grid-overlay grid-major grid-region", style: "{major_style} {right_style}" },
    }
}
