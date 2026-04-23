use crate::core::client::API_BASE_URL;
use crate::public::estimate_types::{SaveScaleRequest, ScaleData, ScalePoint};
use dioxus::prelude::*;
use gloo_net::http::Request;
use std::collections::HashMap;

pub fn handle_scale_canvas_click(
    clicked_page_id: String,
    click_world: (f64, f64),
    mut scale_points: Signal<Vec<(f64, f64)>>,
    mut active_scale_page_id: Signal<Option<String>>,
    mut scale_distance_input: Signal<String>,
) {
    let (wx, wy) = click_world;
    let current_page_id = active_scale_page_id();
    let mut next_points = scale_points();

    if next_points.len() >= 2 {
        next_points = vec![(wx, wy)];
        active_scale_page_id.set(Some(clicked_page_id));
        scale_distance_input.set(String::new());
    } else if next_points.is_empty() {
        next_points.push((wx, wy));
        active_scale_page_id.set(Some(clicked_page_id));
        scale_distance_input.set(String::new());
    } else if current_page_id.as_deref() != Some(clicked_page_id.as_str()) {
        next_points = vec![(wx, wy)];
        active_scale_page_id.set(Some(clicked_page_id));
        scale_distance_input.set(String::new());
    } else {
        next_points.push((wx, wy));
    }

    scale_points.set(next_points);
}

pub fn render_scale_badge(scale: ScaleData, page_y_offset: f64) -> Element {
    let badge_x = 10.0;
    let badge_y = page_y_offset + 10.0;
    let label = format!("{:.2}m", scale.real_distance_m);

    rsx! {
        rect {
            x: "{badge_x}",
            y: "{badge_y}",
            width: "120",
            height: "30",
            rx: "15",
            ry: "15",
            fill: "#EF4444",
            pointer_events: "none",
        }
        text {
            x: "{badge_x + 60.0}",
            y: "{badge_y + 15.0}",
            fill: "#ffffff",
            font_size: "14",
            font_family: "ModularHouseplant, Helvetica, sans-serif",
            text_anchor: "middle",
            dominant_baseline: "middle",
            pointer_events: "none",
            "{label}"
        }
    }
}

fn dimension_offset_points(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
) -> (f64, f64, f64, f64, f64, f64) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let is_horizontal = dx.abs() >= dy.abs();
    let offset = 28.0;
    let (ox, oy) = if is_horizontal {
        (0.0, -offset)
    } else {
        (-offset, 0.0)
    };
    let sx1 = x1 + ox;
    let sy1 = y1 + oy;
    let sx2 = x2 + ox;
    let sy2 = y2 + oy;
    let mid_x = (sx1 + sx2) / 2.0;
    let mid_y = (sy1 + sy2) / 2.0;
    (sx1, sy1, sx2, sy2, mid_x, mid_y)
}

fn render_tick(x: f64, y: f64) -> Element {
    rsx! {
        line {
            x1: "{x}",
            y1: "{y - 8.0}",
            x2: "{x}",
            y2: "{y + 8.0}",
            stroke: "#EF4444",
            stroke_width: "2",
            pointer_events: "none",
        }
    }
}

pub fn render_saved_scale_line(scale: ScaleData, scale_text_stroke: String) -> Element {
    let (x1, y1) = scale.point1;
    let (x2, y2) = scale.point2;
    let (sx1, sy1, sx2, sy2, mid_x, mid_y) = dimension_offset_points(x1, y1, x2, y2);

    rsx! {
        { render_tick(x1, y1) }
        { render_tick(x2, y2) }
        line {
            x1: "{x1}",
            y1: "{y1}",
            x2: "{sx1}",
            y2: "{sy1}",
            stroke: "#EF4444",
            stroke_width: "1.5",
            stroke_dasharray: "5 5",
            pointer_events: "none",
        }
        line {
            x1: "{x2}",
            y1: "{y2}",
            x2: "{sx2}",
            y2: "{sy2}",
            stroke: "#EF4444",
            stroke_width: "1.5",
            stroke_dasharray: "5 5",
            pointer_events: "none",
        }
        line {
            x1: "{sx1}",
            y1: "{sy1}",
            x2: "{sx2}",
            y2: "{sy2}",
            stroke: "#EF4444",
            stroke_width: "2",
            stroke_dasharray: "7 5",
            pointer_events: "none",
        }
        text {
            x: "{mid_x}",
            y: "{mid_y - 8.0}",
            fill: "#EF4444",
            font_size: "22",
            text_anchor: "middle",
            dominant_baseline: "middle",
            stroke: "{scale_text_stroke}",
            stroke_width: "0.8",
            paint_order: "stroke fill",
            pointer_events: "none",
            "{scale.real_distance_m:.2} m"
        }
    }
}

pub fn render_active_scale_drawing(points: Vec<(f64, f64)>) -> Element {
    let line_data = if points.len() == 2 {
        Some((points[0], points[1]))
    } else {
        None
    };

    rsx! {
        for (x, y) in points.iter() {
            { render_tick(*x, *y) }
        }

        if let Some(((x1, y1), (x2, y2))) = line_data {
            {
                let (sx1, sy1, sx2, sy2, _mid_x, _mid_y) = dimension_offset_points(x1, y1, x2, y2);
                rsx! {
                    line {
                        x1: "{x1}",
                        y1: "{y1}",
                        x2: "{sx1}",
                        y2: "{sy1}",
                        stroke: "#EF4444",
                        stroke_width: "1.5",
                        stroke_dasharray: "5 5",
                        pointer_events: "none",
                    }
                    line {
                        x1: "{x2}",
                        y1: "{y2}",
                        x2: "{sx2}",
                        y2: "{sy2}",
                        stroke: "#EF4444",
                        stroke_width: "1.5",
                        stroke_dasharray: "5 5",
                        pointer_events: "none",
                    }
                    line {
                        x1: "{sx1}",
                        y1: "{sy1}",
                        x2: "{sx2}",
                        y2: "{sy2}",
                        stroke: "#EF4444",
                        stroke_width: "2",
                        stroke_dasharray: "7 5",
                        pointer_events: "none",
                    }
                }
            }
        }
    }
}

#[component]
pub fn ScalePanelSection(
    project_id: String,
    page_scales: Signal<HashMap<String, ScaleData>>,
    scale_points: Signal<Vec<(f64, f64)>>,
    active_scale_page_id: Signal<Option<String>>,
    scale_distance_input: Signal<String>,
) -> Element {
    let mut page_scales = page_scales;
    let mut scale_points = scale_points;
    let mut active_scale_page_id = active_scale_page_id;
    let mut scale_distance_input = scale_distance_input;
    let project_id_for_save = project_id.clone();

    rsx! {
        if !page_scales().is_empty() {
            {
                let mut sorted_scales: Vec<_> = page_scales().iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                sorted_scales.sort_by_key(|(pid, _)| pid.parse::<usize>().unwrap_or(999));
                rsx! {
                    div {
                        class: "estimate-panel-scale-list",
                        for (pid, scale) in sorted_scales.iter() {
                            {
                                let pid_for_delete = pid.clone();
                                let project_id_for_delete = project_id.clone();
                                rsx! {
                                    div {
                                        class: "estimate-panel-scale-row",
                                        span {
                                            class: "estimate-panel-scale-row-page",
                                            "Page {pid}"
                                        }
                                        span {
                                            class: "estimate-panel-scale-row-value",
                                            "{scale.real_distance_m:.2}m"
                                        }
                                        button {
                                            class: "estimate-panel-scale-row-close",
                                            onclick: move |_| {
                                                let pid = pid_for_delete.clone();
                                                page_scales.write().remove(&pid);

                                                let project_id = project_id_for_delete.clone();
                                                let pid_del = pid.clone();
                                                spawn(async move {
                                                    let del_url = format!(
                                                        "{}/public/projects/{}/scale/{}",
                                                        API_BASE_URL, project_id, pid_del
                                                    );
                                                    let _ = Request::delete(&del_url).send().await;
                                                });
                                            },
                                            "\u{2715}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if scale_points().len() == 2 {
            form {
                class: "estimate-panel-scale-form",
                onsubmit: move |evt| {
                    evt.prevent_default();

                    let Ok(distance_mm) = scale_distance_input().trim().parse::<f64>() else {
                        return;
                    };
                    if distance_mm <= 0.0 {
                        return;
                    }
                    let distance_m = distance_mm / 1000.0;
                    if distance_m <= 0.0 {
                        return;
                    }

                    let pts = scale_points();
                    if pts.len() != 2 {
                        return;
                    }

                    let Some(page_id) = active_scale_page_id() else {
                        return;
                    };

                    let (x1, y1) = pts[0];
                    let (x2, y2) = pts[1];
                    let px_distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
                    if px_distance <= 0.0 {
                        return;
                    }

                    let submit = ScaleData {
                        page_id: page_id.clone(),
                        point1: (x1, y1),
                        point2: (x2, y2),
                        real_distance_m: distance_m,
                        px_per_meter: px_distance / distance_m,
                    };
                    page_scales.write().insert(page_id.clone(), submit);

                    let project_id_for_save = project_id_for_save.clone();
                    spawn(async move {
                        let payload = SaveScaleRequest {
                            reference_start: ScalePoint { x: x1, y: y1 },
                            reference_end: ScalePoint { x: x2, y: y2 },
                            real_distance_m: distance_m,
                            page_id,
                        };
                        let save_url = format!("{}/public/projects/{}/scale", API_BASE_URL, project_id_for_save);
                        if let Ok(request) = Request::post(&save_url)
                            .header("Content-Type", "application/json")
                            .json(&payload)
                        {
                            let _ = request.send().await;
                        }
                    });

                    scale_points.set(Vec::new());
                    active_scale_page_id.set(None);
                    scale_distance_input.set(String::new());
                },

                p {
                    class: "estimate-panel-description",
                    "Enter the real-world distance between the two points in millimetres."
                }
                input {
                    class: "estimate-panel-scale-input",
                    r#type: "number",
                    step: "1",
                    min: "0",
                    placeholder: "distance (mm)",
                    value: "{scale_distance_input}",
                    oninput: move |evt| scale_distance_input.set(evt.value()),
                    autofocus: true,
                }
                button {
                    class: "estimate-panel-btn",
                    r#type: "submit",
                    "Set Scale"
                }
            }
        } else {
            p {
                class: "estimate-panel-description",
                "Click two points on a page to define a known measurement, then enter the distance in millimetres."
            }
        }
    }
}
