use web_sys::CanvasRenderingContext2d;
use super::shapes::Point;
use super::edit_state::{EditTarget, get_edit_target};

const NODE_RADIUS: f64 = 5.0; // Base radius in screen pixels
const SELECTION_WIDTH: f64 = 5.0; // Selection ring width

/// Draw edit overlay for selected annotation
pub fn draw_edit_overlay(ctx: &CanvasRenderingContext2d, zoom: f64, is_dark_mode: bool) {
    if let Some(target) = get_edit_target() {
        match target {
            EditTarget::Polygon { points, .. } => {
                draw_editable_polygon(ctx, &points, zoom, is_dark_mode);
            }
            EditTarget::BBox { start, end, .. } => {
                draw_editable_bbox(ctx, start, end, zoom, is_dark_mode);
            }
        }
    }
}

/// Draw a polygon with editable nodes
fn draw_editable_polygon(ctx: &CanvasRenderingContext2d, points: &[Point], zoom: f64, is_dark_mode: bool) {
    if points.len() < 2 {
        return;
    }
    
    // Selection ring color
    let ring_color = if is_dark_mode { "white" } else { "black" };
    
    // Draw thick selection outline
    ctx.save();
    ctx.set_line_width(SELECTION_WIDTH / zoom);
    let _ = ctx.set_stroke_style_str(ring_color);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    
    ctx.begin_path();
    ctx.move_to(points[0].x, points[0].y);
    for p in &points[1..] {
        ctx.line_to(p.x, p.y);
    }
    ctx.close_path();
    ctx.stroke();
    
    // Draw the normal polygon on top
    ctx.set_line_width(3.0 / zoom);
    let _ = ctx.set_stroke_style_str("rgb(51, 133, 255)");
    let _ = ctx.set_fill_style_str("rgba(51, 133, 255, 0.2)");
    
    ctx.begin_path();
    ctx.move_to(points[0].x, points[0].y);
    for p in &points[1..] {
        ctx.line_to(p.x, p.y);
    }
    ctx.close_path();
    ctx.fill();
    ctx.stroke();
    
    // Draw draggable nodes
    draw_nodes(ctx, points, zoom, is_dark_mode);
    
    let _ = ctx.restore();
}

/// Draw a bbox with editable corners
fn draw_editable_bbox(ctx: &CanvasRenderingContext2d, start: Point, end: Point, zoom: f64, is_dark_mode: bool) {
    let x = start.x.min(end.x);
    let y = start.y.min(end.y);
    let w = (end.x - start.x).abs();
    let h = (end.y - start.y).abs();
    
    // Selection ring color
    let ring_color = if is_dark_mode { "white" } else { "black" };
    
    // Draw thick selection outline
    ctx.save();
    ctx.set_line_width(SELECTION_WIDTH / zoom);
    let _ = ctx.set_stroke_style_str(ring_color);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    
    ctx.begin_path();
    ctx.rect(x, y, w, h);
    ctx.stroke();
    
    // Draw the normal bbox on top
    ctx.set_line_width(3.0 / zoom);
    let _ = ctx.set_stroke_style_str("rgb(51, 133, 255)");
    let _ = ctx.set_fill_style_str("rgba(51, 133, 255, 0.15)");
    
    ctx.begin_path();
    ctx.rect(x, y, w, h);
    ctx.fill();
    ctx.stroke();
    
    // Draw corner handles
    let corners = [
        Point::new(x, y),         // top-left
        Point::new(x + w, y),     // top-right
        Point::new(x + w, y + h), // bottom-right
        Point::new(x, y + h),     // bottom-left
    ];
    draw_nodes(ctx, &corners, zoom, is_dark_mode);
    
    let _ = ctx.restore();
}

/// Draw draggable node handles
fn draw_nodes(ctx: &CanvasRenderingContext2d, points: &[Point], zoom: f64, is_dark_mode: bool) {
    let node_radius = NODE_RADIUS / zoom;
    
    for point in points {
        // White/black ring
        ctx.set_line_width(2.0 / zoom);
        let _ = ctx.set_stroke_style_str(if is_dark_mode { "white" } else { "black" });
        let _ = ctx.set_fill_style_str("white");
        
        ctx.begin_path();
        let _ = ctx.arc(point.x, point.y, node_radius, 0.0, std::f64::consts::TAU);
        ctx.fill();
        ctx.stroke();
        
        // Inner colored dot
        let _ = ctx.set_fill_style_str("rgb(51, 133, 255)");
        ctx.begin_path();
        let _ = ctx.arc(point.x, point.y, node_radius * 0.6, 0.0, std::f64::consts::TAU);
        ctx.fill();
    }
}