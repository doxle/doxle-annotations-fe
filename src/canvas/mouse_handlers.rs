use super::annotations::bbox::{on_bbox_click, redraw_bbox};
use super::annotations::edit_hit_test::{hit_test_node, hit_test_edge, hit_test_bbox_corner};
use super::annotations::edit_state::{start_edit, start_drag, end_drag, clear_edit, is_editing, get_edit_target, update_drag, EditTarget};
use super::annotations::overlay_canvas::{
    clear_polygon_preview, schedule_overlay_redraw, set_polygon_preview, schedule_edit_redraw,
};
use super::annotations::polygon::on_polygon_click;
use super::annotations::shapes::{BBox, Point, Polygon};
use super::annotations::store::{save_polygon};
use super::annotations::saved_canvas::{get_cached_polygons, get_cached_bboxes};
use super::annotations::{AnnotationTarget, Tool};
use super::cursor_state::CursorState;
use super::dom_cache::get_canvas_container;
use super::hit_utils::{point_in_bbox, point_in_poly};
use super::image_utils::{clamp_f64, get_image_bounds_world, point_inside_image, NAVBAR_H};
use super::sidebar::storage::{get_active_or_first_class_id, increment_class_count};
use crate::dioxus_elements::input_data::MouseButton;
use dioxus::prelude::*;

// Helper for backwards compat - returns overlay context
fn get_canvas_context() -> Option<web_sys::CanvasRenderingContext2d> {
    super::dom_cache::get_overlay_context()
}

pub fn handle_wheel(
    evt: &Event<WheelData>,
    mut annotation_dropdown_open: Signal<bool>,
    mut zoom: Signal<f64>,
    mut pan_x: Signal<f64>,
    mut pan_y: Signal<f64>,
    polygon: Signal<Polygon>,
    bbox: Signal<BBox>,
) {
    // Close any open annotation dropdown on scroll
    if annotation_dropdown_open() {
        annotation_dropdown_open.set(false);
    }

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
        if is_editing() {
            schedule_edit_redraw(zoom(), new_pan_x, new_pan_y);
        } else {
            schedule_overlay_redraw(polygon(), zoom(), new_pan_x, new_pan_y);
            if bbox().start_point.is_some() && !bbox().is_complete {
                if let Some(ctx) = get_canvas_context() {
                    redraw_bbox(&ctx, &bbox(), zoom(), new_pan_x, new_pan_y);
                }
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
    
    // Redraw appropriate overlay
    if is_editing() {
        schedule_edit_redraw(new, new_pan_x, new_pan_y);
    } else {
        // Always schedule overlay redraw (will clear if empty)
        schedule_overlay_redraw(polygon(), new, new_pan_x, new_pan_y);
        // If bbox in progress, redraw it as well
        if bbox().start_point.is_some() && !bbox().is_complete {
            if let Some(ctx) = get_canvas_context() {
                redraw_bbox(&ctx, &bbox(), new, new_pan_x, new_pan_y);
            }
        }
    }
}

pub fn handle_mousedown(
    evt: &Event<MouseData>,
    mut annotation_dropdown_open: Signal<bool>,
    mut selected_tool: Signal<Tool>,
    mut is_panning: Signal<bool>,
    mut last_x: Signal<f64>,
    mut last_y: Signal<f64>,
    zoom: Signal<f64>,
    pan_x: Signal<f64>,
    pan_y: Signal<f64>,
    mut polygon: Signal<Polygon>,
    mut bbox: Signal<BBox>,
    mut class_counter: Signal<u64>,
    project_id: String,
    block_id: String,
    image_id: String,
) {
    // Close dropdown on any left click
    if annotation_dropdown_open() && evt.data.trigger_button() == Some(MouseButton::Primary) {
        annotation_dropdown_open.set(false);
    }
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

    // Select tool + left mouse button
    if selected_tool() == Tool::Select && evt.data.trigger_button() == Some(MouseButton::Primary) {
        let screen_x = coords.x;
        let screen_y = coords.y - NAVBAR_H;
        let world_x = (screen_x - pan_x()) / zoom();
        let world_y = (screen_y - pan_y()) / zoom();
        
        // If already editing, check for node/edge hits first
        if is_editing() {
            if let Some(target) = get_edit_target() {
                let hit_annotation = match target {
                    EditTarget::Polygon { points, .. } => {
                        // Check node hit
                        if let Some(node_idx) = hit_test_node(Point::new(world_x, world_y), &points, zoom()) {
                            start_drag(Some(node_idx), Point::new(world_x, world_y));
                            return;
                        }
                        // Check edge hit (drag entire shape)
                        hit_test_edge(Point::new(world_x, world_y), &points, zoom(), true).is_some()
                    }
                    EditTarget::BBox { start, end, .. } => {
                        // Check corner hit
                        if let Some(corner_idx) = hit_test_bbox_corner(Point::new(world_x, world_y), start, end, zoom()) {
                            // Map corner to start/end (0=start, 2=end for diagonal corners)
                            let node_idx = if corner_idx == 0 || corner_idx == 3 { 0 } else { 1 };
                            start_drag(Some(node_idx), Point::new(world_x, world_y));
                            return;
                        }
                        // Check if clicking inside bbox
                        world_x >= start.x.min(end.x) && world_x <= start.x.max(end.x) &&
                        world_y >= start.y.min(end.y) && world_y <= start.y.max(end.y)
                    }
                };
                
                if hit_annotation {
                    // Start dragging the whole shape
                    start_drag(None, Point::new(world_x, world_y));
                    return;
                } else {
                    // Clicked outside the edited annotation - exit edit mode
                    clear_edit();
                    // Clear overlay
                    schedule_edit_redraw(zoom(), pan_x(), pan_y());
                    // Redraw saved canvas directly without signal
                    use super::annotations::saved_canvas::redraw_saved_annotations;
                    redraw_saved_annotations(&project_id, &block_id, &image_id, zoom(), pan_x(), pan_y());
                    // Start panning
                    is_panning.set(true);
                    last_x.set(coords.x);
                    last_y.set(coords.y);
                    return;
                }
            }
        }
        
        // Only check for annotations if we were editing (need to exit) or might select something
        // Skip expensive localStorage reads if just panning around
        if is_editing() {
            // We were editing, clicking outside should exit
            clear_edit();
            schedule_edit_redraw(zoom(), pan_x(), pan_y());
            // Redraw saved canvas directly
            use super::annotations::saved_canvas::redraw_saved_annotations;
            redraw_saved_annotations(&project_id, &block_id, &image_id, zoom(), pan_x(), pan_y());
            // Start panning
            is_panning.set(true);
            last_x.set(coords.x);
            last_y.set(coords.y);
            } else {
                // Not editing - check if clicking on annotation to select
                // Use cached versions to avoid localStorage reads on every click
                let saved_polys = get_cached_polygons(&project_id, &block_id, &image_id);
                let saved_bboxes = get_cached_bboxes(&project_id, &block_id, &image_id);
                
                let mut found: Option<AnnotationTarget> = None;
            
            // Prefer bboxes over polygons
            for (idx, bb) in saved_bboxes.iter().enumerate().rev() {
                if point_in_bbox(world_x, world_y, bb) {
                    found = Some(AnnotationTarget::BBox(idx));
                    break;
                }
            }
            
            if found.is_none() {
                for (idx, sp) in saved_polys.iter().enumerate().rev() {
                    if sp.points.len() >= 3 && point_in_poly(world_x, world_y, &sp.points) {
                        found = Some(AnnotationTarget::Poly(idx));
                        break;
                    }
                }
            }
            
            if let Some(target) = found {
                // Enter edit mode
                start_edit(target, &saved_polys, &saved_bboxes);
                schedule_edit_redraw(zoom(), pan_x(), pan_y());
                // Redraw saved canvas directly to hide selected annotation (no data change, no counter)
                use super::annotations::saved_canvas::redraw_saved_annotations;
                redraw_saved_annotations(&project_id, &block_id, &image_id, zoom(), pan_x(), pan_y());
            } else {
                // No annotation clicked, just pan
                is_panning.set(true);
                last_x.set(coords.x);
                last_y.set(coords.y);
            }
        }
        return;
    }

    // Tool-based handling
    if selected_tool() == Tool::Polygon {
        // Convert screen coordinates to world coordinates
        let screen_x_vp = coords.x; // viewport X
        let screen_y_vp = coords.y; // viewport Y
        let screen_x = screen_x_vp; // for X we don't offset by navbar
        let screen_y = screen_y_vp - NAVBAR_H; // container-relative Y

        // Require clicks to be inside the displayed image rect
        if !point_inside_image(screen_x_vp, screen_y_vp) {
            return;
        }

        // Apply inverse transform: (screen - pan) / zoom
        let mut world_x = (screen_x - pan_x()) / zoom();
        let mut world_y = (screen_y - pan_y()) / zoom();

        // Clamp to image bounds in world space
        if let Some((l, t, r, b)) = get_image_bounds_world(zoom(), pan_x(), pan_y()) {
            world_x = clamp_f64(world_x, l, r);
            world_y = clamp_f64(world_y, t, b);
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
                        if let Some(class_id) = get_active_or_first_class_id(&project_id) {
                            save_polygon(&project_id, &block_id, &image_id, &poly, &class_id);
                            increment_class_count(&project_id, &class_id, 1);
                            // Cache invalidation will happen in use_effect
                            class_counter.set(class_counter() + 1); // Triggers saved canvas redraw
                        }
                        poly.reset();
                        // Drop borrow before scheduling overlay redraw
                        drop(poly);
                        clear_polygon_preview();
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
        let screen_x_vp = coords.x; // viewport X
        let screen_y_vp = coords.y; // viewport Y
        let screen_x = screen_x_vp;
        let screen_y = screen_y_vp - NAVBAR_H;

        // Apply inverse transform
        let mut world_x = (screen_x - pan_x()) / zoom();
        let mut world_y = (screen_y - pan_y()) / zoom();

        // Clamp to image bounds; also require first click to be inside image
        if let Some((l, t, r, b)) = get_image_bounds_world(zoom(), pan_x(), pan_y()) {
            if bbox().start_point.is_none() {
                // First click must be inside image
                if !point_inside_image(screen_x_vp, screen_y_vp) {
                    return;
                }
            }
            world_x = clamp_f64(world_x, l, r);
            world_y = clamp_f64(world_y, t, b);
        } else {
            // If we can't read image bounds and this is the first click, allow as before
        }

        if let Some(ctx) = get_canvas_context() {
            let mut bb = bbox.write();
            let was_complete = bb.is_complete;
            on_bbox_click(world_x, world_y, &mut bb, &ctx, zoom(), pan_x(), pan_y());
            if !was_complete && bb.is_complete {
                if let Some(class_id) = get_active_or_first_class_id(&project_id) {
                    use super::annotations::store::{save_bbox, SavedPoint as SP};
                    let start = bb.start_point.unwrap();
                    let end = bb.end_point.unwrap();
                    save_bbox(
                        &project_id,
                        &block_id,
                        &image_id,
                        &SP {
                            x: start.x,
                            y: start.y,
                        },
                        &SP { x: end.x, y: end.y },
                        &class_id,
                    );
                    // Cache invalidation will happen in use_effect
                    increment_class_count(&project_id, &class_id, 1);
                    class_counter.set(class_counter() + 1);
                }
                // Reset overlay and clear
                bb.reset();
                redraw_bbox(&ctx, &bb, zoom(), pan_x(), pan_y());
                // Auto-switch to Select (arrow) tool
                selected_tool.set(Tool::Select);
            }
        }
    }
}

pub fn handle_mousemove(
    evt: &Event<MouseData>,
    annotation_dropdown_open: Signal<bool>,
    selected_tool: Signal<Tool>,
    is_panning: Signal<bool>,
    mut last_x: Signal<f64>,
    mut last_y: Signal<f64>,
    mut cursor_state: CursorState,
    zoom: Signal<f64>,
    mut pan_x: Signal<f64>,
    mut pan_y: Signal<f64>,
    mut polygon: Signal<Polygon>,
    mut bbox: Signal<BBox>,
    mut hovered_annotation: Signal<Option<AnnotationTarget>>,
    project_id: String,
    block_id: String,
    image_id: String,
) {
    if annotation_dropdown_open() {
        return;
    } // freeze interactions under dropdown
    let coords = evt.client_coordinates();
    
    // Handle edit mode dragging (fast path - no signals!)
    if is_editing() {
        use super::annotations::edit_state::is_dragging;
        if is_dragging() {
            let screen_x = coords.x;
            let screen_y = coords.y - NAVBAR_H;
            let world_x = (screen_x - pan_x()) / zoom();
            let world_y = (screen_y - pan_y()) / zoom();
            
            if update_drag(Point::new(world_x, world_y)) {
                schedule_edit_redraw(zoom(), pan_x(), pan_y());
            }
            return;
        }
    }

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
        
        // Redraw edit overlay if editing
        if is_editing() {
            schedule_edit_redraw(zoom(), new_pan_x, new_pan_y);
        } else {
            // Always schedule overlay redraw (will clear if empty)
            schedule_overlay_redraw(polygon(), zoom(), new_pan_x, new_pan_y);
            // If bbox in progress, redraw it as well
            if bbox().start_point.is_some() && !bbox().is_complete {
                if let Some(ctx) = get_canvas_context() {
                    redraw_bbox(&ctx, &bbox(), zoom(), new_pan_x, new_pan_y);
                }
            }
        }
        return;
    }

    // Update preview line and guide lines when drawing polygon
    if selected_tool() == Tool::Polygon {
        // Calculate world coordinates and clamp FIRST
        let screen_x = coords.x;
        let screen_y = coords.y - NAVBAR_H;
        let mut world_x = (screen_x - pan_x()) / zoom();
        let mut world_y = (screen_y - pan_y()) / zoom();

        // Clamp to image bounds
        let mut clamped_screen_x = screen_x;
        let mut clamped_screen_y = screen_y;
        if let Some((l, t, r, b)) = get_image_bounds_world(zoom(), pan_x(), pan_y()) {
            world_x = clamp_f64(world_x, l, r);
            world_y = clamp_f64(world_y, t, b);
            // Convert clamped world coords back to screen for crosshair
            clamped_screen_x = world_x * zoom() + pan_x();
            clamped_screen_y = world_y * zoom() + pan_y();
        }

        // Update cursor positions with clamped coordinates - use cached DOM element
        if let Some(html_elem) = get_canvas_container() {
            let style = html_elem.style();
            let _ = style.set_property("--guide-x", &format!("{}px", clamped_screen_x));
            let _ = style.set_property("--guide-y", &format!("{}px", clamped_screen_y));
            let _ = style.set_property("--cursor-x", &format!("{}px", clamped_screen_x));
            let _ = style.set_property("--cursor-y", &format!("{}px", clamped_screen_y));
            // Add guides-active class to show grid lines after first mousemove
            let current_class = html_elem.class_name();
            if !current_class.contains("guides-active") {
                html_elem.set_class_name(&format!("{} guides-active", current_class));
            }
        }

        // Update preview point for drawing
        if polygon().points.len() > 0 && !polygon().is_closed {
            set_polygon_preview(Some(Point::new(world_x, world_y)));
            schedule_overlay_redraw(polygon(), zoom(), pan_x(), pan_y());
        }
    }

    // Update preview box when drawing bbox
    if selected_tool() == Tool::BoundingBox {
        // Calculate world coordinates and clamp FIRST
        let screen_x = coords.x;
        let screen_y = coords.y - NAVBAR_H;
        let mut world_x = (screen_x - pan_x()) / zoom();
        let mut world_y = (screen_y - pan_y()) / zoom();

        // Clamp to image bounds
        let mut clamped_screen_x = screen_x;
        let mut clamped_screen_y = screen_y;
        if let Some((l, t, r, b)) = get_image_bounds_world(zoom(), pan_x(), pan_y()) {
            world_x = clamp_f64(world_x, l, r);
            world_y = clamp_f64(world_y, t, b);
            // Convert clamped world coords back to screen for crosshair
            clamped_screen_x = world_x * zoom() + pan_x();
            clamped_screen_y = world_y * zoom() + pan_y();
        }

        // Update cursor positions with clamped coordinates - use cached DOM element
        if let Some(html_elem) = get_canvas_container() {
            let style = html_elem.style();
            let _ = style.set_property("--guide-x", &format!("{}px", clamped_screen_x));
            let _ = style.set_property("--guide-y", &format!("{}px", clamped_screen_y));
            let _ = style.set_property("--cursor-x", &format!("{}px", clamped_screen_x));
            let _ = style.set_property("--cursor-y", &format!("{}px", clamped_screen_y));
            // Add guides-active class
            let current_class = html_elem.class_name();
            if !current_class.contains("guides-active") {
                html_elem.set_class_name(&format!("{} guides-active", current_class));
            }
        }

        // Update preview point for bbox drawing
        if bbox().start_point.is_some() && !bbox().is_complete {
            bbox.write().set_preview(Some(Point::new(world_x, world_y)));
            if let Some(ctx) = get_canvas_context() {
                redraw_bbox(&ctx, &bbox(), zoom(), pan_x(), pan_y());
            }
        }
    }

    // Check if hovering over saved annotations (for hover effects and delete key)
    if selected_tool() == Tool::Select || selected_tool() == Tool::Pan {
        let screen_x = coords.x;
        let screen_y = coords.y - NAVBAR_H;
        let world_x = (screen_x - pan_x()) / zoom();
        let world_y = (screen_y - pan_y()) / zoom();

        // Check annotations (prefer bboxes, then polygons)
        let mut found: Option<AnnotationTarget> = None;
        let polys = get_cached_polygons(&project_id, &block_id, &image_id);
        let bboxes = get_cached_bboxes(&project_id, &block_id, &image_id);

        for (idx, bb) in bboxes.iter().enumerate().rev() {
            if point_in_bbox(world_x, world_y, bb) {
                found = Some(AnnotationTarget::BBox(idx));
                break;
            }
        }
        if found.is_none() {
            for (idx, sp) in polys.iter().enumerate().rev() {
                if sp.points.len() >= 3 && point_in_poly(world_x, world_y, &sp.points) {
                    found = Some(AnnotationTarget::Poly(idx));
                    break;
                }
            }
        }

        hovered_annotation.set(found);
    } else {
        // Clear hover when using drawing tools
        hovered_annotation.set(None);
    }
}

pub fn handle_mouseup(
    _evt: &Event<MouseData>,
    annotation_dropdown_open: Signal<bool>,
    mut is_panning: Signal<bool>,
    mut class_counter: Signal<u64>,
    project_id: String,
    block_id: String,
    image_id: String,
    zoom: Signal<f64>,
    pan_x: Signal<f64>,
    pan_y: Signal<f64>,
) {
    if annotation_dropdown_open() {
        return;
    }
    
    // End drag but stay in edit mode
    use super::annotations::edit_state::is_dragging;
    use super::annotations::edit_commit::commit_edit;
    if is_dragging() {
        end_drag();
        // Commit changes but DON'T clear edit mode - stay selected for further editing
        if commit_edit(&project_id, &block_id, &image_id) {
            // Keep edit mode active, just refresh the display
            schedule_edit_redraw(zoom(), pan_x(), pan_y());
            // Invalidate saved canvas cache
            use super::annotations::saved_canvas::{invalidate_polygon_cache, invalidate_bbox_cache};
            invalidate_polygon_cache();
            invalidate_bbox_cache();
            class_counter.set(class_counter() + 1);
        }
    }
    is_panning.set(false);
}

pub fn handle_mouseleave(
    _evt: &Event<MouseData>,
    annotation_dropdown_open: Signal<bool>,
    mut is_panning: Signal<bool>,
) {
    if annotation_dropdown_open() {
        return;
    }
    is_panning.set(false);
}

pub fn handle_annotation_dropdown(
    evt: &Event<MouseData>,
    mut annotation_dropdown_open: Signal<bool>,
    mut annotation_dropdown_x: Signal<f64>,
    mut annotation_dropdown_y: Signal<f64>,
    mut annotation_dropdown_target: Signal<Option<AnnotationTarget>>,
    zoom: Signal<f64>,
    pan_x: Signal<f64>,
    pan_y: Signal<f64>,
    project_id: String,
    block_id: String,
    image_id: String,
) {
    // Only intercept right-clicks inside the displayed image AND on an annotation
    let coords = evt.client_coordinates();
    let vp_x = coords.x;
    let vp_y = coords.y;
    if !point_inside_image(vp_x, vp_y) {
        // Outside the image: let the browser show default context actions
        annotation_dropdown_open.set(false);
        return;
    }

    // Convert to world coordinates for hit testing
    let screen_x = vp_x;
    let screen_y = vp_y - NAVBAR_H;
    let world_x = (screen_x - pan_x()) / zoom();
    let world_y = (screen_y - pan_y()) / zoom();

    // Prefer BBoxes (drawn last), then Polys; iterate in reverse order
    let mut found: Option<AnnotationTarget> = None;
    let polys = get_cached_polygons(&project_id, &block_id, &image_id);
    let bboxes = get_cached_bboxes(&project_id, &block_id, &image_id);
    for (idx, bb) in bboxes.iter().enumerate().rev() {
        if point_in_bbox(world_x, world_y, bb) {
            found = Some(AnnotationTarget::BBox(idx));
            break;
        }
    }
    if found.is_none() {
        for (idx, sp) in polys.iter().enumerate().rev() {
            if sp.points.len() >= 3 && point_in_poly(world_x, world_y, &sp.points) {
                found = Some(AnnotationTarget::Poly(idx));
                break;
            }
        }
    }

    if let Some(kind) = found {
        // We own the annotation dropdown only when an annotation is under cursor
        evt.prevent_default();
        annotation_dropdown_target.set(Some(kind));
        annotation_dropdown_x.set(vp_x);
        annotation_dropdown_y.set(vp_y - NAVBAR_H);
        annotation_dropdown_open.set(true);
    } else {
        // Inside image but no annotation: let browser show default context actions
        annotation_dropdown_open.set(false);
    }
}
