use dioxus::prelude::*;

const CROSSHAIR_CSS: &str = include_str!("crosshair_overlay.css");

/// Viewport-level red crosshair drawn at the cursor.
/// Pass screen coords (element_coordinates) via CSS vars; styling is static CSS.
#[component]
pub fn CrosshairOverlay(x: f64, y: f64, visible: bool) -> Element {
    if !visible { return rsx! { Fragment {} }; }

    let style = format!("--cx: {x}px; --cy: {y}px;");

    rsx! {
        style{{CROSSHAIR_CSS}}
        div { class: "crosshair-overlay", style: "{style}",
            div { class: "h" }
            div { class: "v" }
        }
    }
}

/// Bbox-specific crosshair: same grid-line style as polygon but in --comment-blue.
#[component]
pub fn BboxCrosshairOverlay(x: f64, y: f64, visible: bool) -> Element {
    if !visible { return rsx! { Fragment {} }; }

    let style = format!("--cx: {x}px; --cy: {y}px;");

    rsx! {
        style{{CROSSHAIR_CSS}}
        div { class: "bbox-crosshair-overlay", style: "{style}",
            div { class: "h" }
            div { class: "v" }
        }
    }
}
