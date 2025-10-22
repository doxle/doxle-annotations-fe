use super::annotations::bbox::redraw_bbox;
use super::annotations::shapes::BBox;
use super::dom_cache::{get_window, get_document};
use super::annotations::overlay_canvas::{get_overlay_canvas_context, schedule_overlay_redraw};
use super::annotations::shapes::Polygon;
use super::annotations::store::{delete_bbox_at, delete_polygon_at, load_bboxes, load_polygons, save_bbox, save_polygon, SavedPoint as SP};
use super::annotations::AnnotationTarget;
use super::annotations::Tool;
use super::sidebar::storage::{get_active_or_first_class_id, increment_class_count};
use dioxus::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{window, HtmlElement, KeyboardEvent};

fn get_canvas_context() -> Option<web_sys::CanvasRenderingContext2d> {
    get_overlay_canvas_context()
}

pub fn setup_keyboard_shortcuts(
    project_id: String,
    block_id: String,
    image_id: String,
    mut selected_tool: Signal<Tool>,
    mut polygon: Signal<Polygon>,
    mut bbox: Signal<BBox>,
    zoom: Signal<f64>,
    pan_x: Signal<f64>,
    pan_y: Signal<f64>,
    mut sidebar_open: Signal<bool>,
    mut class_counter: Signal<u64>,
    mut hovered_annotation: Signal<Option<AnnotationTarget>>,
) {
    use_effect(move || {
        let win = match get_window() {
            Some(w) => w,
            None => return,
        };

        let pid_value = project_id.clone();
        let block_value = block_id.clone();
        let image_value = image_id.clone();
        
        let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            let key = event.key();

            // Handle Delete/Backspace key - delete hovered annotation
            if key == "Delete" || key == "Backspace" {
                if let Some(target) = hovered_annotation() {
                    match target {
                        AnnotationTarget::Poly(idx) => {
                            let polys = load_polygons(&pid_value, &block_value, &image_value);
                            if let Some(old) = polys.get(idx) {
                                super::sidebar::storage::increment_class_count(&pid_value, &old.class_id, -1);
                            }
                            delete_polygon_at(&pid_value, &block_value, &image_value, idx);
                        }
                        AnnotationTarget::BBox(idx) => {
                            let bboxes = load_bboxes(&pid_value, &block_value, &image_value);
                            if let Some(old) = bboxes.get(idx) {
                                super::sidebar::storage::increment_class_count(&pid_value, &old.class_id, -1);
                            }
                            delete_bbox_at(&pid_value, &block_value, &image_value, idx);
                        }
                    }
                    hovered_annotation.set(None);
                    class_counter.set(class_counter() + 1); // Triggers reactive redraw
                }
                return;
            }
            
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
                        if let Some(class_id) = get_active_or_first_class_id(&pid_value) {
                            save_polygon(
                                &pid_value,
                                &block_value,
                                &image_value,
                                &polygon(),
                                &class_id,
                            );
                            increment_class_count(&pid_value, &class_id, 1);
                            class_counter.set(class_counter() + 1); // Triggers reactive redraw in canvas_effects
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
                                if let Some(class_id) = get_active_or_first_class_id(&pid_value) {
                                    save_polygon(
                                        &pid_value,
                                        &block_value,
                                        &image_value,
                                        &polygon(),
                                        &class_id,
                                    );
                                    increment_class_count(&pid_value, &class_id, 1);
                                    class_counter.set(class_counter() + 1); // Triggers reactive redraw in canvas_effects
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
                                // Note: BBox preview now in thread_local, this code path unused
                                // Would need to get current mouse position here
                            }
                            // If complete, persist and reset overlay
                            if bb.start_point.is_some() && bb.end_point.is_some() {
                                if let Some(class_id) = get_active_or_first_class_id(&pid_value) {
                                    let start = bb.start_point.unwrap();
                                    let end = bb.end_point.unwrap();
                                    save_bbox(
                                        &pid_value,
                                        &block_value,
                                        &image_value,
                                        &SP {
                                            x: start.x,
                                            y: start.y,
                                        },
                                        &SP { x: end.x, y: end.y },
                                        &class_id,
                                    );
                                    increment_class_count(&pid_value, &class_id, 1);
                                    class_counter.set(class_counter() + 1); // Triggers reactive redraw in canvas_effects
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
                    if let Some(win) = get_window() {
                        if let Some(document) = get_document() {
                            if let Some(html) = document.document_element() {
                                if let Ok(html_el) = html.dyn_into::<HtmlElement>() {
                                    let current_class = html_el.class_name();
                                    if current_class.contains("dark") {
                                        html_el.set_class_name("light");
                                    } else if current_class.contains("light") {
                                        html_el.set_class_name("dark");
                                    } else {
                                        let prefers_dark = win
                                            .match_media("(prefers-color-scheme: dark)")
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

        let _ = win.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());

        // Keep closure alive
        closure.forget();
    });
}
