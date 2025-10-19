pub mod bbox;
pub mod polygon;

pub use bbox::*;
pub use polygon::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AnnotationTool {
    Polygon,
    BoundingBox,
}
