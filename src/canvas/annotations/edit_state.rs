use std::cell::RefCell;
use super::shapes::Point;
use super::store::{SavedPolygon, SavedBBox};
use crate::canvas::annotations::AnnotationTarget;

/// Represents what we're currently editing
#[derive(Clone, Debug)]
pub enum EditTarget {
    Polygon {
        index: usize,
        points: Vec<Point>,
        class_id: String,
    },
    BBox {
        index: usize,
        start: Point,
        end: Point,
        class_id: String,
    },
}

/// Node or edge being dragged
#[derive(Clone, Debug)]
pub struct DragState {
    pub node_index: Option<usize>, // None if dragging entire shape
    pub start_mouse: Point,        // Mouse position at drag start
    pub start_points: Vec<Point>,  // Original points before drag
}

thread_local! {
    /// Currently selected annotation for editing
    static SELECTED_ANNOTATION: RefCell<Option<EditTarget>> = RefCell::new(None);
    
    /// Active drag operation
    static DRAG_STATE: RefCell<Option<DragState>> = RefCell::new(None);
    
    /// Snapshot for undo on Escape
    static EDIT_SNAPSHOT: RefCell<Option<EditTarget>> = RefCell::new(None);
}

// --- Public API ---

/// Start editing an annotation
pub fn start_edit(target: AnnotationTarget, saved_polys: &[SavedPolygon], saved_bboxes: &[SavedBBox]) {
    let edit_target = match target {
        AnnotationTarget::Poly(idx) => {
            if let Some(poly) = saved_polys.get(idx) {
                Some(EditTarget::Polygon {
                    index: idx,
                    points: poly.points.iter().map(|p| Point::new(p.x, p.y)).collect(),
                    class_id: poly.class_id.clone(),
                })
            } else {
                None
            }
        }
        AnnotationTarget::BBox(idx) => {
            if let Some(bbox) = saved_bboxes.get(idx) {
                Some(EditTarget::BBox {
                    index: idx,
                    start: Point::new(bbox.start.x, bbox.start.y),
                    end: Point::new(bbox.end.x, bbox.end.y),
                    class_id: bbox.class_id.clone(),
                })
            } else {
                None
            }
        }
        _ => None,
    };
    
    if let Some(target) = edit_target {
        // Store snapshot for undo
        EDIT_SNAPSHOT.with(|s| *s.borrow_mut() = Some(target.clone()));
        SELECTED_ANNOTATION.with(|a| *a.borrow_mut() = Some(target));
    }
}

/// Check if we're currently editing
pub fn is_editing() -> bool {
    SELECTED_ANNOTATION.with(|a| a.borrow().is_some())
}

/// Get the current edit target
pub fn get_edit_target() -> Option<EditTarget> {
    SELECTED_ANNOTATION.with(|a| a.borrow().clone())
}

/// Start dragging a node or entire shape
pub fn start_drag(node_index: Option<usize>, mouse_pos: Point) {
    if let Some(target) = get_edit_target() {
        let points = match &target {
            EditTarget::Polygon { points, .. } => points.clone(),
            EditTarget::BBox { start, end, .. } => vec![*start, *end],
        };
        
        DRAG_STATE.with(|d| {
            *d.borrow_mut() = Some(DragState {
                node_index,
                start_mouse: mouse_pos,
                start_points: points,
            });
        });
    }
}

/// Update dragged node position (called on mousemove)
pub fn update_drag(mouse_pos: Point) -> bool {
    let drag_state = DRAG_STATE.with(|d| d.borrow().clone());
    
    if let Some(drag) = drag_state {
        SELECTED_ANNOTATION.with(|a| {
            if let Some(mut target) = a.borrow_mut().as_mut() {
                let dx = mouse_pos.x - drag.start_mouse.x;
                let dy = mouse_pos.y - drag.start_mouse.y;
                
                match &mut target {
                    EditTarget::Polygon { points, .. } => {
                        if let Some(node_idx) = drag.node_index {
                            // Drag single node
                            if let Some(original) = drag.start_points.get(node_idx) {
                                if let Some(point) = points.get_mut(node_idx) {
                                    point.x = original.x + dx;
                                    point.y = original.y + dy;
                                }
                            }
                        } else {
                            // Drag entire polygon
                            for (i, point) in points.iter_mut().enumerate() {
                                if let Some(original) = drag.start_points.get(i) {
                                    point.x = original.x + dx;
                                    point.y = original.y + dy;
                                }
                            }
                        }
                    }
                    EditTarget::BBox { start, end, .. } => {
                        if let Some(node_idx) = drag.node_index {
                            // Drag corner
                            let original = &drag.start_points[node_idx];
                            let target_point = if node_idx == 0 { start } else { end };
                            target_point.x = original.x + dx;
                            target_point.y = original.y + dy;
                        } else {
                            // Drag entire bbox
                            if let (Some(orig_start), Some(orig_end)) = 
                                (drag.start_points.get(0), drag.start_points.get(1)) {
                                start.x = orig_start.x + dx;
                                start.y = orig_start.y + dy;
                                end.x = orig_end.x + dx;
                                end.y = orig_end.y + dy;
                            }
                        }
                    }
                }
                return true;
            }
            false
        })
    } else {
        false
    }
}

/// End drag operation
pub fn end_drag() {
    DRAG_STATE.with(|d| *d.borrow_mut() = None);
}

/// Cancel edit and restore snapshot
pub fn cancel_edit() {
    if let Some(snapshot) = EDIT_SNAPSHOT.with(|s| s.borrow_mut().take()) {
        SELECTED_ANNOTATION.with(|a| *a.borrow_mut() = Some(snapshot));
    }
    end_drag();
}

/// Clear all edit state
pub fn clear_edit() {
    SELECTED_ANNOTATION.with(|a| *a.borrow_mut() = None);
    DRAG_STATE.with(|d| *d.borrow_mut() = None);
    EDIT_SNAPSHOT.with(|s| *s.borrow_mut() = None);
}

/// Check if currently dragging
pub fn is_dragging() -> bool {
    DRAG_STATE.with(|d| d.borrow().is_some())
}

/// Update the edit snapshot after committing changes (for undo)
pub fn update_edit_snapshot(new_target: EditTarget) {
    EDIT_SNAPSHOT.with(|s| *s.borrow_mut() = Some(new_target.clone()));
    SELECTED_ANNOTATION.with(|a| *a.borrow_mut() = Some(new_target));
}
