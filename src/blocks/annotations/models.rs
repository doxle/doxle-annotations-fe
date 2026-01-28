use serde::Deserialize;
use crate::atoms::svg_canvas::Geometry;

// ============================================
// Label (color comes from here)
// ============================================

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct Label {
    pub label_id: String,
    pub name: String,
    pub color: String,
}

// ============================================
// Annotation
// ============================================

#[derive(Clone, PartialEq, Debug)]
pub struct Annotation {
    pub id: String,
    pub label_id: String,
    pub geometry: Geometry,
}
