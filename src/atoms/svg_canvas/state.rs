use dioxus::prelude::*;

// ============================================
// Transform (pan/zoom math)
// ============================================

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub x: f64,      // pan x (screen pixels)
    pub y: f64,      // pan y (screen pixels)
    pub zoom: f64,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
        }
    }
}

impl Transform {
    // screen = world * zoom + pan
    // world = (screen - pan) / zoom

    pub fn world_to_screen(&self, wx: f64, wy: f64) -> (f64, f64) {
        let sx = wx * self.zoom + self.x;
        let sy = wy * self.zoom + self.y;
        (sx, sy)
    }

    pub fn screen_to_world(&self, sx: f64, sy: f64) -> (f64, f64) {
        let wx = (sx - self.x) / self.zoom;
        let wy = (sy - self.y) / self.zoom;
        (wx, wy)
    }

    pub fn zoom_at_screen_point(&mut self, factor: f64, anchor_sx: f64, anchor_sy: f64) {
        let (wx, wy) = self.screen_to_world(anchor_sx, anchor_sy);
        let new_zoom = (self.zoom * factor).clamp(0.1, 10.0);
        self.x = anchor_sx - wx * new_zoom;
        self.y = anchor_sy - wy * new_zoom;
        self.zoom = new_zoom;
    }

    pub fn pan_by(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }
}

// ============================================
// Tool
// ============================================

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tool {
    Select,
    Pan,
    Polygon,
    BBox,
}

// ============================================
// Canvas State (pan/zoom only - block adds its own signals)
// ============================================

pub struct CanvasState {
    pub transform: Signal<Transform>,
    pub selected_tool: Signal<Tool>,
}

impl CanvasState {
    pub fn new() -> Self {
        Self {
            transform: Signal::new(Transform::default()),
            selected_tool: Signal::new(Tool::Polygon),
        }
    }

    pub fn zoom(&mut self, factor: f64, anchor_sx: f64, anchor_sy: f64) {
        self.transform.write().zoom_at_screen_point(factor, anchor_sx, anchor_sy);
    }

    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.transform.write().pan_by(dx, dy);
    }

    pub fn set_view(&mut self, zoom: f64, pan_x: f64, pan_y: f64) {
        let mut t = self.transform.write();
        t.zoom = zoom;
        t.x = pan_x;
        t.y = pan_y;
    }

    pub fn reset_view(&mut self) {
        let mut t = self.transform.write();
        t.x = 0.0;
        t.y = 0.0;
        t.zoom = 1.0;
    }

    pub fn toggle_tool(&mut self) {
        let current = *self.selected_tool.read();
        let next = match current {
            Tool::Polygon => Tool::Pan,
            _ => Tool::Polygon,
        };
        self.selected_tool.set(next);
    }

    pub fn set_tool(&mut self, tool: Tool) {
        self.selected_tool.set(tool);
    }
}
