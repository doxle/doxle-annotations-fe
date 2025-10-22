use super::annotations::comment::Comment;
use super::annotations::shapes::BBox;
use super::annotations::shapes::Polygon;
use super::annotations::Tool;
use super::annotations::{AnnotationDropdown, AnnotationTarget};
use super::canvas_navbar::CanvasNavbar;
use super::cursor_state::CursorState;
use super::image_utils::cache_image_world_bounds;
use super::sidebar::Sidebar;
use dioxus::prelude::*;

#[component]
pub fn CanvasPage(task_id: String) -> Element {
    let project_id = "1".to_string(); // TODO: Parse from task_id or pass as separate param
    let mut zoom = use_signal(|| 1.0);
    let mut pan_x = use_signal(|| 0.0);
    let mut pan_y = use_signal(|| 0.0);
    let mut selected_tool = use_signal(|| Tool::Select); // Default is Select (Arrow)
    let mut avatar_dropdown_open = use_signal(|| false);
    let mut show_grid_lines = use_signal(|| true); // Grid lines enabled by default
    let mut sidebar_open = use_signal(|| false);
    let mut is_panning: Signal<bool> = use_signal(|| false);
    let mut last_x: Signal<f64> = use_signal(|| 0.0);
    let mut last_y: Signal<f64> = use_signal(|| 0.0);
    let mut polygon = use_signal(Polygon::new);
    let mut bbox = use_signal(BBox::new);
    let mut comment = use_signal(Comment::new);
    let dot_spacing_px: f64 = 12.0;
    let dot_radius_px: f64 = 1.0;

    // Clone for use inside closures without moving the original into handlers
    let project_id_for_ann = project_id.clone();
    // TODO: wire from route/selection
    let block_id_for_ann = "block1".to_string();
    let image_id_for_ann = "house1".to_string();
    let mut class_counter = use_signal(|| 0_u64);

    // --- Annotation dropdown state ---
    let mut annotation_dropdown_open = use_signal(|| false);
    let mut annotation_dropdown_x = use_signal(|| 0.0);
    let mut annotation_dropdown_y = use_signal(|| 0.0);
    let mut annotation_dropdown_target = use_signal(|| None as Option<AnnotationTarget>);

    // --- Hovered annotation state for delete on hover and visual feedback ---
    let mut hovered_annotation = use_signal(|| None as Option<AnnotationTarget>);

    // Setup all canvas effects
    super::canvas_effects::setup_annotation_redraw_effect(
        project_id_for_ann.clone(),
        block_id_for_ann.clone(),
        image_id_for_ann.clone(),
        class_counter,
        zoom,
        pan_x,
        pan_y,
    );
    super::canvas_effects::setup_canvas_size_effect();
    super::canvas_effects::setup_tool_switch_effect(selected_tool, polygon, bbox, comment);

    // Keyboard shortcuts handler
    super::keyboard_shortcuts::setup_keyboard_shortcuts(
        project_id_for_ann.clone(),
        block_id_for_ann.clone(),
        image_id_for_ann.clone(),
        selected_tool,
        polygon,
        bbox,
        zoom,
        pan_x,
        pan_y,
        sidebar_open,
        class_counter,
        hovered_annotation,
    );

    // Cursor state for custom crosshair
    let cursor_state = CursorState::new();

    let container_class = if is_panning() {
        if show_grid_lines() && selected_tool().is_drawing_tool() {
            "canvas-container is-panning polygon-tool-active"
        } else if show_grid_lines() {
            "canvas-container is-panning polygon-tool-active"
        } else {
            "canvas-container is-panning"
        }
    } else if selected_tool() == Tool::Comment {
        "canvas-container comment-tool-active"
    } else if selected_tool() == Tool::Polygon || selected_tool() == Tool::BoundingBox {
        if show_grid_lines() {
            "canvas-container polygon-tool-active"
        } else {
            "canvas-container polygon-tool-active-no-grid"
        }
    } else if selected_tool() == Tool::Pan {
        "canvas-container pan-tool-active"
    } else {
        "canvas-container"
    };

    rsx! {
        div { class: "top-edge-mask" }
        div {
            class: "canvas-page",

            // Navbar
            CanvasNavbar {
                task_id: task_id.clone(),
                project_id: project_id.clone(),
                selected_tool: selected_tool,
                avatar_dropdown_open: avatar_dropdown_open,
                show_grid_lines: show_grid_lines,
                sidebar_open: sidebar_open,
                polygon: polygon,
                bbox: bbox
            }
            div{
                //Canvas container (viewport has fixed dots)
                id: "canvas-container",
                class:container_class,
                style:format_args!(
                "
                --dot-spacing: {}px;
                --dot-radius: {}px;
                --dot-size-scale: {};
                --dot-spacing-scale: {};
                --dot-offset-x: {}px;
                --dot-offset-y: {}px;
                overscroll-behavior: contain;
                touch-action: none;
            ",
                dot_spacing_px,
                dot_radius_px,
                zoom().clamp(1.0,4.0), //Dot size
                zoom().clamp(1.0, 5.0), //Dot spacing
                -pan_x(),
                -pan_y(),
            ),

            onclick: move |_evt| {
                if avatar_dropdown_open() {
                    avatar_dropdown_open.set(false);
                }
            },
            onwheel: move |evt: Event<WheelData>| {
                super::mouse_handlers::handle_wheel(&evt, annotation_dropdown_open, zoom, pan_x, pan_y, polygon, bbox);
            },
            onmousedown: {let pid = project_id_for_ann.clone(); let bid = block_id_for_ann.clone(); let iid = image_id_for_ann.clone(); move |evt: Event<MouseData>| {
                super::mouse_handlers::handle_mousedown(
                    &evt, annotation_dropdown_open, selected_tool, is_panning, last_x, last_y,
                    zoom, pan_x, pan_y, polygon, bbox, class_counter,
                    pid.clone(), bid.clone(), iid.clone(),
                );
            }},
            onmousemove: {let pid = project_id_for_ann.clone(); let bid = block_id_for_ann.clone(); let iid = image_id_for_ann.clone(); move |evt: Event<MouseData>| {
                super::mouse_handlers::handle_mousemove(
                    &evt, annotation_dropdown_open, selected_tool, is_panning, last_x, last_y,
                    cursor_state, zoom, pan_x, pan_y, polygon, bbox,
                    hovered_annotation, pid.clone(), bid.clone(), iid.clone(),
                );
            }},
            onmouseup: move |evt: Event<MouseData>| {
                super::mouse_handlers::handle_mouseup(&evt, annotation_dropdown_open, is_panning);
            },
            onmouseleave: move |evt: Event<MouseData>| {
                super::mouse_handlers::handle_mouseleave(&evt, annotation_dropdown_open, is_panning);
            },
            oncontextmenu: {let pid = project_id_for_ann.clone(); let bid = block_id_for_ann.clone(); let iid = image_id_for_ann.clone(); move |evt: Event<MouseData>| {
                super::mouse_handlers::handle_annotation_dropdown(
                    &evt, annotation_dropdown_open, annotation_dropdown_x, annotation_dropdown_y,
                    annotation_dropdown_target, zoom, pan_x, pan_y,
                    pid.clone(), bid.clone(), iid.clone(),
                );
            }},

            // Canvas world (zooms/pans) - only image layer
            div{
                class:"canvas-world",
                style: format_args!("
                    transform: translate({}px, {}px) scale({});
                    ",
                    pan_x(), pan_y(), zoom(),
                ),

                // Background image layer only
                div {
                    class: "canvas-layer canvas-image",
                    img {
                        src: asset!("/assets/images/test.png") ,
                        onload:move|_|{
                            tracing::info!("Image loaded");
                            // Cache image world bounds
                            cache_image_world_bounds(zoom(), pan_x(), pan_y());
                        }
                    }
                }
            }

            // Canvas-A: Saved annotations (background)
            canvas {
                id: "canvas-saved",
                class: "canvas-layer canvas-saved",
                style: "background-color: transparent; position: absolute; pointer-events: none;",
            }

            // Canvas-B: In-progress overlay (foreground)
            canvas {
                id: "canvas-overlay",
                class: if selected_tool().is_drawing_tool() {
                    "canvas-layer canvas-overlay tool-active"
                } else {
                    "canvas-layer canvas-overlay"
                },
                style: "background-color: transparent; position: absolute; pointer-events: none;",
            }

            // Custom crosshair cursor overlay
            if selected_tool() == Tool::Polygon || selected_tool() == Tool::BoundingBox {
                div {
                    class: "crosshair-cursor",
                    // Center rectangle
                    div {
                        class: "crosshair-center"
                    }
                }

            }

            // Annotation dropdown component
            if annotation_dropdown_open() && annotation_dropdown_target().is_some() {
                AnnotationDropdown {
                    open: annotation_dropdown_open,
                    x: annotation_dropdown_x,
                    y: annotation_dropdown_y,
                    target: annotation_dropdown_target,
                    project_id: project_id.clone(),
                    block_id: block_id_for_ann.clone(),
                    image_id: image_id_for_ann.clone(),
                    zoom: zoom,
                    pan_x: pan_x,
                    pan_y: pan_y,
                    class_counter: class_counter,
                }
            }
            }

            // Sidebar
            Sidebar { task_id: task_id.clone(), project_id: project_id.clone(), sidebar_open: sidebar_open, class_counter: class_counter }

        }
    }
}

// Canvas layer kept for future annotations
