/**

AnnotationCanvasPage
└── SvgCanvas
    ├── world-container (div, transformed by pan/zoom)
    │   ├── ImageLayer (img)           ← children passed in
    │   └── AnnotationsLayer (svg)     ← children passed in
    ├── GridOverlay (div)              ← fixed, not in world-container
    ├── CrosshairOverlay (div)         ← fixed, not in world-container
    └── event-layer (div)              ← captures mouse events

**/

use dioxus::prelude::*;
use super::state::{CanvasState, Tool};
use super::grid_overlay::GridOverlay;
use super::dot_overlay::DotOverlay;
use super::crosshair_overlay::CrosshairOverlay;
use super::event_layer::EventLayer;

const SVG_CANVAS_CSS: &str = include_str!("svg_canvas.css");

#[component]
pub fn SvgCanvas(
    active_drawing:Signal<Vec<(f64,f64)>>, 
    cursor_world_pos:Signal<(f64,f64)>,
    image_size: Signal<(f64,f64)>,
    mut zoom: Signal<f64>,
    grid_visible: bool,
    selected_tool: Tool,
    children: Element,
    ) -> Element {
    // Provide canvas state to children
    let mut state = use_context_provider(|| Signal::new(CanvasState::new()));

    // Local UI state
    let is_dragging = use_signal(|| false);
    let last_mouse_pos = use_signal(|| (0.0, 0.0));
    let cursor_pos = use_signal(|| (0.0, 0.0));
    
    let cursor_inside = use_signal(|| false);
    let mut is_centering_done = use_signal(|| false);



    // Center image on first render
    let mut centered = use_signal(|| false);
    use_effect(move || {
        if centered() { return; }
        
        spawn(async move {
            use wasm_bindgen::JsCast;
            
            gloo_timers::future::TimeoutFuture::new(100).await;
            
            for _ in 0..10 {
                let Some(win) = web_sys::window() else { continue };
                let Some(doc) = win.document() else { continue };
                let Ok(Some(vp_el)) = doc.query_selector(".svg-canvas-viewport") else { continue };
                let Ok(Some(img_el)) = doc.query_selector(".svg-canvas-viewport .canvas-bg-image") else { continue };
                
                let vp_rect = vp_el.get_bounding_client_rect();
                let (vw, vh) = (vp_rect.width(), vp_rect.height());
                
                let Ok(img) = img_el.dyn_into::<web_sys::HtmlImageElement>() else { continue };
                let img_rect = img.get_bounding_client_rect();
                let (iw, ih) = (img_rect.width(), img_rect.height());
                
                if vw > 0.0 && vh > 0.0 && iw > 0.0 && ih > 0.0 {
                    let pan_x = (vw - iw) * 0.5;
                    let pan_y = (vh - ih) * 0.5;
                    state.write().set_view(1.0, pan_x, pan_y);
                    centered.set(true);
                    is_centering_done.set(true);
                    return;
                }
                
                gloo_timers::future::TimeoutFuture::new(50).await;
            }
        });
    });

    

    // Read transform and sync zoom signal
    let (pan_x, pan_y, zoom_val) = {
        let s = state.read();
        let t = *s.transform.read();
        (t.x, t.y, t.zoom)
    };
    
    // Update the zoom signal so parent/siblings can read it
    if zoom() != zoom_val {
        zoom.set(zoom_val);
    }

    rsx! {
        style { {SVG_CANVAS_CSS} }

        div {
            class: "svg-canvas-viewport",
            tabindex: "0",
            autofocus: true,

            style: match selected_tool {
                Tool::Polygon => "cursor:none;", // Hide cursor (crosshair overlay replaces it)
                _=> if is_dragging() {  // For Pan, BBox, Select, etc.
                    "cursor:grabbing;"  // Closed hand (while dragging)
                } else {
                    "cursor:grab;"      // Open hand (ready to drag)
                }
            },

            // World container - transformed by pan/zoom
            div {
                class: "world-container",
                style: format!(
                    "transform: translate({}px, {}px) scale({}); pointer-events: none; opacity:{};", 
                    pan_x, pan_y, zoom_val,
                    if is_centering_done() {1} else {0}
                    ),
                    {children} // All children come here
            }

            if grid_visible {
                DotOverlay {
                    zoom: zoom_val,
                    pan_x: pan_x,
                    pan_y: pan_y,
                    step_world: 25.0,
                    opacity: 1.0,
                    image_width: image_size().0,
                    image_height: image_size().1,
                }
            }

            CrosshairOverlay {
                x: cursor_pos().0,
                y: cursor_pos().1,
                visible: selected_tool == Tool::Polygon && cursor_inside(),
            }

            EventLayer {
                is_dragging: is_dragging,
                last_mouse_pos: last_mouse_pos,
                cursor_pos: cursor_pos,
                cursor_inside: cursor_inside,
                active_drawing:active_drawing,
                cursor_world_pos:cursor_world_pos,
                selected_tool: selected_tool,
            }
        }
    }
}
