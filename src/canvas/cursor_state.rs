use dioxus::prelude::*;

/// Cursor state signals for custom crosshair cursor
#[derive(Clone, Copy)]
pub struct CursorState {
    pub guide_x: Signal<f64>,
    pub guide_y: Signal<f64>,
    pub cursor_x: Signal<f64>,
    pub cursor_y: Signal<f64>,
}

impl CursorState {
    pub fn new() -> Self {
        Self {
            guide_x: use_signal(|| 0.0),
            guide_y: use_signal(|| 0.0),
            cursor_x: use_signal(|| 0.0),
            cursor_y: use_signal(|| 0.0),
        }
    }
}
