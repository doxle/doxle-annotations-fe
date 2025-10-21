use wasm_bindgen::{JsCast, JsValue};
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Polygon {
    pub points: Vec<Point>,
    pub is_closed: bool,
    pub preview_point: Option<Point>, // For live preview line
    pub undo_history: Vec<Point>,     // Stack of undone points
}

impl Polygon {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            is_closed: false,
            preview_point: None,
            undo_history: Vec::new(),
        }
    }
    pub fn add(&mut self, p: Point) {
        self.points.push(p);
        // Clear redo history when new action is taken
        self.undo_history.clear();
    }
    pub fn close(&mut self) {
        if self.points.len() >= 3 {
            self.is_closed = true;
            self.preview_point = None;
        }
    }
    pub fn set_preview(&mut self, p: Option<Point>) {
        self.preview_point = p;
    }

    pub fn undo(&mut self) -> bool {
        if let Some(point) = self.points.pop() {
            self.undo_history.push(point);
            self.is_closed = false; // Reopen if was closed
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
        self.preview_point = None;
        self.undo_history.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.points.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.undo_history.is_empty()
    }
}

/* ---- styles (edit once here) ---- */
pub const POINT_STROKE: &str = "rgb(51, 66, 255)";
pub const POINT_FILL: &str = "rgba(51, 66, 255, 0.28)";
pub const POINT_RADIUS: f64 = 7.2;
pub const ZOOMED_IN_RADIUS: f64 = 9.0; // Radius when zoomed in (large)
pub const ZOOMED_OUT_RADIUS: f64 = 3.0; // Radius when zoomed out (small)
pub const ENDPOINT_STROKE: &str = "rgb(0, 255, 0)";
pub const ENDPOINT_FILL: &str = "rgba(0, 255, 0, 0.28)";
pub const LINE_COLOR: &str = "rgb(51, 66, 255)";
pub const LINE_WIDTH: f64 = 3.0;
pub const FILL_COLOR: &str = "rgba(51, 66, 255, 0.35)";
pub const PREVIEW_LINE_COLOR: &str = "rgba(51, 66, 255, 0.6)";

fn clear_canvas(ctx: &CanvasRenderingContext2d) {
    if let Some(canvas) = ctx.canvas() {
        // Clear entire device buffer regardless of current transform
        let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    }
}

//Draw a point with 40% opacity fill
pub fn draw_point(ctx: &CanvasRenderingContext2d, p: Point, radius: f64) {
    // Draw filled circle with 40% opacity
    ctx.set_fill_style(&JsValue::from_str(POINT_FILL));
    ctx.set_stroke_style(&JsValue::from_str(POINT_STROKE));
    ctx.set_line_width(2.0);

    ctx.begin_path();
    let _ = ctx.arc(p.x, p.y, radius, 0.0, std::f64::consts::PI * 2.0);
    ctx.fill();
    ctx.stroke();
}

//Draw endpoint with green color
pub fn draw_endpoint(ctx: &CanvasRenderingContext2d, p: Point, radius: f64) {
    // Draw filled circle with green color
    ctx.set_fill_style(&JsValue::from_str(ENDPOINT_FILL));
    ctx.set_stroke_style(&JsValue::from_str(ENDPOINT_STROKE));
    ctx.set_line_width(2.0);

    ctx.begin_path();
    let _ = ctx.arc(p.x, p.y, radius, 0.0, std::f64::consts::PI * 2.0);
    ctx.fill();
    ctx.stroke();
}

pub fn draw_polygon(ctx: &CanvasRenderingContext2d, pts: &[Point], close_shape: bool) {
    // Draw polyline connecting points
    if pts.is_empty() {
        return;
    }

    // If only one point, just draw the point (no line yet)
    if pts.len() < 2 {
        return;
    }

    // Set line style for continuous polyline
    ctx.set_line_width(LINE_WIDTH);
    ctx.set_stroke_style(&JsValue::from_str(LINE_COLOR));
    ctx.set_line_cap("round");
    ctx.set_line_join("round");

    ctx.begin_path();
    ctx.move_to(pts[0].x, pts[0].y);
    for p in &pts[1..] {
        ctx.line_to(p.x, p.y);
    }
    if close_shape {
        ctx.close_path();
    }
    ctx.stroke();
}

// Draw preview line from last point to cursor
pub fn draw_preview_line(ctx: &CanvasRenderingContext2d, from: Point, to: Point) {
    ctx.set_line_width(LINE_WIDTH);
    ctx.set_stroke_style(&JsValue::from_str(PREVIEW_LINE_COLOR));
    ctx.set_line_cap("round");

    ctx.begin_path();
    ctx.move_to(from.x, from.y);
    ctx.line_to(to.x, to.y);
    ctx.stroke();
}

pub fn fill_polygon(ctx: &CanvasRenderingContext2d, pts: &[Point]) {
    if pts.len() < 3 {
        return;
    }

    // Set styles for filled polygon with outline
    ctx.set_line_width(LINE_WIDTH);
    ctx.set_stroke_style(&JsValue::from_str(LINE_COLOR));
    ctx.set_fill_style(&JsValue::from_str(FILL_COLOR));
    ctx.set_line_cap("round");
    ctx.set_line_join("round");

    ctx.begin_path();
    ctx.move_to(pts[0].x, pts[0].y);
    for p in &pts[1..] {
        ctx.line_to(p.x, p.y);
    }
    ctx.close_path();
    ctx.fill();
    ctx.stroke();
}

pub fn redraw_polygon(
    ctx: &CanvasRenderingContext2d,
    poly: &Polygon,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) {
    clear_canvas(ctx); //need to clear before we redraw every frame

    // Transform world coordinates to screen coordinates for drawing
    let screen_points: Vec<Point> = poly
        .points
        .iter()
        .map(|p| Point {
            x: p.x * zoom + pan_x,
            y: p.y * zoom + pan_y,
        })
        .collect();

    if poly.is_closed {
        // If poly is closed, fill with 50% opacity
        fill_polygon(ctx, &screen_points);
    } else {
        // Draw polyline connecting all points
        draw_polygon(ctx, &screen_points, false);

        // Draw preview lines (CVAT-style)
        if let Some(preview) = poly.preview_point {
            if let Some(last) = screen_points.last() {
                let preview_screen = Point {
                    x: preview.x * zoom + pan_x,
                    y: preview.y * zoom + pan_y,
                };
                // Draw line from last point to cursor
                draw_preview_line(ctx, *last, preview_screen);

                // Draw closing line only when near first point
                if screen_points.len() >= 2 && can_close_polygon(poly, zoom, pan_x, pan_y) {
                    if let Some(first) = screen_points.first() {
                        draw_preview_line(ctx, preview_screen, *first);
                    }
                }
            }
        }
    }

    // Draw all points with zoom-adjusted radius
    let adjusted_radius = (POINT_RADIUS * zoom).clamp(ZOOMED_OUT_RADIUS, ZOOMED_IN_RADIUS);
    for (i, &p) in screen_points.iter().enumerate() {
        // First point is green when we have 3+ points and hovering near it
        if i == 0 && screen_points.len() >= 3 && can_close_polygon(poly, zoom, pan_x, pan_y) {
            draw_endpoint(ctx, p, adjusted_radius);
        } else {
            draw_point(ctx, p, adjusted_radius);
        }
    }
}

// Draw current polygon without clearing the canvas (overlay)
pub fn draw_polygon_overlay(
    ctx: &CanvasRenderingContext2d,
    poly: &Polygon,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) {
    // Transform world to screen
    let screen_points: Vec<Point> = poly
        .points
        .iter()
        .map(|p| Point { x: p.x * zoom + pan_x, y: p.y * zoom + pan_y })
        .collect();

    if poly.is_closed {
        fill_polygon(ctx, &screen_points);
    } else {
        draw_polygon(ctx, &screen_points, false);
        if let Some(preview) = poly.preview_point {
            if let Some(last) = screen_points.last() {
                let preview_screen = Point { x: preview.x * zoom + pan_x, y: preview.y * zoom + pan_y };
                draw_preview_line(ctx, *last, preview_screen);
                if screen_points.len() >= 2 && can_close_polygon(poly, zoom, pan_x, pan_y) {
                    if let Some(first) = screen_points.first() {
                        draw_preview_line(ctx, preview_screen, *first);
                    }
                }
            }
        }
    }

    // draw points
    let adjusted_radius = (POINT_RADIUS * zoom).clamp(ZOOMED_OUT_RADIUS, ZOOMED_IN_RADIUS);
    for (i, &p) in screen_points.iter().enumerate() {
        if i == 0 && screen_points.len() >= 3 && can_close_polygon(poly, zoom, pan_x, pan_y) {
            draw_endpoint(ctx, p, adjusted_radius);
        } else {
            draw_point(ctx, p, adjusted_radius);
        }
    }
}

// Check if cursor is near first point to close polygon
fn can_close_polygon(poly: &Polygon, zoom: f64, pan_x: f64, pan_y: f64) -> bool {
    if poly.points.len() < 3 {
        return false;
    }

    if let (Some(first), Some(preview)) = (poly.points.first(), poly.preview_point) {
        let first_screen = Point {
            x: first.x * zoom + pan_x,
            y: first.y * zoom + pan_y,
        };
        let preview_screen = Point {
            x: preview.x * zoom + pan_x,
            y: preview.y * zoom + pan_y,
        };

        let dx = preview_screen.x - first_screen.x;
        let dy = preview_screen.y - first_screen.y;
        let distance = (dx * dx + dy * dy).sqrt();

        // Close if within 30px of first point
        distance < 30.0
    } else {
        false
    }
}

//redraw polygon every frame with on click
pub fn on_polygon_click(
    world_x: f64,
    world_y: f64,
    poly: &mut Polygon,
    ctx: &CanvasRenderingContext2d,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) {
    // Store vertices in world coordinates
    tracing::info!("Polygon clicked world x: {} and y: {}", world_x, world_y);
    poly.add(Point {
        x: world_x,
        y: world_y,
    });
}

//Get the canvas element and its 2d context
pub fn get_canvas_context() -> Option<CanvasRenderingContext2d> {
    let window = window().expect("Should get window");
    let document = window.document().expect("Should get document");
    let canvas = document
        .get_element_by_id("canvas-annotations")
        .expect("Should get canvas-annotations");
    let canvas = canvas
        .dyn_into::<HtmlCanvasElement>()
        .expect("Should get canvas element");
    let context = canvas
        .get_context("2d")
        .ok()
        .expect("Should get 2d context")
        .expect("2d context");
    let context: CanvasRenderingContext2d = context
        .dyn_into::<CanvasRenderingContext2d>()
        .expect("Should get canvas context");
    Some(context)
}
