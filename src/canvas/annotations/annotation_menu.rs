use super::saved_canvas::{redraw_saved_annotations, invalidate_polygon_cache, invalidate_bbox_cache};
use super::store::{
    delete_bbox_at, delete_polygon_at, load_bboxes, load_polygons, update_bbox_class,
    update_polygon_class,
};
use crate::canvas::sidebar::storage::{increment_class_count, load_json};
use crate::canvas::sidebar::ClassItem;
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum AnnotationTarget {
    Poly(usize),
    BBox(usize),
}

#[component]
pub fn AnnotationMenu(
    open: Signal<bool>,
    x: Signal<f64>,
    y: Signal<f64>,
    target: Signal<Option<AnnotationTarget>>,
    project_id: String,
    block_id: String,
    image_id: String,
    zoom: Signal<f64>,
    pan_x: Signal<f64>,
    pan_y: Signal<f64>,
    classes_version: Signal<u64>,
) -> Element {
    let classes: Vec<ClassItem> = load_json(&format!("classes:{}", project_id)).unwrap_or_default();
    let mut selected_class = use_signal(|| String::new());

    // Clone IDs for use in use_effect
    let pid_for_effect = project_id.clone();
    let bid_for_effect = block_id.clone();
    let iid_for_effect = image_id.clone();

    // Get current annotation's class
    use_effect(move || {
        if let Some(t) = target() {
            let current_class_id = match t {
                AnnotationTarget::Poly(idx) => {
                    let polys = load_polygons(&pid_for_effect, &bid_for_effect, &iid_for_effect);
                    polys.get(idx).map(|p| p.class_id.clone())
                }
                AnnotationTarget::BBox(idx) => {
                    let bboxes = load_bboxes(&pid_for_effect, &bid_for_effect, &iid_for_effect);
                    bboxes.get(idx).map(|b| b.class_id.clone())
                }
            };
            if let Some(cid) = current_class_id {
                selected_class.set(cid);
            }
        }
    });

    // Handler builders (avoid moving captured values repeatedly)
    fn make_reassign_handler(
        pid: std::rc::Rc<String>,
        bid: std::rc::Rc<String>,
        iid: std::rc::Rc<String>,
        mut classes_version: Signal<u64>,
        mut open_sig: Signal<bool>,
        target: Signal<Option<AnnotationTarget>>,
        new_class_id: String,
        zoom: Signal<f64>,
        pan_x: Signal<f64>,
        pan_y: Signal<f64>,
    ) -> impl FnMut(Event<MouseData>) {
        move |_| {
            if let Some(t) = target() {
                match t {
                    AnnotationTarget::Poly(idx) => {
                        let polys = load_polygons(&*pid, &*bid, &*iid);
                        if let Some(old) = polys.get(idx) {
                            increment_class_count(&*pid, &old.class_id, -1);
                        }
                        if update_polygon_class(&*pid, &*bid, &*iid, idx, &new_class_id) {
                            increment_class_count(&*pid, &new_class_id, 1);
                        }
                    }
                    AnnotationTarget::BBox(idx) => {
                        let bboxes = load_bboxes(&*pid, &*bid, &*iid);
                        if let Some(old) = bboxes.get(idx) {
                            increment_class_count(&*pid, &old.class_id, -1);
                        }
                        if update_bbox_class(&*pid, &*bid, &*iid, idx, &new_class_id) {
                            increment_class_count(&*pid, &new_class_id, 1);
                        }
                    }
                }
                redraw_saved_annotations(&*pid, &*bid, &*iid, zoom(), pan_x(), pan_y());
                classes_version.set(classes_version() + 1);
                open_sig.set(false);
            }
        }
    }

    fn make_delete_handler(
        pid: std::rc::Rc<String>,
        bid: std::rc::Rc<String>,
        iid: std::rc::Rc<String>,
        mut classes_version: Signal<u64>,
        mut open_sig: Signal<bool>,
        target: Signal<Option<AnnotationTarget>>,
        zoom: Signal<f64>,
        pan_x: Signal<f64>,
        pan_y: Signal<f64>,
    ) -> impl FnMut(Event<MouseData>) {
        move |_| {
            tracing::info!("Delete button clicked");
            if let Some(t) = target() {
                match t {
                    AnnotationTarget::Poly(idx) => {
                        let polys_before = load_polygons(&*pid, &*bid, &*iid);
                        tracing::info!("BEFORE delete - Total polys: {}, deleting index: {}", polys_before.len(), idx);
                        
                        if let Some(old) = polys_before.get(idx) {
                            increment_class_count(&*pid, &old.class_id, -1);
                        }
                        let deleted = delete_polygon_at(&*pid, &*bid, &*iid, idx);
                        tracing::info!("delete_polygon_at returned: {}", deleted);
                        
                        invalidate_polygon_cache();
                        
                        let polys_after = load_polygons(&*pid, &*bid, &*iid);
                        tracing::info!("AFTER delete - Total polys: {}", polys_after.len());
                    }
                    AnnotationTarget::BBox(idx) => {
                        let bboxes_before = load_bboxes(&*pid, &*bid, &*iid);
                        tracing::info!("BEFORE delete - Total bboxes: {}, deleting index: {}", bboxes_before.len(), idx);
                        
                        if let Some(old) = bboxes_before.get(idx) {
                            increment_class_count(&*pid, &old.class_id, -1);
                        }
                        let deleted = delete_bbox_at(&*pid, &*bid, &*iid, idx);
                        tracing::info!("delete_bbox_at returned: {}", deleted);
                        
                        invalidate_bbox_cache();
                        
                        let bboxes_after = load_bboxes(&*pid, &*bid, &*iid);
                        tracing::info!("AFTER delete - Total bboxes: {}", bboxes_after.len());
                    }
                }
                // Invalidation already done above, now trigger redraw via use_effect
                classes_version.set(classes_version() + 1);
                open_sig.set(false);
            }
        }
    }

    // Pre-clone shared state into cheap Rc/Signal clones for multiple closures
    let pid_arc = std::rc::Rc::new(project_id.clone());
    let bid_arc = std::rc::Rc::new(block_id.clone());
    let iid_arc = std::rc::Rc::new(image_id.clone());
    let open_base = open.clone();
    let ver_base = classes_version.clone();
    let tgt_base = target.clone();
    let z_base = zoom.clone();
    let px_base = pan_x.clone();
    let py_base = pan_y.clone();

    rsx! {
        div {
            class: "annotation-dropdown-menu annotation_menu",
            style: format_args!("left: {}px; top: {}px; right: auto;", x(), y()),
            onmousedown: move |e| { e.stop_propagation(); },
            onclick: move |e| { e.stop_propagation(); },
            onwheel: move |e| { e.stop_propagation(); },

            // Header with delete button
            div {
                class: "annotation-menu-header",
                
                // Delete button in header
                button {
                    class: "annotation-delete-btn",
                    onclick: make_delete_handler(
                        pid_arc.clone(), bid_arc.clone(), iid_arc.clone(),
                        ver_base.clone(), open_base.clone(), tgt_base.clone(),
                        z_base.clone(), px_base.clone(), py_base.clone(),
                    ),
                    img { src: asset!("/assets/icons/trash.svg") }
                }
            }

            // Divider
            div {
                class: "annotation-menu-divider"
            }

            // Class selector list
            div {
                class: "annotation-class-list",
                
                for ci in classes.iter().cloned() {{
                    let class_id = ci.id.clone();
                    let class_name = ci.name.clone();
                    let class_color = ci.color.clone();
                    let pid = pid_arc.clone();
                    let bid = bid_arc.clone();
                    let iid = iid_arc.clone();
                    let tgt = tgt_base;
                    let mut ver = ver_base;
                    let mut opn = open_base;
                    let zm = z_base;
                    let px = px_base;
                    let py = py_base;
                    
                    rsx! {
                        div {
                            key: "{class_id}",
                            class: "annotation-class-item",
                            onclick: move |_| {
                                selected_class.set(class_id.clone());
                                tracing::info!("Class item clicked: {}", class_id);
                                
                                if let Some(t) = tgt() {
                                    match t {
                                        AnnotationTarget::Poly(idx) => {
                                            let polys_before = load_polygons(&*pid, &*bid, &*iid);
                                            tracing::info!("BEFORE update - Poly[{}] class: {:?}", idx, polys_before.get(idx).map(|p| &p.class_id));
                                            
                                            if let Some(old) = polys_before.get(idx) {
                                                increment_class_count(&*pid, &old.class_id, -1);
                                            }
                                            let updated = update_polygon_class(&*pid, &*bid, &*iid, idx, &class_id);
                                            tracing::info!("update_polygon_class returned: {}", updated);
                                            
                                            if updated {
                                                increment_class_count(&*pid, &class_id, 1);
                                            }
                                            invalidate_polygon_cache();
                                            
                                            let polys_after = load_polygons(&*pid, &*bid, &*iid);
                                            tracing::info!("AFTER update - Poly[{}] class: {:?}", idx, polys_after.get(idx).map(|p| &p.class_id));
                                        }
                                        AnnotationTarget::BBox(idx) => {
                                            let bboxes_before = load_bboxes(&*pid, &*bid, &*iid);
                                            tracing::info!("BEFORE update - BBox[{}] class: {:?}", idx, bboxes_before.get(idx).map(|b| &b.class_id));
                                            
                                            if let Some(old) = bboxes_before.get(idx) {
                                                increment_class_count(&*pid, &old.class_id, -1);
                                            }
                                            let updated = update_bbox_class(&*pid, &*bid, &*iid, idx, &class_id);
                                            tracing::info!("update_bbox_class returned: {}", updated);
                                            
                                            if updated {
                                                increment_class_count(&*pid, &class_id, 1);
                                            }
                                            invalidate_bbox_cache();
                                            
                                            let bboxes_after = load_bboxes(&*pid, &*bid, &*iid);
                                            tracing::info!("AFTER update - BBox[{}] class: {:?}", idx, bboxes_after.get(idx).map(|b| &b.class_id));
                                        }
                                    }
                                    // Invalidation already done above, now trigger redraw via use_effect
                                    ver.set(ver() + 1);
                                    opn.set(false);
                                }
                            },
                            
                            span { 
                                class: "class-color-dot",
                                style: format_args!("background: {};", class_color)
                            }
                            span { "{class_name}" }
                        }
                    }
                }}
            }
        }
    }
}
