//! Shared geometry types used across the application
//!
//! This file contains ALL shape/geometry types in one place:
//! - Point, Polygon, BBox: Used by canvas for drawing state
//! - Geometry enum: Used ONLY for API serialization (tagged union)
//!
//! Why the Geometry enum exists:
//! The backend API stores "either a polygon OR a bbox" in a single JSON field.
//! The JSON looks like: {"type": "polygon", "points": [...]}
//! This is called a "tagged union" - we need the "type" field to know which shape it is.
//! The canvas code works with Polygon and BBox directly - the enum is only for API serialization.

use serde::{Deserialize, Serialize};

/// A 2D point in world coordinates
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// A polygon annotation (closed or in-progress)
///
/// Used by canvas for drawing state. Includes undo/redo history
/// which is NOT sent to the backend.
#[derive(Clone, Debug, PartialEq)]
pub struct Polygon {
    pub points: Vec<Point>,
    pub is_closed: bool,
    pub undo_history: Vec<Point>,
}

impl Polygon {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            is_closed: false,
            undo_history: Vec::new(),
        }
    }

    pub fn add(&mut self, p: Point) {
        self.points.push(p);
        self.undo_history.clear();
    }

    pub fn close(&mut self) {
        if self.points.len() >= 3 {
            self.is_closed = true;
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(point) = self.points.pop() {
            self.undo_history.push(point);
            self.is_closed = false;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(point) = self.undo_history.pop() {
            self.points.push(point);
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.points.clear();
        self.is_closed = false;
        self.undo_history.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.points.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.undo_history.is_empty()
    }
}

/// Actions that can be performed on a bounding box (for undo/redo)
#[derive(Clone, Debug, PartialEq)]
pub enum BBoxAction {
    SetStart(Point),
    SetEnd(Point),
}

/// A bounding box annotation (rectangular)
///
/// Used by canvas for drawing state. Includes undo/redo history
/// which is NOT sent to the backend.
#[derive(Clone, Debug, PartialEq)]
pub struct BBox {
    pub start_point: Option<Point>,
    pub end_point: Option<Point>,
    pub is_complete: bool,
    pub undo_history: Vec<BBoxAction>,
}

impl BBox {
    pub fn new() -> Self {
        Self {
            start_point: None,
            end_point: None,
            is_complete: false,
            undo_history: Vec::new(),
        }
    }

    pub fn set_start(&mut self, p: Point) {
        self.start_point = Some(p);
        self.is_complete = false;
        self.undo_history.clear();
    }

    pub fn set_end(&mut self, p: Point) {
        self.end_point = Some(p);
        self.is_complete = true;
        self.undo_history.clear();
    }

    pub fn undo(&mut self) -> bool {
        if self.end_point.is_some() {
            if let Some(end) = self.end_point.take() {
                self.undo_history.push(BBoxAction::SetEnd(end));
                self.is_complete = false;
                return true;
            }
        } else if self.start_point.is_some() {
            if let Some(start) = self.start_point.take() {
                self.undo_history.push(BBoxAction::SetStart(start));
                return true;
            }
        }
        false
    }

    pub fn redo(&mut self) -> bool {
        if let Some(action) = self.undo_history.pop() {
            match action {
                BBoxAction::SetStart(p) => {
                    self.start_point = Some(p);
                    self.is_complete = false;
                }
                BBoxAction::SetEnd(p) => {
                    self.end_point = Some(p);
                    self.is_complete = true;
                }
            }
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.start_point = None;
        self.end_point = None;
        self.is_complete = false;
        self.undo_history.clear();
    }

    pub fn can_undo(&self) -> bool {
        self.start_point.is_some()
    }

    pub fn can_redo(&self) -> bool {
        !self.undo_history.is_empty()
    }
}

/// Geometry enum for API serialization ONLY
///
/// This enum exists because the backend API stores "either a polygon OR a bbox"
/// in a single JSON field with a "type" tag:
///
/// ```json
/// {
///   "geometry": {
///     "type": "polygon",
///     "points": [{"x": 0, "y": 0}, ...]
///   }
/// }
/// ```
///
/// The canvas code doesn't use this enum - it works with Polygon and BBox directly.
/// This is ONLY for deserializing API responses and serializing API requests.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Geometry {
    #[serde(rename = "polygon")]
    Polygon { points: Vec<Point> },
    #[serde(rename = "bbox")]
    BBox { start: Point, end: Point },
}
