/*

AnnotationCanvasPage (blocks/annotations/canvas_page.rs)
└─ SvgCanvas (atoms/svg_canvas/svg_canvas.rs)
   ├─ world-container (transformed)
   │  ├─ ImageLayer (blocks/annotations/image_layer.rs)
   │  └─ AnnotationsLayer (blocks/annotations/annotations_layer.rs)
   ├─ GridOverlay (atoms/svg_canvas/grid_overlay.rs)
   ├─ CrosshairOverlay (atoms/svg_canvas/crosshair_overlay.rs)
   └─ EventLayer (atoms/svg_canvas/event_layer.rs)
      └─ uses Transform/Tool from atoms/svg_canvas/state.rs

*/





use dioxus::prelude::*;
use super::models::Annotation;
use crate::atoms::svg_canvas::{Geometry, Point};
use dioxus::logger::tracing;

#[component]
pub fn AnnotationsLayer(
    width:f64,
    height:f64,
    annotations: Vec<Annotation>,
    active_drawing: Vec<(f64, f64)>,
    cursor_world_pos: (f64,f64),
    zoom: f64,
    on_right_click: EventHandler<(String, String, f64, f64)>, // (ann_id, label_id, screen_x, screen_y)
) -> Element {
    tracing::info!("AnnotationsLayer active_drawing: {:?}", active_drawing);
    
    // Scale stroke width inversely with zoom so lines stay thin when zoomed in
    let stroke_w = (0.8 / zoom).clamp(0.3, 2.0);
    let circle_r = (1.5 / zoom).clamp(0.5, 3.0);
    // Helper to format points for SVG
    let format_points = |points: &Vec<(f64, f64)>| {
        let mut result = String::new();
        for (i, (x, y)) in points.iter().enumerate() {
            if i > 0 {
                result.push(' ');
            }
            result.push_str(&format!("{},{}", x, y));
        }
        result
    };

    // Helper to format Geometry points
    let format_geometry = |geom: &Geometry| {
        match geom {
            Geometry::Polygon { points } => {
                let mut result = String::new();
                for (i, p) in points.iter().enumerate() {
                    if i > 0 {
                        result.push(' ');
                    }
                    result.push_str(&format!("{},{}", p.x, p.y));
                }
                result
            }
            Geometry::BBox { start, end } => {
                // Convert bbox to polygon points
                format!("{},{} {},{} {},{} {},{}", start.x, start.y, end.x, start.y, end.x, end.y, start.x, end.y)
            }
        }
    };

    let first_point = active_drawing.first();
    tracing::info!("{:?} [ANN:first_point]", first_point);
    let last_point = active_drawing.last();
    tracing::info!("{:?} [ANN:last_point]", last_point);
    let snap_threshold = 15.0/zoom;
     tracing::info!("{:?} [ANN:snap_threshold]",snap_threshold);
    let mut snap_distance = f64::MAX; // Start with huge numbe
     tracing::info!("{:?} [ANN:snap_distance]",snap_distance);
    if let Some(&(fx,fy)) = first_point {
        let dx = cursor_world_pos.0 - fx;
        let dy = cursor_world_pos.1 - fy;
        snap_distance = (dx*dx + dy*dy).sqrt();
    };

    let can_close = active_drawing.len() >=3 && snap_distance < snap_threshold;



    rsx! {
        svg {
            width: "{width}",
            height: "{height}",
            style: "position: absolute; top: 0; left: 0; pointer-events:none; overflow:visible;",
            // view_box: "0 0 {width} {height}",

            // Rendering Saved annotations
            for ann in annotations {
                {
                
                  let ann_id_ctx = ann.id.clone();
                  let label_id_ctx = ann.label_id.clone();
                    rsx!
                    {
                        polygon {
                        key: "{ann.id}",
                        points: "{format_geometry(&ann.geometry)}",
                        fill: "#00ff00",  // TODO: get from label
                        fill_opacity: "0.5",
                        stroke: "#00ff00",
                        stroke_width: "0.8",
                        vector_effect: "non-scaling-stroke",
                        cursor: "pointer",
                        // onclick: move |evt| {
                        //     evt.stop_propagation();
                        //     tracing::info!("Clicked annotation: {}", ann.id);
                        // },
                        oncontextmenu:move |evt:Event<MouseData>| {
                            tracing::info!("right click:-");
                            evt.prevent_default();
                            evt.stop_propagation();
                            let ann_id = ann.id.clone();
                            let label_id = ann.label_id.clone();
                            on_right_click.call((
                                ann_id_ctx.clone(),
                                label_id_ctx.clone(),
                                evt.client_coordinates().x,
                                evt.client_coordinates().y,
                                ));
                        }
                        }
                    }
                }
            }

            // Active drawing preview (only if we have points)
            if !active_drawing.is_empty() {
                // Closed polygon for fill preview (includes cursor so fill follows preview line)
                {
                    let mut fill_points = active_drawing.clone();
                    fill_points.push(cursor_world_pos);
                    rsx! {
                        polygon {
                            points: "{format_points(&fill_points)}",
                            fill: "rgba(255, 0, 0, 0.15)",
                            stroke: "none",
                        }
                    }
                }
                // Open polyline for stroke (doesn't auto-close)
                polyline {
                    points: "{format_points(&active_drawing)}",
                    fill: "none",
                    stroke: "red",
                    stroke_width: "{stroke_w}",
                }

                for (i, (x, y)) in active_drawing.iter().enumerate() {
                    circle {
                        key: "node-{i}",
                        cx: "{x}",
                        cy: "{y}",
                        r: "{circle_r}",
                        fill: "rgba(255, 0, 0, 0.5)",
                        stroke: "red",
                        stroke_width: "{stroke_w}",
                    }
                }

                // Preview line from last point to cursor
                if let Some(&(lx,ly)) = last_point{
                    line{
                        x1: "{lx}",
                        y1: "{ly}",
                        x2: "{cursor_world_pos.0}",
                        y2: "{cursor_world_pos.1}",
                        stroke:"red",
                        stroke_width:"{stroke_w}",
                    }
                }

                if can_close {
                    if let Some(&(fx,fy)) = first_point {
                        circle {
                            cx: "{fx}",
                            cy: "{fy}",
                            r: "{circle_r * 4.0}",
                            fill: "rgba(0,255,0,0.5)",
                            stroke: "green",
                            stroke_width: "{stroke_w}",
                        }
                    }
                }
            }
        }
    }
}
