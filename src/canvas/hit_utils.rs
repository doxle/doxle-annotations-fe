use super::annotations::store::{SavedBBox, SavedPoint};

// Hit test helpers (world coords)
pub fn point_in_poly(x: f64, y: f64, pts: &Vec<SavedPoint>) -> bool {
    let mut inside = false;
    let mut j = pts.len().wrapping_sub(1);
    for i in 0..pts.len() {
        let xi = pts[i].x;
        let yi = pts[i].y;
        let xj = pts[j].x;
        let yj = pts[j].y;
        let intersect =
            ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi + 1e-9) + xi);
        if intersect {
            inside = !inside;
        }
        j = i;
    }
    inside
}

pub fn point_in_bbox(x: f64, y: f64, bb: &SavedBBox) -> bool {
    let l = bb.start.x.min(bb.end.x);
    let r = bb.start.x.max(bb.end.x);
    let t = bb.start.y.min(bb.end.y);
    let b = bb.start.y.max(bb.end.y);
    x >= l && x <= r && y >= t && y <= b
}
