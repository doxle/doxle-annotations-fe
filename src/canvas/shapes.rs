//! Pure data structures for canvas annotations.
//!
//! This file contains ONLY data structures and their methods.
//! No DOM access, no drawing logic - just geometry and state management.
//!
//! By separating shapes from drawing logic, we make the code:
//! - Easier to test (no web_sys dependencies)
//! - Easier to understand (clear separation of concerns)
//! - More reusable (can use Point/Polygon/BBox in other contexts)

/// A 2D point in world coordinates
///
/// Used by both Polygon and BBox to represent vertices.
/// Before: We had duplicate Point structs in polygon.rs AND bbox.rs
/// After: Single source of truth here

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Create a new point
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// A polygon annotation (closed or in-progress)
///
/// Polygons are drawn by clicking multiple points, then closing the shape
/// by clicking near the first point.
#[derive(Clone, Debug, PartialEq)]
pub struct Polygon {
    /// The vertices of the polygon in world coordinates
    pub points: Vec<Point>,

    /// Whether the polygon has been closed (completed)
    pub is_closed: bool,

    /// Stack of undone points for redo functionality
    /// When user undos, we move points here. When they redo, we pop from here.
    pub undo_history: Vec<Point>,
}

impl Polygon {
    /// Create a new empty polygon
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            is_closed: false,
            undo_history: Vec::new(),
        }
    }

    /// Add a point to the polygon
    /// Clears redo history (can't redo after new action)
    pub fn add(&mut self, p: Point) {
        self.points.push(p);
        self.undo_history.clear();
    }

    /// Close the polygon (marks it as complete)
    /// Requires at least 3 points to form a valid polygon
    pub fn close(&mut self) {
        if self.points.len() >= 3 {
            self.is_closed = true;
        }
    }
    /// Remove the last point and store it for redo
    /// Returns true if a point was undone, false if nothing to undo
    pub fn undo(&mut self) -> bool {
        if let Some(point) = self.points.pop() {
            self.undo_history.push(point);
            self.is_closed = false; // Reopen if was closed
            true
        } else {
            false
        }
    }

    /// Restore the last undone point
    /// Returns true if a point was redone, false if nothing to redo
    pub fn redo(&mut self) -> bool {
        if let Some(point) = self.undo_history.pop() {
            self.points.push(point);
            true
        } else {
            false
        }
    }

    /// Clear the polygon (start fresh)
    pub fn reset(&mut self) {
        self.points.clear();
        self.is_closed = false;
        self.undo_history.clear();
    }

    /// Check if there are any points that can be undone
    pub fn can_undo(&self) -> bool {
        !self.points.is_empty()
    }

    /// Check if there are any points that can be redone
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
/// BBoxes are drawn with two clicks: start corner, then end corner.
/// Unlike polygons, they're always rectangular (axis-aligned).
#[derive(Clone, Debug, PartialEq)]
pub struct BBox {
    /// The first corner clicked (can be any corner)
    pub start_point: Option<Point>,

    /// The second corner clicked (opposite corner from start)
    pub end_point: Option<Point>,

    /// Whether both corners have been set (box is complete)
    pub is_complete: bool,

    /// Stack of undone actions for redo functionality
    pub undo_history: Vec<BBoxAction>,
}

impl BBox {
    /// Create a new empty bounding box
    pub fn new() -> Self {
        Self {
            start_point: None,
            end_point: None,
            is_complete: false,
            undo_history: Vec::new(),
        }
    }

    /// Set the first corner of the box
    pub fn set_start(&mut self, p: Point) {
        self.start_point = Some(p);
        self.is_complete = false;
        self.undo_history.clear();
    }

    /// Set the second corner of the box (completes it)
    pub fn set_end(&mut self, p: Point) {
        self.end_point = Some(p);
        self.is_complete = true;
        self.undo_history.clear();
    }
    /// Undo the last action (remove end point, then start point)
    /// Returns true if something was undone, false if nothing to undo
    pub fn undo(&mut self) -> bool {
        if self.end_point.is_some() {
            // Undo end point first
            if let Some(end) = self.end_point.take() {
                self.undo_history.push(BBoxAction::SetEnd(end));
                self.is_complete = false;
                return true;
            }
        } else if self.start_point.is_some() {
            // Then undo start point
            if let Some(start) = self.start_point.take() {
                self.undo_history.push(BBoxAction::SetStart(start));
                return true;
            }
        }
        false
    }
    /// Restore the last undone action
    /// Returns true if something was redone, false if nothing to redo
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
    /// Clear the bounding box (start fresh)
    pub fn reset(&mut self) {
        self.start_point = None;
        self.end_point = None;
        self.is_complete = false;
        self.undo_history.clear();
    }

    /// Check if there's anything that can be undone
    pub fn can_undo(&self) -> bool {
        self.start_point.is_some()
    }

    /// Check if there's anything that can be redone
    pub fn can_redo(&self) -> bool {
        !self.undo_history.is_empty()
    }
}
