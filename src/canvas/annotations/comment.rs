use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Comment {
    pub position: Option<Point>,
    pub text: String,
    pub is_complete: bool,
}

impl Comment {
    pub fn new() -> Self {
        Self {
            position: None,
            text: String::new(),
            is_complete: false,
        }
    }

    pub fn set_position(&mut self, p: Point) {
        self.position = Some(p);
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.is_complete = true;
    }

    pub fn reset(&mut self) {
        self.position = None;
        self.text = String::new();
        self.is_complete = false;
    }
}

/* ---- styles (same as polygon/bbox) ---- */
const POINT_STROKE: &str = "rgb(51, 66, 255)";
const POINT_FILL: &str = "rgba(51, 66, 255, 0.4)";
const POINT_RADIUS: f64 = 18.0;
const TEXT_COLOR: &str = "rgb(51, 66, 255)";
const TEXT_BG: &str = "rgba(255, 255, 255, 0.9)";

fn clear_canvas(ctx: &CanvasRenderingContext2d) {
    if let Some(canvas) = ctx.canvas() {
        let window = web_sys::window().expect("Should get window");
        let dpr = window.device_pixel_ratio();
        let css_w = canvas.width() as f64 / dpr;
        let css_h = canvas.height() as f64 / dpr;
        ctx.clear_rect(0.0, 0.0, css_w, css_h);
    }
}

// Draw a point marker for comment
fn draw_point(ctx: &CanvasRenderingContext2d, p: Point) {
    ctx.set_fill_style(&JsValue::from_str(POINT_FILL));
    ctx.set_stroke_style(&JsValue::from_str(POINT_STROKE));
    ctx.set_line_width(2.0);

    ctx.begin_path();
    let _ = ctx.arc(p.x, p.y, POINT_RADIUS, 0.0, std::f64::consts::PI * 2.0);
    ctx.fill();
    ctx.stroke();
}

// Draw comment text
fn draw_comment_text(ctx: &CanvasRenderingContext2d, p: Point, text: &str) {
    // Draw background
    ctx.set_fill_style(&JsValue::from_str(TEXT_BG));
    ctx.set_stroke_style(&JsValue::from_str(POINT_STROKE));
    ctx.set_line_width(2.0);

    let padding = 8.0;
    let text_width = 150.0; // Fixed width for now
    let text_height = 30.0;

    ctx.fill_rect(
        p.x + POINT_RADIUS + 5.0,
        p.y - text_height / 2.0,
        text_width,
        text_height,
    );
    ctx.stroke_rect(
        p.x + POINT_RADIUS + 5.0,
        p.y - text_height / 2.0,
        text_width,
        text_height,
    );

    // Draw text
    ctx.set_fill_style(&JsValue::from_str(TEXT_COLOR));
    ctx.set_font("14px Arial");
    let _ = ctx.fill_text(text, p.x + POINT_RADIUS + 5.0 + padding, p.y + 5.0);
}

pub fn redraw_comment(
    ctx: &CanvasRenderingContext2d,
    comment: &Comment,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) {
    clear_canvas(ctx);

    if let Some(pos) = comment.position {
        let screen_pos = Point {
            x: pos.x * zoom + pan_x,
            y: pos.y * zoom + pan_y,
        };

        draw_point(ctx, screen_pos);

        if comment.is_complete && !comment.text.is_empty() {
            draw_comment_text(ctx, screen_pos, &comment.text);
        }
    }
}

// Handle comment click
pub fn on_comment_click(
    world_x: f64,
    world_y: f64,
    comment: &mut Comment,
    ctx: &CanvasRenderingContext2d,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) {
    tracing::info!("Comment clicked world x: {} and y: {}", world_x, world_y);

    if comment.position.is_none() {
        // Set position
        comment.set_position(Point {
            x: world_x,
            y: world_y,
        });
        // TODO: Show text input dialog
        // For now, just set dummy text
        comment.set_text("Comment text".to_string());
    } else {
        // Start new comment
        comment.reset();
        comment.set_position(Point {
            x: world_x,
            y: world_y,
        });
        comment.set_text("Comment text".to_string());
    }

    redraw_comment(ctx, comment, zoom, pan_x, pan_y);
}
