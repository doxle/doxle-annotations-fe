use dioxus::prelude::*;

#[component]
pub fn CanvasPage(task_id: ReadSignal<String>) -> Element {
    // --- State ---
    let mut zoom = use_signal(|| 1.0);
    let mut _pan_x = use_signal(|| 0.0);
    let mut _pan_y = use_signal(|| 0.0);

    // --- Pan ----
    let mut _is_panning: Signal<bool> = use_signal(|| false);
    let mut _last_x: Signal<f64> = use_signal(|| 0.0);
    let mut _last_y: Signal<f64> = use_signal(|| 0.0);

    // --- Assets ----
    const CANVAS_IMG: Asset = asset!("/assets/images/test.png");

    // --- Dots & Theme ---
    let dot_spacing_px: f64 = 12.0;
    let dot_radius_px: f64 = 1.0;
    let dots_style = format!(
        "pointer-events: none; background: radial-gradient(circle, var(--dot-color) 0px, var(--dot-color) {r}px, transparent {r}px); background-size: {s}px {s}px; background-position: 0 0;",
        r = dot_radius_px,
        s = dot_spacing_px,
    );

    // -------------------------------------------------------------------------

    // --- Handlers ----

    let onwheel = move |evt: Event<WheelData>| {
        evt.prevent_default();
        let delta = evt.data.delta().strip_units().y;
        tracing::info!("delta : {}", delta);
        let zoom_factor = if delta < 0.0 { 1.1 } else { 0.9 };
        let new_zoom = ((zoom() * zoom_factor) as f64).clamp(0.1, 5.0);
        zoom.set(new_zoom);
    };

    // -------------------------------------------------------------------------

    rsx! {
        div{
            class:"canvas-container",
            style:"display:grid; place-items:center;",
            onwheel:onwheel,

            div{
                class:"canvas-world",
                style: format_args!("
                    position:relative;
                    width:100%;
                    height:100%;
                    display:grid;
                    place-items:center;
                    transform-origin:center center;
                    will-change: transform;
                    transform: scale({});
                    ",zoom()),


                // Layer 1: Background Dots - GPU-accelerated
                div{
                    class: "canvas-layer canvas-dots",
                    style: format_args!("{}",dots_style),
                }

                // Layer 2: Image - GPU-accelerated (starts at 60% width, centered)
                div {
                    class: "canvas-layer canvas-image",
                    style: "display: grid; place-items: center;",
                    img {
                        src: CANVAS_IMG,
                        style: "
                        display: block;
                        width: 60%; /* intial size = 60% of canvas */
                        height: auto;
                        max-width: none;
                        user-select:none;
                        pointer-events:none;
                        image-rendering: -webkit-optimize-contrast;
                        image-rendering: crisp-edges;
                        "
                    }
                }

                // Layer 3: Annotations - CPU - Scales with world
                canvas{
                    id:"dots-canvas",
                    class:"canvas-layer canvas-annotations",
                    style: "background-color: transparent;",
                }
            }
        }
    }
}

// Canvas layer kept for future annotations
