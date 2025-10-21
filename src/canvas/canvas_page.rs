use super::annotations::polygon::on_polygon_click;
use super::annotations::polygon::{Polygon, Point};
use super::annotations::bbox::{on_bbox_click, redraw_bbox, BBox};
use super::annotations::comment::Comment;
use super::annotations::Tool;
use super::canvas_navbar::CanvasNavbar;
use super::sidebar::Sidebar;
use super::sidebar::ClassItem;
use super::sidebar::storage::{get_active_or_first_class_id, increment_class_count, load_json as load_json_any};
use super::annotations::store::{save_polygon, load_polygons, load_bboxes, update_polygon_class, delete_polygon_at, update_bbox_class, delete_bbox_at};
use super::annotations::saved_canvas::{redraw_saved_annotations, invalidate_polygon_cache, invalidate_bbox_cache};
use super::annotations::overlay_canvas::{schedule_overlay_redraw, get_overlay_canvas_context};
use super::annotations::{AnnotationMenu, AnnotationTarget};
use crate::dioxus_elements::input_data::MouseButton;
use dioxus::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, KeyboardEvent};

const NAVBAR_H: f64 = 36.0; // canvas container sits below navbar

// Compute the displayed image bounds in WORLD coordinates (after undoing pan/zoom).
fn get_image_bounds_world(zoom: f64, pan_x: f64, pan_y: f64) -> Option<(f64, f64, f64, f64)> {
    let win = window()?;
    let doc = win.document()?;
    let el = doc.query_selector(".canvas-image img").ok().flatten()?;
    let rect = el.get_bounding_client_rect();
    let left = (rect.left() - pan_x) / zoom;
    let top = (rect.top() - NAVBAR_H - pan_y) / zoom;
    let right = (rect.right() - pan_x) / zoom;
    let bottom = (rect.bottom() - NAVBAR_H - pan_y) / zoom;
    tracing::info!("Image bounds - Screen rect: L={} T={} R={} B={}, World bounds: L={} T={} R={} B={}", 
        rect.left(), rect.top(), rect.right(), rect.bottom(), left, top, right, bottom);
    Some((left, top, right, bottom))
}

// Check if a viewport point lies inside the displayed image rect (SCREEN coordinates).
fn point_inside_image(screen_x: f64, screen_y: f64) -> bool {
    if let Some(win) = window() {
        if let Some(doc) = win.document() {
            if let Ok(Some(el)) = doc.query_selector(".canvas-image img") {
                let r = el.get_bounding_client_rect();
                return screen_x >= r.left()
                    && screen_x <= r.right()
                    && screen_y >= r.top()
                    && screen_y <= r.bottom();
            }
        }
    }
    true // If we can't detect the image, allow input (fallback)
}

fn clamp_f64(v: f64, min: f64, max: f64) -> f64 { v.max(min).min(max) }

// Helper for backwards compat - returns overlay context
fn get_canvas_context() -> Option<CanvasRenderingContext2d> {
    get_overlay_canvas_context()
}

#[component]
pub fn CanvasPage(task_id: String) -> Element {
    let project_id = "1".to_string(); // TODO: Parse from task_id or pass as separate param
    let mut zoom = use_signal(|| 1.0);
    let mut pan_x = use_signal(|| 0.0);
    let mut pan_y = use_signal(|| 0.0);
    let mut selected_tool = use_signal(|| Tool::Select); // Default is Select (Arrow)
    let mut avatar_menu_open = use_signal(|| false);
    let mut show_grid_lines = use_signal(|| false);
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
    let mut classes_version = use_signal(|| 0_u64);

    // --- Annotation menu state ---
    let mut annotation_menu_open = use_signal(|| false);
    let mut annotation_menu_x = use_signal(|| 0.0);
    let mut annotation_menu_y = use_signal(|| 0.0);
    let mut annotation_menu_target = use_signal(|| None as Option<AnnotationTarget>);
    
    // Note: preview_point is managed within polygon state, not as separate signal

    // Redraw saved canvas when polygons change OR zoom/pan changes
    // (rAF throttling in redraw_saved_annotations prevents jitter)
    let pid_for_saved = project_id_for_ann.clone();
    let block_for_saved = block_id_for_ann.clone();
    let image_for_saved = image_id_for_ann.clone();
    use_effect(move || {
        tracing::info!("use_effect REDRAW triggered - classes_version={}, zoom={}, pan_x={}, pan_y={}", 
            classes_version(), zoom(), pan_x(), pan_y());
        
        redraw_saved_annotations(&pid_for_saved, &block_for_saved, &image_for_saved, zoom(), pan_x(), pan_y());
    });

    // Set canvas size with DPR for crisp rendering (both canvases)
    use_effect(move || {
        if let Some(win) = window() {
            if let Some(document) = win.document() {
                // Measure the actual canvas container instead of window to avoid rounding/clipping on the right edge
                let (css_w, css_h) = if let Some(container) = document.get_element_by_id("canvas-container") {
                    let rect = container.get_bounding_client_rect();
                    (rect.width(), rect.height())
                } else {
                    // Fallback to window size if container not found
                    (
                        win.inner_width().ok().and_then(|w| w.as_f64()).unwrap_or(1920.0),
                        win.inner_height().ok().and_then(|h| h.as_f64()).map(|h| h - NAVBAR_H).unwrap_or(1080.0),
                    )
                };
                let dpr = win.device_pixel_ratio();

                // Size both canvases to match container precisely
                for canvas_id in ["canvas-saved", "canvas-overlay"] {
                    if let Some(canvas_el) = document.get_element_by_id(canvas_id) {
                        if let Ok(canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() {
                            // Internal device buffer
                            canvas.set_width((css_w * dpr).round() as u32);
                            canvas.set_height((css_h * dpr).round() as u32);

                            // CSS display size
                            let _ = canvas.style().set_property("width", &format!("{}px", css_w));
                            let _ = canvas.style().set_property("height", &format!("{}px", css_h));

                            // Scale context so drawing uses CSS pixels
                            if let Ok(Some(ctx_any)) = canvas.get_context("2d") {
                                if let Ok(ctx) = ctx_any.dyn_into::<CanvasRenderingContext2d>() {
                                    let _ = ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);
                                }
                            }
                        }
                    }
                }

                tracing::info!(
                    "Canvases sized to container: {}x{} CSS, buffer: {}x{}, DPR: {}",
                    css_w,
                    css_h,
                    (css_w * dpr) as u32,
                    (css_h * dpr) as u32,
                    dpr
                );
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

    // Keyboard shortcuts handler
    let pid_for_keys = project_id_for_ann.clone();
    let pid_for_keys_val = pid_for_keys.clone();
    let mut classes_version_keys = classes_version.clone();
    use_effect(move || {
        let win = match window() {
            Some(w) => w,
            None => return,
        };

        let pid_value = pid_for_keys_val.clone();
        let block_value = "block1".to_string();
        let image_value = "house1".to_string();
        let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            let key = event.key();
            
            // Handle ESC key - clear annotations when drawing
            if key == "Escape" {
                match selected_tool() {
                    Tool::Polygon => {
                        if polygon().points.len() > 0 {
                            polygon.write().reset();
                        }
                        // Clear overlay immediately
                        schedule_overlay_redraw(polygon(), zoom(), pan_x(), pan_y());
                    }
                    Tool::BoundingBox => {
                        if bbox().start_point.is_some() {
                            bbox.write().reset();
                        }
                        // Clear overlay by redrawing (no start -> only clears)
                        if let Some(ctx) = get_canvas_context() {
                            redraw_bbox(&ctx, &bbox(), zoom(), pan_x(), pan_y());
                        }
                    }
                    _ => {}
                }
                return;
            }

            // Enter closes polygon and increments class count
            if key == "Enter" {
                if selected_tool() == Tool::Polygon {
                    if polygon().points.len() >= 3 && !polygon().is_closed {
                        polygon.write().close();
                        // Save and reset current polygon
                        let pid_local = pid_value.clone();
                        let block_local = block_value.clone();
                        let image_local = image_value.clone();
                        if let Some(class_id) = get_active_or_first_class_id(&pid_value) {
                            save_polygon(&pid_local, &block_local, &image_local, &polygon(), &class_id);
                            increment_class_count(&pid_local, &class_id, 1);
                            invalidate_polygon_cache(); // Force cache refresh
                            classes_version_keys.set(classes_version_keys() + 1); // Triggers saved canvas redraw
                        }
                        polygon.write().reset();
                        // Clear overlay immediately after closing
                        schedule_overlay_redraw(polygon(), zoom(), pan_x(), pan_y());
                        // Auto-switch to Select (arrow) tool
                        selected_tool.set(Tool::Select);
                    }
                }
                return;
            }

            // Handle tool shortcuts (p, b, c, a, h, \, t, f, n)
            match key.as_str() {
                "p" => selected_tool.set(Tool::Polygon),
                "b" => selected_tool.set(Tool::BoundingBox),
                "c" => selected_tool.set(Tool::Comment),
                "a" => selected_tool.set(Tool::Select),
                "h" => selected_tool.set(Tool::Pan),
                "f" => {
                    // Finish current annotation
                    match selected_tool() {
                        Tool::Polygon => {
                            if polygon().points.len() >= 3 && !polygon().is_closed {
                                polygon.write().close();
                                let pid_local = pid_value.clone();
                                let block_local = block_value.clone();
                                let image_local = image_value.clone();
                                if let Some(class_id) = get_active_or_first_class_id(&pid_value) {
                                    save_polygon(&pid_local, &block_local, &image_local, &polygon(), &class_id);
                                    increment_class_count(&pid_local, &class_id, 1);
                                    invalidate_polygon_cache();
                                    classes_version_keys.set(classes_version_keys() + 1);
                                }
                                polygon.write().reset();
                                schedule_overlay_redraw(polygon(), zoom(), pan_x(), pan_y());
                                // Auto-switch to Select (arrow) tool
                                selected_tool.set(Tool::Select);
                            }
                        }
                        Tool::BoundingBox => {
                            let mut bb = bbox.write();
                            if bb.start_point.is_some() && !bb.is_complete {
                                if let Some(preview) = bb.preview_point {
                                    bb.set_end(preview);
                                }
                            }
                            // If complete, persist and reset overlay
                            if bb.start_point.is_some() && bb.end_point.is_some() {
                                if let Some(class_id) = get_active_or_first_class_id(&pid_value) {
                                    use super::annotations::store::{save_bbox, SavedPoint as SP};
                                    let start = bb.start_point.unwrap();
                                    let end = bb.end_point.unwrap();
                                    save_bbox(&pid_value, &block_value, &image_value,
                                             &SP { x: start.x, y: start.y },
                                             &SP { x: end.x, y: end.y },
                                             &class_id);
                                    super::annotations::saved_canvas::invalidate_bbox_cache();
                                    increment_class_count(&pid_value, &class_id, 1);
                                    classes_version_keys.set(classes_version_keys() + 1);
                                }
                                bb.reset();
                            }
                            drop(bb);
                            if let Some(ctx) = get_canvas_context() {
                                redraw_bbox(&ctx, &bbox(), zoom(), pan_x(), pan_y());
                            }
                            // Auto-switch to Select (arrow) tool
                            selected_tool.set(Tool::Select);
                        }
                        _ => {}
                    }
                }
                "n" => {
                    // Start a new polygon annotation
                    selected_tool.set(Tool::Polygon);
                    // Reset any in-progress annotation
                    if polygon().points.len() > 0 || polygon().is_closed {
                        polygon.write().reset();
                        schedule_overlay_redraw(polygon(), zoom(), pan_x(), pan_y());
                    }
                    if bbox().start_point.is_some() {
                        bbox.write().reset();
                        if let Some(ctx) = get_canvas_context() {
                            redraw_bbox(&ctx, &bbox(), zoom(), pan_x(), pan_y());
                        }
                    }
                }
                "\\" => sidebar_open.set(!sidebar_open()),
                "t" => {
                    // Toggle theme
                    if let Some(win) = window() {
                        if let Some(document) = win.document() {
                            if let Some(html) = document.document_element() {
                                if let Ok(html_el) = html.dyn_into::<HtmlElement>() {
                                    let current_class = html_el.class_name();
                                    if current_class.contains("dark") {
                                        html_el.set_class_name("light");
                                    } else if current_class.contains("light") {
                                        html_el.set_class_name("dark");
                                    } else {
                                        let prefers_dark = win.match_media("(prefers-color-scheme: dark)")
                                            .ok()
                                            .flatten()
                                            .map(|m| m.matches())
                                            .unwrap_or(false);
                                        if prefers_dark {
                                            html_el.set_class_name("light");
                                        } else {
                                            html_el.set_class_name("dark");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }) as Box<dyn FnMut(_)>);

        let _ = win
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());

        // Keep closure alive
        closure.forget();
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

    // AnnotationMenu component will load classes itself

    // --- Handlers ----

    // Hit test helpers (world coords)
    fn point_in_poly(x: f64, y: f64, pts: &Vec<super::annotations::store::SavedPoint>) -> bool {
        let mut inside = false;
        let mut j = pts.len().wrapping_sub(1);
        for i in 0..pts.len() {
            let xi = pts[i].x; let yi = pts[i].y;
            let xj = pts[j].x; let yj = pts[j].y;
            let intersect = ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi + 1e-9) + xi);
            if intersect { inside = !inside; }
            j = i;
        }
        inside
    }
    fn point_in_bbox(x: f64, y: f64, bb: &super::annotations::store::SavedBBox) -> bool {
        let l = bb.start.x.min(bb.end.x);
        let r = bb.start.x.max(bb.end.x);
        let t = bb.start.y.min(bb.end.y);
        let b = bb.start.y.max(bb.end.y);
        x >= l && x <= r && y >= t && y <= b
    }

    // Annotation menu handlers moved into AnnotationMenu component
    // cursor-centered-zoom and shift+wheel for panning
    let onwheel = move |evt: Event<WheelData>| {
        // Close any open annotation menu on scroll
        if annotation_menu_open() { annotation_menu_open.set(false); }
        evt.prevent_default();
        let mouse = evt.client_coordinates();
        let delta = evt.data.delta().strip_units();
        
        // Shift key held = pan (like CVAT)
        if evt.modifiers().shift() {
            let dx = delta.x;
            let dy = delta.y;
            let new_pan_x = pan_x() - dx;
            let new_pan_y = pan_y() - dy;
            pan_x.set(new_pan_x);
            pan_y.set(new_pan_y);
            
            // Redraw overlays
            schedule_overlay_redraw(polygon(), zoom(), new_pan_x, new_pan_y);
            if bbox().start_point.is_some() && !bbox().is_complete {
                if let Some(ctx) = get_canvas_context() {
                    redraw_bbox(&ctx, &bbox(), zoom(), new_pan_x, new_pan_y);
                }
            }
            return;
        }
        
        // Normal zoom
        let dy = delta.y;
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
        // Note: use_effect watching zoom/pan will redraw Canvas-A
        // Always schedule overlay redraw (will clear if empty)
        schedule_overlay_redraw(polygon(), new, new_pan_x, new_pan_y);
        // If bbox in progress, redraw it as well
        if bbox().start_point.is_some() && !bbox().is_complete {
            if let Some(ctx) = get_canvas_context() {
                redraw_bbox(&ctx, &bbox(), new, new_pan_x, new_pan_y);
            }
        }
    };

    // start pan or add polygon vertex (NO world conversion)
    let pid_for_down = project_id_for_ann.clone();
    let block_for_down = block_id_for_ann.clone();
    let image_for_down = image_id_for_ann.clone();
    let onmousedown = move |evt: Event<MouseData>| {
        // Close menu on any left click
        if annotation_menu_open() && evt.data.trigger_button() == Some(MouseButton::Primary) { annotation_menu_open.set(false); }
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

        // Select tool + left mouse button -> click-drag pan (trackpad-style)
        if selected_tool() == Tool::Select && evt.data.trigger_button() == Some(MouseButton::Primary) {
            is_panning.set(true);
            last_x.set(coords.x);
            last_y.set(coords.y);
            return;
        }

        // Tool-based handling
        if selected_tool() == Tool::Polygon {
            // Convert screen coordinates to world coordinates
            let screen_x_vp = coords.x;           // viewport X
            let screen_y_vp = coords.y;           // viewport Y
            let screen_x = screen_x_vp;           // for X we don't offset by navbar
            let screen_y = screen_y_vp - NAVBAR_H; // container-relative Y

            // Require clicks to be inside the displayed image rect
            if !point_inside_image(screen_x_vp, screen_y_vp) { return; }

            // Apply inverse transform: (screen - pan) / zoom
            let mut world_x = (screen_x - pan_x()) / zoom();
            let mut world_y = (screen_y - pan_y()) / zoom();

            // Clamp to image bounds in world space
            if let Some((l, t, r, b)) = get_image_bounds_world(zoom(), pan_x(), pan_y()) {
                tracing::info!("Before clamp: world_x={}, world_y={}", world_x, world_y);
                world_x = clamp_f64(world_x, l, r);
                world_y = clamp_f64(world_y, t, b);
                tracing::info!("After clamp: world_x={}, world_y={}", world_x, world_y);
            }

            if let Some(ctx) = get_canvas_context() {
                // Check if clicking near first point to close (only if inside image)
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
                            // Persist completed polygon and reset for a new one
                            if let Some(class_id) = get_active_or_first_class_id(&pid_for_down) {
                                save_polygon(&pid_for_down, &block_for_down, &image_for_down, &poly, &class_id);
                                increment_class_count(&pid_for_down, &class_id, 1);
                                invalidate_polygon_cache();
                                classes_version.set(classes_version() + 1); // Triggers saved canvas redraw
                            }
                            poly.reset();
                            // Drop borrow before scheduling overlay redraw
                            drop(poly);
                            schedule_overlay_redraw(polygon(), zoom(), pan_x(), pan_y());
                            // Auto-switch to Select (arrow) tool
                            selected_tool.set(Tool::Select);
                            return;
                        }
                    }
                }
                on_polygon_click(world_x, world_y, &mut poly, &ctx, zoom(), pan_x(), pan_y());
                // Drop mutable borrow before scheduling rAF redraw
                drop(poly);
                schedule_overlay_redraw(polygon(), zoom(), pan_x(), pan_y());
            }
        }

        if selected_tool() == Tool::BoundingBox {
            // Convert screen coordinates to world coordinates
            let screen_x_vp = coords.x;           // viewport X
            let screen_y_vp = coords.y;           // viewport Y
            let screen_x = screen_x_vp;
            let screen_y = screen_y_vp - NAVBAR_H;

            // Apply inverse transform
            let mut world_x = (screen_x - pan_x()) / zoom();
            let mut world_y = (screen_y - pan_y()) / zoom();

            // Clamp to image bounds; also require first click to be inside image
            if let Some((l, t, r, b)) = get_image_bounds_world(zoom(), pan_x(), pan_y()) {
                if bbox().start_point.is_none() {
                    // First click must be inside image
                    if !point_inside_image(screen_x_vp, screen_y_vp) { return; }
                }
                tracing::info!("BBox before clamp: world_x={}, world_y={}", world_x, world_y);
                world_x = clamp_f64(world_x, l, r);
                world_y = clamp_f64(world_y, t, b);
                tracing::info!("BBox after clamp: world_x={}, world_y={}", world_x, world_y);
            } else {
                // If we can't read image bounds and this is the first click, allow as before
            }

            if let Some(ctx) = get_canvas_context() {
                let mut bb = bbox.write();
                let was_complete = bb.is_complete;
                on_bbox_click(world_x, world_y, &mut bb, &ctx, zoom(), pan_x(), pan_y());
                if !was_complete && bb.is_complete {
                    if let Some(class_id) = get_active_or_first_class_id(&pid_for_down) {
                        use super::annotations::store::{save_bbox, SavedPoint as SP};
                        let start = bb.start_point.unwrap();
                        let end = bb.end_point.unwrap();
                        save_bbox(&pid_for_down, &block_for_down, &image_for_down,
                                 &SP { x: start.x, y: start.y },
                                 &SP { x: end.x, y: end.y },
                                 &class_id);
                        super::annotations::saved_canvas::invalidate_bbox_cache();
                        increment_class_count(&pid_for_down, &class_id, 1);
                        classes_version.set(classes_version() + 1);
                    }
                    // Reset overlay and clear
                    bb.reset();
                    redraw_bbox(&ctx, &bb, zoom(), pan_x(), pan_y());
                    // Auto-switch to Select (arrow) tool
                    selected_tool.set(Tool::Select);
                }
            }
        }
    };

    // pan drag and preview line tracking
    let onmousemove = move |evt: Event<MouseData>| {
        if annotation_menu_open() { return; } // freeze interactions under menu
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
            // Note: use_effect watching pan will redraw Canvas-A
            // Always schedule overlay redraw (will clear if empty)
            schedule_overlay_redraw(polygon(), zoom(), new_pan_x, new_pan_y);
            // If bbox in progress, redraw it as well
            if bbox().start_point.is_some() && !bbox().is_complete {
                if let Some(ctx) = get_canvas_context() {
                    redraw_bbox(&ctx, &bbox(), zoom(), new_pan_x, new_pan_y);
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
                // Convert screen to world coords for preview, clamped to image bounds
                let screen_x = coords.x;
                let screen_y = coords.y - NAVBAR_H;
                let mut world_x = (screen_x - pan_x()) / zoom();
                let mut world_y = (screen_y - pan_y()) / zoom();

                if let Some((l, t, r, b)) = get_image_bounds_world(zoom(), pan_x(), pan_y()) {
                    world_x = clamp_f64(world_x, l, r);
                    world_y = clamp_f64(world_y, t, b);
                }

                polygon
                    .write()
                    .set_preview(Some(Point {
                        x: world_x,
                        y: world_y,
                    }));

                // Schedule rAF-gated redraw (throttled to 60fps)
                schedule_overlay_redraw(polygon(), zoom(), pan_x(), pan_y());
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
                // Convert screen to world coords for preview and clamp to image
                let screen_x = coords.x;
                let screen_y = coords.y - NAVBAR_H;
                let mut world_x = (screen_x - pan_x()) / zoom();
                let mut world_y = (screen_y - pan_y()) / zoom();

                if let Some((l, t, r, b)) = get_image_bounds_world(zoom(), pan_x(), pan_y()) {
                    world_x = clamp_f64(world_x, l, r);
                    world_y = clamp_f64(world_y, t, b);
                }

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
        if annotation_menu_open() { return; }
        is_panning.set(false);
    };

    //release pan
    let onmouseleave = move |_evt: Event<MouseData>| {
        if annotation_menu_open() { return; }
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
            oncontextmenu: move |evt: Event<MouseData>| {
                // Only intercept right-clicks inside the displayed image AND on an annotation
                let coords = evt.client_coordinates();
                let vp_x = coords.x;
                let vp_y = coords.y;
                if !point_inside_image(vp_x, vp_y) {
                    // Outside the image: let the browser menu appear
                    annotation_menu_open.set(false);
                    return;
                }

                // Convert to world coordinates for hit testing
                let screen_x = vp_x; let screen_y = vp_y - NAVBAR_H;
                let world_x = (screen_x - pan_x()) / zoom();
                let world_y = (screen_y - pan_y()) / zoom();

                // Prefer BBoxes (drawn last), then Polys; iterate in reverse order
                let mut found: Option<AnnotationTarget> = None;
                let polys = load_polygons(&project_id_for_ann, &block_id_for_ann, &image_id_for_ann);
                let bboxes = load_bboxes(&project_id_for_ann, &block_id_for_ann, &image_id_for_ann);
                for (idx, bb) in bboxes.iter().enumerate().rev() {
                    if point_in_bbox(world_x, world_y, bb) { found = Some(AnnotationTarget::BBox(idx)); break; }
                }
                if found.is_none() {
                    for (idx, sp) in polys.iter().enumerate().rev() {
                        if sp.points.len() >= 3 && point_in_poly(world_x, world_y, &sp.points) { found = Some(AnnotationTarget::Poly(idx)); break; }
                    }
                }

                if let Some(kind) = found {
                    // We own the annotation menu only when an annotation is under cursor
                    evt.prevent_default();
                    annotation_menu_target.set(Some(kind));
                    annotation_menu_x.set(vp_x);
                    annotation_menu_y.set(vp_y - NAVBAR_H);
                    annotation_menu_open.set(true);
                } else {
                    // Inside image but no annotation: let browser menu appear
                    annotation_menu_open.set(false);
                }
            },

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
                    style: format_args!("--cursor-x: {}px; --cursor-y: {}px;", cursor_x(), cursor_y()),
                    // Center rectangle
                    div {
                        class: "crosshair-center"
                    }
                }

            }

            // Annotation menu component
            if annotation_menu_open() && annotation_menu_target().is_some() {
                AnnotationMenu {
                    open: annotation_menu_open,
                    x: annotation_menu_x,
                    y: annotation_menu_y,
                    target: annotation_menu_target,
                    project_id: project_id.clone(),
                    block_id: block_id_for_ann.clone(),
                    image_id: image_id_for_ann.clone(),
                    zoom: zoom,
                    pan_x: pan_x,
                    pan_y: pan_y,
                    classes_version: classes_version,
                }
            }
            }

            // Sidebar
            Sidebar { task_id: task_id.clone(), project_id: project_id.clone(), sidebar_open: sidebar_open, classes_version: classes_version }

        }
    }
}

// Canvas layer kept for future annotations
