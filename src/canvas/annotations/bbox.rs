use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BBoxAction {
    SetStart(Point),
    SetEnd(Point),
}

#[derive(Clone, Debug, PartialEq)]
pub struct BBox {
    pub start_point: Option<Point>,
    pub end_point: Option<Point>,
    pub is_complete: bool,
    pub preview_point: Option<Point>, // For live preview of box
    pub undo_history: Vec<BBoxAction>, // Stack of undone actions
}

impl BBox {
    pub fn new() -> Self {
        Self {
            start_point: None,
            end_point: None,
            is_complete: false,
            preview_point: None,
            undo_history: Vec::new(),
        }
    }

    pub fn set_start(&mut self, p: Point) {
        self.start_point = Some(p);
        self.is_complete = false;
        // Clear redo history when new action is taken
        self.undo_history.clear();
    }

    pub fn set_end(&mut self, p: Point) {
        self.end_point = Some(p);
        self.is_complete = true;
        self.preview_point = None;
        // Clear redo history when new action is taken
        self.undo_history.clear();
    }

    pub fn set_preview(&mut self, p: Option<Point>) {
        self.preview_point = p;
    }

    pub fn undo(&mut self) -> bool {
        if self.end_point.is_some() {
            // Undo end point
            if let Some(end) = self.end_point.take() {
                self.undo_history.push(BBoxAction::SetEnd(end));
                self.is_complete = false;
                return true;
            }
        } else if self.start_point.is_some() {
            // Undo start point
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
                    self.preview_point = None;
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
        self.preview_point = None;
        self.undo_history.clear();
    }

    pub fn can_undo(&self) -> bool {
        self.start_point.is_some()
    }

    pub fn can_redo(&self) -> bool {
        !self.undo_history.is_empty()
    }
}

/* ---- styles (same as polygon) ---- */
const POINT_STROKE: &str = "rgb(51, 66, 255)";
const POINT_FILL: &str = "rgba(51, 66, 255, 0.4)";
const POINT_RADIUS: f64 = 18.0;
const ZOOMED_IN_RADIUS: f64 = 9.0;
const ZOOMED_OUT_RADIUS: f64 = 3.0;
const LINE_COLOR: &str = "rgb(51, 66, 255)";
const LINE_WIDTH: f64 = 5.0;
const FILL_COLOR: &str = "rgba(51, 66, 255, 0.5)";
const PREVIEW_LINE_COLOR: &str = "rgba(51, 66, 255, 0.6)";
const PREVIEW_FILL_COLOR: &str = "rgba(51, 66, 255, 0.3)";

fn clear_canvas(ctx: &CanvasRenderingContext2d) {
    if let Some(canvas) = ctx.canvas() {
        // Clear entire device buffer regardless of current transform
        let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    }
}

// Draw a point with 40% opacity fill
fn draw_point(ctx: &CanvasRenderingContext2d, p: Point, radius: f64) {
    ctx.set_fill_style(&JsValue::from_str(POINT_FILL));
    ctx.set_stroke_style(&JsValue::from_str(POINT_STROKE));
    ctx.set_line_width(2.0);

    ctx.begin_path();
    let _ = ctx.arc(p.x, p.y, radius, 0.0, std::f64::consts::PI * 2.0);
    ctx.fill();
    ctx.stroke();
}

// Draw bounding box outline
fn draw_bbox_outline(ctx: &CanvasRenderingContext2d, start: Point, end: Point, is_preview: bool) {
    let x = start.x.min(end.x);
    let y = start.y.min(end.y);
    let width = (end.x - start.x).abs();
    let height = (end.y - start.y).abs();

    if is_preview {
        ctx.set_stroke_style(&JsValue::from_str(PREVIEW_LINE_COLOR));
    } else {
        ctx.set_stroke_style(&JsValue::from_str(LINE_COLOR));
    }
    ctx.set_line_width(LINE_WIDTH);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");

    ctx.begin_path();
    ctx.rect(x, y, width, height);
    ctx.stroke();
}

// Draw filled bounding box
fn fill_bbox(ctx: &CanvasRenderingContext2d, start: Point, end: Point, is_preview: bool) {
    let x = start.x.min(end.x);
    let y = start.y.min(end.y);
    let width = (end.x - start.x).abs();
    let height = (end.y - start.y).abs();

    // Set fill style
    if is_preview {
        ctx.set_fill_style(&JsValue::from_str(PREVIEW_FILL_COLOR));
        ctx.set_stroke_style(&JsValue::from_str(PREVIEW_LINE_COLOR));
    } else {
        ctx.set_fill_style(&JsValue::from_str(FILL_COLOR));
        ctx.set_stroke_style(&JsValue::from_str(LINE_COLOR));
    }
    ctx.set_line_width(LINE_WIDTH);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");

    ctx.begin_path();
    ctx.rect(x, y, width, height);
    ctx.fill();
    ctx.stroke();
}

pub fn redraw_bbox(
    ctx: &CanvasRenderingContext2d,
    bbox: &BBox,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) {
    clear_canvas(ctx);

    // Apply combined DPR*zoom transform once
    if let Some(win) = web_sys::window() {
        let dpr = win.device_pixel_ratio();
        let _ = ctx.set_transform(zoom * dpr, 0.0, 0.0, zoom * dpr, pan_x * dpr, pan_y * dpr);
    }

    // Draw in WORLD coordinates; transform handles screen mapping
    if let Some(start) = bbox.start_point {
        // Adjust radius to keep roughly constant on screen
        let radius = (POINT_RADIUS / zoom).clamp(ZOOMED_OUT_RADIUS / zoom, ZOOMED_IN_RADIUS / zoom);

        // Draw start point
        draw_point(ctx, start, radius);

        if bbox.is_complete {
            if let Some(end) = bbox.end_point {
                fill_bbox(ctx, start, end, false);
                draw_point(ctx, end, radius);
            }
        } else if let Some(preview) = bbox.preview_point {
            draw_bbox_outline(ctx, start, preview, true);
        }
    }

    // Reset transform to identity to avoid leaking state
    let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
}

// Handle bbox click
pub fn on_bbox_click(
    world_x: f64,
    world_y: f64,
    bbox: &mut BBox,
    ctx: &CanvasRenderingContext2d,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) {
    tracing::info!("BBox clicked world x: {} and y: {}", world_x, world_y);

    if bbox.start_point.is_none() {
        // First click - set start point
        bbox.set_start(Point {
            x: world_x,
            y: world_y,
        });
    } else if !bbox.is_complete {
        // Second click - set end point and complete
        bbox.set_end(Point {
            x: world_x,
            y: world_y,
        });
    } else {
        // Box is complete, start a new one
        bbox.reset();
        bbox.set_start(Point {
            x: world_x,
            y: world_y,
        });
    }

    // Immediate visual feedback on the overlay canvas
    redraw_bbox(ctx, bbox, zoom, pan_x, pan_y);
    redraw_bbox(ctx, bbox, zoom, pan_x, pan_y);
}
