use super::canvas_navbar::{AnnotationTool, CanvasNavbar};
use dioxus::prelude::*;

#[component]
pub fn CanvasPage(task_id: String) -> Element {
    // Extract project_id from task_id (format: "project_id-task_id" or just use "1" for now)
    let project_id = "1".to_string(); // TODO: Parse from task_id or pass as separate param

    // --- State ---
    let mut zoom = use_signal(|| 1.0);
    let mut pan_x = use_signal(|| 0.0);
    let mut pan_y = use_signal(|| 0.0);
    let mut selected_tool = use_signal(|| Option::<AnnotationTool>::None);

    // --- Pan ----
    let mut is_panning: Signal<bool> = use_signal(|| false);
    let mut last_x: Signal<f64> = use_signal(|| 0.0);
    let mut last_y: Signal<f64> = use_signal(|| 0.0);

    // --- Assets ----
    const CANVAS_IMG: Asset = asset!("/assets/images/test.png");

    // --- Dots & Theme ---
    let dot_spacing_px: f64 = 12.0;
    let dot_radius_px: f64 = 1.0;

    // --- Cursor State ---
    let container_class = if is_panning() {
        "canvas-container is-panning"
    } else {
        "canvas-container"
    };

    // -------------------------------------------------------------------------

    // --- Handlers ----

    //Zoom with current cursor position
    let onwheel = move |evt: Event<WheelData>| {
        evt.prevent_default();

        //Get mouse position in viewport (where cursor is on screen)
        let mouse_coords = evt.client_coordinates();
        let mouse_x = mouse_coords.x;
        let mouse_y = mouse_coords.y;

        //Calculate zoom change
        let delta_y = evt.data.delta().strip_units().y;
        // let zoom_step: f64 = 0.0010;
        let zoom_factor = if delta_y < 0.0 { 1.1 } else { 0.9 };

        // ❶ exponential zoom factor:
        //    -dy makes scrolling up (negative) zoom in.
        //    1.0015 ~ gentle; bump to 1.003 for faster; 1.01 for super fast.
        // let zoom_factor = (1.0 + zoom_step).powf(-delta_y);
        let old_zoom = zoom();
        let mut new_zoom = (old_zoom * zoom_factor as f64).clamp(0.3, 6.0);

        // (Optional) snap tiny zooms to avoid jitter
        if new_zoom < 0.06 {
            new_zoom = 0.05;
        }

        //Calculate the zoom ratio
        let zoom_ratio = new_zoom / old_zoom;

        //Current pans
        let current_pan_x = pan_x();
        let current_pan_y = pan_y();

        //Adjust for pan so the same world coordinates stays under the cursor
        // Formula: pan = mouse_position - (world_position * new_zoom)
        let new_pan_x = mouse_x - zoom_ratio * (mouse_x - current_pan_x);
        let new_pan_y = mouse_y - zoom_ratio * (mouse_y - current_pan_y);

        // let new_pan_x = current_pan_x + (mouse_x - current_pan_x) * (1.0 - zoom_ratio);
        // let new_pan_y = current_pan_y + (mouse_y - current_pan_y) * (1.0 - zoom_ratio);

        // tracing::info!("=== ZOOM DEBUG ===");
        // tracing::info!(
        //     "Mouse: ({}, {}), Old zoom: {}, New zoom: {}",
        //     mouse_x,
        //     mouse_y,
        //     old_zoom,
        //     new_zoom
        // );
        // tracing::info!("Zoom ratio: {}", zoom_ratio);
        // tracing::info!(
        //     "Old pan: ({}, {}), New pan: ({}, {})",
        //     pan_x(),
        //     pan_y(),
        //     new_pan_x,
        //     new_pan_y
        // );

        //Apply new zoom & pan
        pan_x.set(new_pan_x);
        pan_y.set(new_pan_y);
        zoom.set(new_zoom);
    };

    //Start Panning
    let onmousedown = move |evt: Event<MouseData>| {
        is_panning.set(true);
        let coords = evt.client_coordinates();
        last_x.set(coords.x);
        last_y.set(coords.y);
    };

    let onmousemove = move |evt: Event<MouseData>| {
        //User is moving mouse - update pan if dragging
        if is_panning() {
            //Get current mouse position
            let coords = evt.client_coordinates();
            let current_x = coords.x;
            let current_y = coords.y;

            //Calculate how far the mouse has moved
            let dx = current_x - last_x();
            let dy = current_y - last_y();

            //Update pan position
            pan_x.set(pan_x() + dx);
            pan_y.set(pan_y() + dy);

            //Remember the last position
            last_x.set(current_x);
            last_y.set(current_y);
        }
    };

    //Release pan
    let onmouseup = move |_evt: Event<MouseData>| {
        //User has released mouse button - stop panning
        is_panning.set(false);
    };

    //Release pan
    let onmouseleave = move |_evt: Event<MouseData>| {
        //User has released mouse button - stop panning
        is_panning.set(false);
    };

    // -------------------------------------------------------------------------

    rsx! {
        div { class: "top-edge-mask" }
        div {
            class: "canvas-page",

            // Navbar
            CanvasNavbar {
                task_id: task_id.clone(),
                project_id: project_id.clone(),
                selected_tool: selected_tool
            }

            div{
                //Canvas container (viewport has fixed dots)
                class:container_class,
            style:format_args!("
                --dot-spacing: {}px;
                --dot-radius: {}px;
                --dot-size-scale: {};
                --dot-spacing-scale: {};
                --dot-offset-x: {}px;
                --dot-offset-y: {}px;
            ",
                dot_spacing_px,
                dot_radius_px,
                zoom().clamp(1.0,4.0), //Dot size
                zoom().clamp(1.0, 5.0), //Dot spacing
                -pan_x(),
                -pan_y(),
            ),
            onwheel:onwheel,
            onmousedown:onmousedown,
            onmouseup:onmouseup,
            onmousemove:onmousemove,
            onmouseleave:onmouseleave,

            div{
                //Canvas world (zooms/pans)
                class:"canvas-world",
                style: format_args!("
                    transform: translate({}px, {}px) scale({});
                    ",
                    pan_x(), pan_y(), zoom(),
                ),


                // Layer 1: Background Dots - GPU-accelerated
                // div{ class: "canvas-layer canvas-dots"}

                // Layer 2: Image - GPU-accelerated (starts at 60% width, centered)
                div {
                    class: "canvas-layer canvas-image",
                    img {src: CANVAS_IMG}
                }

                // Layer 3: Annotations - CPU - Scales with world
                canvas{
                    id:"canvas-annotations",
                    class:"canvas-layer canvas-annotations",
                    style: "background-color: transparent;",
                }
            }
        }
        }
    }
}

// Canvas layer kept for future annotations
