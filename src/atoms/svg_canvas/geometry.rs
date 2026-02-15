use serde::{Deserialize, Serialize};

// ============================================
// Shared Geometry Primitives
// Used by all blocks that use drawing canvases
// ============================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Geometry {
    #[serde(rename="polygon")]
    Polygon { points: Vec<Point> },
    #[serde(rename="bbox")]
    BBox { start: Point, end: Point },
}
