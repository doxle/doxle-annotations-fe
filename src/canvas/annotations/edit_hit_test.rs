use crate::shared::shapes::Point;

const NODE_HIT_RADIUS: f64 = 10.0; // Screen pixels
const EDGE_HIT_THRESHOLD: f64 = 8.0; // Screen pixels

/// Check if a point is near a node (vertex)
pub fn hit_test_node(mouse: Point, nodes: &[Point], zoom: f64) -> Option<usize> {
    let threshold_world = NODE_HIT_RADIUS / zoom;
    let threshold_sq = threshold_world * threshold_world;
    
    for (i, node) in nodes.iter().enumerate() {
        let dx = mouse.x - node.x;
        let dy = mouse.y - node.y;
        let dist_sq = dx * dx + dy * dy;
        
        if dist_sq <= threshold_sq {
            return Some(i);
        }
    }
    None
}

/// Check if a point is near an edge (line segment)
pub fn hit_test_edge(mouse: Point, nodes: &[Point], zoom: f64, closed: bool) -> Option<usize> {
    if nodes.len() < 2 {
        return None;
    }
    
    let threshold_world = EDGE_HIT_THRESHOLD / zoom;
    let threshold_sq = threshold_world * threshold_world;
    
    let num_edges = if closed { nodes.len() } else { nodes.len() - 1 };
    
    for i in 0..num_edges {
        let p1 = nodes[i];
        let p2 = if closed && i == nodes.len() - 1 {
            nodes[0]
        } else {
            nodes[i + 1]
        };
        
        let dist_sq = point_to_segment_sq(mouse, p1, p2);
        if dist_sq <= threshold_sq {
            return Some(i);
        }
    }
    None
}

/// Hit test bbox corners (4 corners)
pub fn hit_test_bbox_corner(mouse: Point, start: Point, end: Point, zoom: f64) -> Option<usize> {
    let threshold_world = NODE_HIT_RADIUS / zoom;
    let threshold_sq = threshold_world * threshold_world;
    
    // Check 4 corners: top-left(0), top-right(1), bottom-right(2), bottom-left(3)
    let corners = [
        Point::new(start.x.min(end.x), start.y.min(end.y)), // top-left
        Point::new(start.x.max(end.x), start.y.min(end.y)), // top-right
        Point::new(start.x.max(end.x), start.y.max(end.y)), // bottom-right
        Point::new(start.x.min(end.x), start.y.max(end.y)), // bottom-left
    ];
    
    for (i, corner) in corners.iter().enumerate() {
        let dx = mouse.x - corner.x;
        let dy = mouse.y - corner.y;
        let dist_sq = dx * dx + dy * dy;
        
        if dist_sq <= threshold_sq {
            return Some(i);
        }
    }
    None
}

/// Hit test bbox edges
pub fn hit_test_bbox_edge(mouse: Point, start: Point, end: Point, zoom: f64) -> Option<usize> {
    let threshold_world = EDGE_HIT_THRESHOLD / zoom;
    let threshold_sq = threshold_world * threshold_world;
    
    let left = start.x.min(end.x);
    let right = start.x.max(end.x);
    let top = start.y.min(end.y);
    let bottom = start.y.max(end.y);
    
    // Check 4 edges: top(0), right(1), bottom(2), left(3)
    let edges = [
        (Point::new(left, top), Point::new(right, top)),       // top
        (Point::new(right, top), Point::new(right, bottom)),   // right
        (Point::new(right, bottom), Point::new(left, bottom)), // bottom
        (Point::new(left, bottom), Point::new(left, top)),     // left
    ];
    
    for (i, (p1, p2)) in edges.iter().enumerate() {
        let dist_sq = point_to_segment_sq(mouse, *p1, *p2);
        if dist_sq <= threshold_sq {
            return Some(i);
        }
    }
    None
}

/// Calculate squared distance from point to line segment
fn point_to_segment_sq(p: Point, a: Point, b: Point) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    
    if dx == 0.0 && dy == 0.0 {
        // a and b are the same point
        let dx = p.x - a.x;
        let dy = p.y - a.y;
        return dx * dx + dy * dy;
    }
    
    // Parameter t represents position along segment [0,1]
    let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / (dx * dx + dy * dy);
    let t = t.max(0.0).min(1.0);
    
    // Closest point on segment
    let closest_x = a.x + t * dx;
    let closest_y = a.y + t * dy;
    
    // Distance from p to closest point
    let dx = p.x - closest_x;
    let dy = p.y - closest_y;
    dx * dx + dy * dy
}