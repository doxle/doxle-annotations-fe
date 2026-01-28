use dioxus::prelude::*;

const DOT_CSS: &str = include_str!("dot_overlay.css");

/// Positive modulo for f64 in [0, m)
fn pos_mod(a: f64, m: f64) -> f64 {
    if m <= 0.0 { return 0.0; }
    let r = a % m;
    if r < 0.0 { r + m } else { r }
}

/// Viewport-filling dot pattern overlay.
#[component]
pub fn DotOverlay(
    pan_x: f64,
    pan_y: f64,
    zoom: f64,
    step_world: f64,
    opacity: f64,
    image_width: f64,
    image_height: f64,
) -> Element {
    let step_screen = (step_world * zoom).abs();

    // Hide if too dense
    if step_screen < 8.0 {
        return rsx! { Fragment {} };
    }

    // Align dots to world origin
    let off_x = pos_mod(pan_x, step_screen);
    let off_y = pos_mod(pan_y, step_screen);

    // Dot size scales with zoom but clamped
    let dot_size = (2.0 * zoom).clamp(1.0, 4.0);

    let dot_style = format!(
        "--dot-step: {s}px; --dot-off-x: {ox}px; --dot-off-y: {oy}px; --dot-opacity: {op}; --dot-size: {ds}px;",
        s = step_screen, ox = off_x, oy = off_y, op = opacity, ds = dot_size
    );

    // Image bounds in screen coordinates
    let img_left = pan_x;
    let img_top = pan_y;
    let img_w = image_width * zoom;
    let img_h = image_height * zoom;

    // If no image, show full dot overlay
    if image_width <= 0.0 || image_height <= 0.0 {
        return rsx! {
            style { {DOT_CSS} }
            div { class: "dot-overlay", style: "{dot_style}" }
        };
    }

    // 4 regions around the image
    let top_style = format!("top:0; left:0; right:0; height:{h}px;", h = img_top.max(0.0));
    let bottom_style = format!("top:{t}px; left:0; right:0; bottom:0;", t = img_top + img_h);
    let left_style = format!("top:{t}px; left:0; width:{w}px; height:{h}px;", 
        t = img_top.max(0.0), w = img_left.max(0.0), h = img_h);
    let right_style = format!("top:{t}px; left:{l}px; right:0; height:{h}px;",
        t = img_top.max(0.0), l = img_left + img_w, h = img_h);

    rsx! {
        style { {DOT_CSS} }
        div { class: "dot-overlay dot-region", style: "{dot_style} {top_style}" }
        div { class: "dot-overlay dot-region", style: "{dot_style} {bottom_style}" }
        div { class: "dot-overlay dot-region", style: "{dot_style} {left_style}" }
        div { class: "dot-overlay dot-region", style: "{dot_style} {right_style}" }
    }
}
