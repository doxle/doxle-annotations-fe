use dioxus::prelude::*;

#[component]
pub fn CanvasPage(task_id: ReadSignal<String>) -> Element {
    // --- State ---
    let mut zoom = use_signal(|| 1.0);
    let mut pan_x = use_signal(|| 0.0);
    let mut pan_y = use_signal(|| 0.0);

    // --- Pan ----
    let mut is_panning: Signal<bool> = use_signal(|| false);
    let mut last_x: Signal<f64> = use_signal(|| 0.0);
    let mut last_y: Signal<f64> = use_signal(|| 0.0);

    // --- Assets ----
    const CANVAS_IMG: Asset = asset!("/assets/images/test.png");

    // --- Dots & Theme ---
    let dot_spacing_px: f64 = 12.0;
    let dot_radius_px: f64 = 1.0;

    // let dots_style = format!(
    //     "pointer-events: none; background: radial-gradient(circle, var(--dot-color) 0px, var(--dot-color) {r}px, transparent {r}px); background-size: {s}px {s}px; background-position: 0 0;",
    //     r = dot_radius_px,
    //     s = dot_spacing_px,
    // );

    // -------------------------------------------------------------------------

    // --- Handlers ----

    //Zoom
    let onwheel = move |evt: Event<WheelData>| {
        evt.prevent_default();
        let delta = evt.data.delta().strip_units().y;
        tracing::info!("delta : {}", delta);
        let zoom_factor = if delta < 0.0 { 1.1 } else { 0.9 };
        let new_zoom = ((zoom() * zoom_factor) as f64).clamp(0.1, 5.0);
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

    // -------------------------------------------------------------------------

    rsx! {
        div{
            //Canvas container (viewport has fixed dots)
            class:"canvas-container",
            style:format_args!("
                --dot-spacing: {}px;
                --dot-radius: {}px;
                --dot-scale: {};
                --dot-offset-x: {}px;
                --dot-offset-y: {}px;
            ",
                dot_spacing_px,
                dot_radius_px,
                zoom().max(0.4),
                pan_x()*zoom(),
                pan_y()*zoom(),
            ),
            onwheel:onwheel,
            onmousedown:onmousedown,
            onmouseup:onmouseup,
            onmousemove:onmousemove,

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

// Canvas layer kept for future annotations
