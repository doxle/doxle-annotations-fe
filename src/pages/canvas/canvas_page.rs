use super::annotations::polygon::on_polygon_click;
use super::annotations::polygon::{get_canvas_context, Polygon};
use super::annotations::bbox::{on_bbox_click, redraw_bbox, BBox};
use super::annotations::comment::{on_comment_click, redraw_comment, Comment};
use super::annotations::Tool;
use super::canvas_navbar::CanvasNavbar;
use crate::dioxus_elements::input_data::MouseButton;
use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};

const NAVBAR_H: f64 = 36.0; // canvas container sits below navbar

#[component]
pub fn CanvasPage(task_id: String) -> Element {
    let project_id = "1".to_string(); // TODO: Parse from task_id or pass as separate param
    let mut zoom = use_signal(|| 1.0);
    let mut pan_x = use_signal(|| 0.0);
    let mut pan_y = use_signal(|| 0.0);
    let selected_tool = use_signal(|| Tool::Select); // Default is Select (Arrow)
    let mut avatar_menu_open = use_signal(|| false);
    let mut show_grid_lines = use_signal(|| false);
    let mut is_panning: Signal<bool> = use_signal(|| false);
    let mut last_x: Signal<f64> = use_signal(|| 0.0);
    let mut last_y: Signal<f64> = use_signal(|| 0.0);
    let mut polygon = use_signal(Polygon::new);
    let mut bbox = use_signal(BBox::new);
    let mut comment = use_signal(Comment::new);
    let dot_spacing_px: f64 = 12.0;
    let dot_radius_px: f64 = 1.0;

    // Set canvas size with DPR for crisp rendering
    use_effect(move || {
        if let Some(window) = window() {
            if let Some(document) = window.document() {
                if let Some(canvas_el) = document.get_element_by_id("canvas-annotations") {
                    if let Ok(canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() {
                        let css_w = window
                            .inner_width()
                            .ok()
                            .and_then(|w| w.as_f64())
                            .unwrap_or(1920.0);
                        let css_h = window
                            .inner_height()
                            .ok()
                            .and_then(|h| h.as_f64())
                            .map(|h| h - 36.0)
                            .unwrap_or(1080.0);
                        let dpr = window.device_pixel_ratio();

                        // Set internal buffer size (with DPR for crisp rendering)
                        canvas.set_width((css_w * dpr).round() as u32);
                        canvas.set_height((css_h * dpr).round() as u32);

                        // Set CSS display size
                        let _ = canvas
                            .style()
                            .set_property("width", &format!("{}px", css_w));
                        let _ = canvas
                            .style()
                            .set_property("height", &format!("{}px", css_h));

                        // Scale the context by DPR so we can draw in CSS pixels
                        if let Ok(Some(ctx_any)) = canvas.get_context("2d") {
                            if let Ok(ctx) = ctx_any.dyn_into::<CanvasRenderingContext2d>() {
                                let _ = ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);
                            }
                        }

                        tracing::info!(
                            "Canvas sized: {}x{} CSS, buffer: {}x{}, DPR: {}",
                            css_w,
                            css_h,
                            (css_w * dpr) as u32,
                            (css_h * dpr) as u32,
                            dpr
                        );
                    }
                }
            }
        }
    });

    // Redraw when polygon or bbox state changes (for undo/redo)
    use_effect(move || {
        let _ = polygon();
        let _ = bbox();
        
        if let Some(ctx) = get_canvas_context() {
            match selected_tool() {
                Tool::Polygon => {
                    if polygon().points.len() > 0 {
                        super::annotations::polygon::redraw_polygon(
                            &ctx,
                            &polygon(),
                            zoom(),
                            pan_x(),
                            pan_y(),
                        );
                    }
                }
                Tool::BoundingBox => {
                    if bbox().start_point.is_some() {
                        redraw_bbox(
                            &ctx,
                            &bbox(),
                            zoom(),
                            pan_x(),
                            pan_y(),
                        );
                    }
                }
                _ => {}
            }
        }
    });
    
    // Clear annotations when switching tools
    use_effect(move || {
        if let Some(ctx) = get_canvas_context() {
            // Clear canvas
            if let Some(canvas) = ctx.canvas() {
                let window = window().expect("Should get window");
                let dpr = window.device_pixel_ratio();
                let css_w = canvas.width() as f64 / dpr;
                let css_h = canvas.height() as f64 / dpr;
                ctx.clear_rect(0.0, 0.0, css_w, css_h);
            }

            // When switching tools, clear the previous tool's annotation
            // TODO: Save to database before clearing
            match selected_tool() {
                Tool::Select | Tool::Pan => {
                    // Clear all annotations when switching to Select or Pan
                    polygon.write().points.clear();
                    polygon.write().is_closed = false;
                    polygon.write().preview_point = None;
                    bbox.write().reset();
                    comment.write().reset();
                }
                Tool::Polygon => {
                    // Clear bbox and comment when switching to Polygon
                    bbox.write().reset();
                    comment.write().reset();
                }
                Tool::BoundingBox => {
                    // Clear polygon and comment when switching to BBox
                    polygon.write().points.clear();
                    polygon.write().is_closed = false;
                    polygon.write().preview_point = None;
                    comment.write().reset();
                }
                Tool::Comment => {
                    // Clear polygon and bbox when switching to Comment
                    polygon.write().points.clear();
                    polygon.write().is_closed = false;
                    polygon.write().preview_point = None;
                    bbox.write().reset();
                }
            }
        }
    });

    // --- Cursor State ---
    let mut guide_x = use_signal(|| 0.0);
    let mut guide_y = use_signal(|| 0.0);
    let mut cursor_x = use_signal(|| 0.0);
    let mut cursor_y = use_signal(|| 0.0);

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

    // --- Handlers ----

    // cursor-centered-zoom
    let onwheel = move |evt: Event<WheelData>| {
        evt.prevent_default();
        let mouse = evt.client_coordinates();
        let dy = evt.data.delta().strip_units().y;
        let factor = if dy < 0.0 { 1.2 } else { 0.8 };
        let old = zoom();
        let new = (old * factor as f64).clamp(0.3, 50.0);
        let r = new / old;

        // keep cursor position stable visually
        // Convert viewport coords to container coords (container starts at NAVBAR_H)
        let mx = mouse.x;
        let my = mouse.y - NAVBAR_H;
        let (px, py) = (pan_x(), pan_y());

        let new_pan_x = mx - r * (mx - px);
        let new_pan_y = my - r * (my - py);

        //Apply new zoom & pan
        pan_x.set(new_pan_x);
        pan_y.set(new_pan_y);
        zoom.set(new);

        // Redraw only the active tool's annotation
        if let Some(ctx) = get_canvas_context() {
            match selected_tool() {
                Tool::Polygon => {
                    if polygon().points.len() > 0 {
                        super::annotations::polygon::redraw_polygon(
                            &ctx,
                            &polygon(),
                            new,
                            new_pan_x,
                            new_pan_y,
                        );
                    }
                }
                Tool::BoundingBox => {
                    if bbox().start_point.is_some() {
                        redraw_bbox(
                            &ctx,
                            &bbox(),
                            new,
                            new_pan_x,
                            new_pan_y,
                        );
                    }
                }
                Tool::Comment => {
                    if comment().position.is_some() {
                        redraw_comment(
                            &ctx,
                            &comment(),
                            new,
                            new_pan_x,
                            new_pan_y,
                        );
                    }
                }
                Tool::Select | Tool::Pan => {
                    // No active annotation to redraw
                }
            }
        }
    };

    // start pan or add polygon vertex (NO world conversion)
    let onmousedown = move |evt: Event<MouseData>| {
        let coords = evt.client_coordinates();

        // middle mouse -> pan (keeping existing logic)
        if evt.data.trigger_button() == Some(MouseButton::Auxiliary) {
            is_panning.set(true);
            last_x.set(coords.x);
            last_y.set(coords.y);
            return;
        }

        // Pan tool with left mouse button -> pan
        if selected_tool() == Tool::Pan && evt.data.trigger_button() == Some(MouseButton::Primary) {
            is_panning.set(true);
            last_x.set(coords.x);
            last_y.set(coords.y);
            return;
        }

        // Tool-based handling
        if selected_tool() == Tool::Polygon {
            // Convert screen coordinates to world coordinates
            let screen_x = coords.x;
            let screen_y = coords.y - NAVBAR_H;

            // Apply inverse transform: (screen - pan) / zoom
            let world_x = (screen_x - pan_x()) / zoom();
            let world_y = (screen_y - pan_y()) / zoom();

            if let Some(ctx) = get_canvas_context() {
                // Check if clicking near first point to close
                let mut poly = polygon.write();
                if poly.points.len() >= 3 {
                    if let Some(first) = poly.points.first() {
                        let first_screen_x = first.x * zoom() + pan_x();
                        let first_screen_y = first.y * zoom() + pan_y();
                        let dx = screen_x - first_screen_x;
                        let dy = screen_y - first_screen_y;
                        let distance = (dx * dx + dy * dy).sqrt();

                        if distance < 30.0 {
                            // Close the polygon
                            poly.close();
                            on_polygon_click(
                                world_x,
                                world_y,
                                &mut poly,
                                &ctx,
                                zoom(),
                                pan_x(),
                                pan_y(),
                            );
                            return;
                        }
                    }
                }
                on_polygon_click(world_x, world_y, &mut poly, &ctx, zoom(), pan_x(), pan_y());
            }
        }

        if selected_tool() == Tool::BoundingBox {
            // Convert screen coordinates to world coordinates
            let screen_x = coords.x;
            let screen_y = coords.y - NAVBAR_H;

            // Apply inverse transform: (screen - pan) / zoom
            let world_x = (screen_x - pan_x()) / zoom();
            let world_y = (screen_y - pan_y()) / zoom();

            if let Some(ctx) = get_canvas_context() {
                let mut bb = bbox.write();
                on_bbox_click(world_x, world_y, &mut bb, &ctx, zoom(), pan_x(), pan_y());
            }
        }
    };

    // pan drag and preview line tracking
    let onmousemove = move |evt: Event<MouseData>| {
        let coords = evt.client_coordinates();

        // Handle panning
        if is_panning() {
            let dx = coords.x - last_x();
            let dy = coords.y - last_y();
            let new_pan_x = pan_x() + dx;
            let new_pan_y = pan_y() + dy;
            pan_x.set(new_pan_x);
            pan_y.set(new_pan_y);
            last_x.set(coords.x);
            last_y.set(coords.y);

            // Redraw only the active tool's annotation
            if let Some(ctx) = get_canvas_context() {
                match selected_tool() {
                    Tool::Polygon => {
                        if polygon().points.len() > 0 {
                            super::annotations::polygon::redraw_polygon(
                                &ctx,
                                &polygon(),
                                zoom(),
                                new_pan_x,
                                new_pan_y,
                            );
                        }
                    }
                    Tool::BoundingBox => {
                        if bbox().start_point.is_some() {
                            redraw_bbox(
                                &ctx,
                                &bbox(),
                                zoom(),
                                new_pan_x,
                                new_pan_y,
                            );
                        }
                    }
                    Tool::Comment => {
                        if comment().position.is_some() {
                            redraw_comment(
                                &ctx,
                                &comment(),
                                zoom(),
                                new_pan_x,
                                new_pan_y,
                            );
                        }
                    }
                    Tool::Select | Tool::Pan => {}
                }
            }
            return;
        }

        // Update preview line and guide lines when drawing polygon
        if selected_tool() == Tool::Polygon {
            // Update cursor positions (no snapping - blazing fast)
            guide_x.set(coords.x);
            guide_y.set(coords.y - NAVBAR_H);
            cursor_x.set(coords.x);
            cursor_y.set(coords.y - NAVBAR_H); // Match guide_y to align with canvas

            if polygon().points.len() > 0 && !polygon().is_closed {
                // Convert screen to world coords for preview
                let screen_x = coords.x;
                let screen_y = coords.y - NAVBAR_H;
                let world_x = (screen_x - pan_x()) / zoom();
                let world_y = (screen_y - pan_y()) / zoom();

                polygon
                    .write()
                    .set_preview(Some(super::annotations::polygon::Point {
                        x: world_x,
                        y: world_y,
                    }));

                if let Some(ctx) = get_canvas_context() {
                    super::annotations::polygon::redraw_polygon(
                        &ctx,
                        &polygon(),
                        zoom(),
                        pan_x(),
                        pan_y(),
                    );
                }
            }
        }

        // Update preview box when drawing bbox
        if selected_tool() == Tool::BoundingBox {
            // Update cursor positions
            guide_x.set(coords.x);
            guide_y.set(coords.y - NAVBAR_H);
            cursor_x.set(coords.x);
            cursor_y.set(coords.y - NAVBAR_H);

            if bbox().start_point.is_some() && !bbox().is_complete {
                // Convert screen to world coords for preview
                let screen_x = coords.x;
                let screen_y = coords.y - NAVBAR_H;
                let world_x = (screen_x - pan_x()) / zoom();
                let world_y = (screen_y - pan_y()) / zoom();

                bbox
                    .write()
                    .set_preview(Some(super::annotations::bbox::Point {
                        x: world_x,
                        y: world_y,
                    }));

                if let Some(ctx) = get_canvas_context() {
                    redraw_bbox(
                        &ctx,
                        &bbox(),
                        zoom(),
                        pan_x(),
                        pan_y(),
                    );
                }
            }
        }
    };

    //release pan
    let onmouseup = move |_evt: Event<MouseData>| {
        is_panning.set(false);
    };

    //release pan
    let onmouseleave = move |_evt: Event<MouseData>| {
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
                selected_tool: selected_tool,
                avatar_menu_open: avatar_menu_open,
                show_grid_lines: show_grid_lines,
                polygon: polygon,
                bbox: bbox
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
                --guide-x: {}px;
                --guide-y: {}px;
            ",
                dot_spacing_px,
                dot_radius_px,
                zoom().clamp(1.0,4.0), //Dot size
                zoom().clamp(1.0, 5.0), //Dot spacing
                -pan_x(),
                -pan_y(),
                guide_x(),
                guide_y(),
            ),

            onclick: move |_evt| {
                if avatar_menu_open() {
                    avatar_menu_open.set(false);
                }
                // Don't stop propagation - let mousedown handle it
            },
            onwheel:onwheel,
            onmousedown:onmousedown,
            onmouseup:onmouseup,
            onmousemove:onmousemove,
            onmouseleave:onmouseleave,

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
                    img { src: asset!("/assets/images/test.png") }
                }
            }

            // Annotation canvas - outside transform for crisp rendering
            canvas {
                id: "canvas-annotations",
                class: if selected_tool().is_drawing_tool() {
                    "canvas-layer canvas-annotations tool-active"
                } else {
                    "canvas-layer canvas-annotations"
                },
                style: "background-color: transparent;",
            }

            // Custom crosshair cursor overlay
            if selected_tool() == Tool::Polygon || selected_tool() == Tool::BoundingBox {
                div {
                    class: "crosshair-cursor",
                    style: format_args!("--cursor-x: {}px; --cursor-y: {}px;", cursor_x(), cursor_y()),
                    // Center rectangle
                    div {
                        class: "crosshair-center"
                    }
                }

            }
        }
        }
    }
}

// Canvas layer kept for future annotations
