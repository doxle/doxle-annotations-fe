use super::edit_state::{get_edit_target, EditTarget};
use super::store::{SavedPolygon, SavedBBox, SavedPoint, save_polygon, save_bbox, load_polygons, load_bboxes, save_json, polys_key, bboxes_key};

/// Commit the current edit to storage (but keep edit mode active)
pub fn commit_edit(project_id: &str, block_id: &str, image_id: &str) -> bool {
    if let Some(mut target) = get_edit_target() {
        match &mut target {
            EditTarget::Polygon { index, points, class_id } => {
                // Load all polygons
                let mut all_polys = load_polygons(project_id, block_id, image_id);
                
                // Update the edited polygon
                if let Some(poly) = all_polys.get_mut(*index) {
                    poly.points = points.iter().map(|p| SavedPoint { x: p.x, y: p.y }).collect();
                    poly.class_id = class_id.clone();
                    
                    // Save back to storage
                    save_json(&polys_key(project_id, block_id, image_id), &all_polys);
                    
                    // Update the edit target with new snapshot but DON'T clear edit mode
                    use super::edit_state::update_edit_snapshot;
                    update_edit_snapshot(target);
                    return true;
                }
            }
            EditTarget::BBox { index, start, end, class_id } => {
                // Load all bboxes
                let mut all_bboxes = load_bboxes(project_id, block_id, image_id);
                
                // Update the edited bbox
                if let Some(bbox) = all_bboxes.get_mut(*index) {
                    bbox.start = SavedPoint { x: start.x, y: start.y };
                    bbox.end = SavedPoint { x: end.x, y: end.y };
                    bbox.class_id = class_id.clone();
                    
                    // Save back to storage
                    save_json(&bboxes_key(project_id, block_id, image_id), &all_bboxes);
                    
                    // Update the edit target with new snapshot but DON'T clear edit mode
                    use super::edit_state::update_edit_snapshot;
                    update_edit_snapshot(target);
                    return true;
                }
            }
        }
    }
    false
}
