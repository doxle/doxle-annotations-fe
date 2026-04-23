use crate::core::client::API_BASE_URL;
use crate::public::estimate_types::{PageLayout, ScaleData};
use dioxus::prelude::*;
use gloo_net::http::Request;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct FootprintData {
    pub page_id: String,
    pub points: Vec<(f64, f64)>,
}

#[derive(Debug, Clone)]
pub struct FootprintClickResult {
    pub next_points: Vec<(f64, f64)>,
    pub next_page_id: Option<String>,
    pub closed_polygon: Option<(String, Vec<(f64, f64)>)>,
}

#[derive(Debug, Clone, Serialize)]
struct GeometryPointPayload {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, Serialize)]
struct GeometryPayload {
    #[serde(rename = "type")]
    geometry_type: String,
    points: Vec<GeometryPointPayload>,
}

#[derive(Debug, Clone, Serialize)]
struct SaveFootprintRequest {
    email: String,
    geometry_type: String,
    page_id: Option<String>,
    geometry: GeometryPayload,
}

pub fn polygon_points_to_svg(points: &[(f64, f64)]) -> String {
    points
        .iter()
        .map(|(x, y)| format!("{x},{y}"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn polygon_area_px(points: &[(f64, f64)]) -> f64 {
    if points.len() < 3 {
        return 0.0;
    }

    let mut area = 0.0;
    for i in 0..points.len() {
        let j = (i + 1) % points.len();
        area += points[i].0 * points[j].1;
        area -= points[j].0 * points[i].1;
    }
    (area / 2.0).abs()
}

pub fn scale_px_per_meter_for_area(scale: &ScaleData) -> f64 {
    if scale.real_distance_m > 100.0 {
        scale.px_per_meter * 1000.0
    } else {
        scale.px_per_meter
    }
}

pub fn compute_preview_area_m2(
    active_points: &[(f64, f64)],
    cursor_world: Option<(f64, f64)>,
    page_id: Option<&str>,
    scales: &HashMap<String, ScaleData>,
) -> f64 {
    let selected_scale = page_id
        .and_then(|pid| scales.get(pid).cloned())
        .or_else(|| scales.values().next().cloned());

    let Some(scale) = selected_scale else {
        return 0.0;
    };

    let px_per_meter = scale_px_per_meter_for_area(&scale);
    if px_per_meter <= 0.0 {
        return 0.0;
    }

    let mut preview_points = active_points.to_vec();
    if let Some(cursor) = cursor_world {
        preview_points.push(cursor);
    }

    if preview_points.len() < 3 {
        return 0.0;
    }

    let area_px = polygon_area_px(&preview_points);
    area_px / (px_per_meter * px_per_meter)
}

pub fn first_floor_plan_page(pages: &[PageLayout]) -> Option<PageLayout> {
    pages.first().cloned()
}

pub fn handle_footprint_click(
    pages: &[PageLayout],
    current_points: &[(f64, f64)],
    current_page_id: Option<&str>,
    click_world: (f64, f64),
    zoom: f64,
) -> FootprintClickResult {
    let (wx, wy) = click_world;
    let clicked_page = pages.iter().find(|page| {
        wy >= page.y_offset
            && wy < page.y_offset + page.display_height as f64
    });

    let Some(page) = clicked_page else {
        return FootprintClickResult {
            next_points: current_points.to_vec(),
            next_page_id: current_page_id.map(ToOwned::to_owned),
            closed_polygon: None,
        };
    };

    let clicked_page_id = page.page_num.to_string();
    let mut next_points = current_points.to_vec();
    let next_page_id = Some(clicked_page_id.clone());

    if next_points.is_empty() {
        next_points.push((wx, wy));
        return FootprintClickResult {
            next_points,
            next_page_id,
            closed_polygon: None,
        };
    }

    if current_page_id != Some(clicked_page_id.as_str()) {
        return FootprintClickResult {
            next_points: vec![(wx, wy)],
            next_page_id,
            closed_polygon: None,
        };
    }

    if next_points.len() >= 3 {
        if let Some(&(fx, fy)) = next_points.first() {
            let snap_threshold = 15.0 / zoom.max(0.0001);
            let dist = ((wx - fx).powi(2) + (wy - fy).powi(2)).sqrt();
            if dist < snap_threshold {
                let closed_points = next_points.clone();
                return FootprintClickResult {
                    next_points: Vec::new(),
                    next_page_id: None,
                    closed_polygon: Some((clicked_page_id, closed_points)),
                };
            }
        }
    }

    next_points.push((wx, wy));
    FootprintClickResult {
        next_points,
        next_page_id,
        closed_polygon: None,
    }
}

pub async fn save_footprint_geometry(
    project_id: String,
    email: String,
    page_id: String,
    points: Vec<(f64, f64)>,
) {
    if email.trim().is_empty() || points.len() < 3 {
        return;
    }

    let payload = SaveFootprintRequest {
        email,
        geometry_type: "footprint".to_string(),
        page_id: Some(page_id),
        geometry: GeometryPayload {
            geometry_type: "polygon".to_string(),
            points: points
                .into_iter()
                .map(|(x, y)| GeometryPointPayload { x, y })
                .collect(),
        },
    };

    let save_url = format!("{}/public/projects/{}/geometry", API_BASE_URL, project_id);
    if let Ok(request) = Request::post(&save_url)
        .header("Content-Type", "application/json")
        .json(&payload)
    {
        let _ = request.send().await;
    }
}

#[component]
pub fn FootprintPanelSection(
    footprints: HashMap<String, FootprintData>,
    page_scales: HashMap<String, ScaleData>,
    live_area_m2: Option<f64>,
    active_drawing_points: usize,
) -> Element {
    let saved_total_area_m2 = footprints
        .iter()
        .filter_map(|(page_id, footprint)| {
            page_scales.get(page_id).and_then(|scale| {
                let px_per_meter = scale_px_per_meter_for_area(scale);
                if px_per_meter > 0.0 {
                    Some(polygon_area_px(&footprint.points) / (px_per_meter * px_per_meter))
                } else {
                    None
                }
            })
        })
        .fold(0.0_f64, |sum, area| sum + area);

    let live_value = live_area_m2.unwrap_or(0.0);
    let display_area_m2 = if active_drawing_points > 0 {
        live_value
    } else if saved_total_area_m2 > 0.0 {
        saved_total_area_m2
    } else {
        0.0
    };

    rsx! {
        p {
            class: "estimate-panel-description",
            "Trace the outline of the ground floor area."
        }

        div {
            class: "estimate-panel-scale-list estimate-footprint-area-list",
            div {
                class: "estimate-panel-scale-row estimate-footprint-area-row",
                span {
                    class: "estimate-footprint-area-value",
                    "{display_area_m2:.5} m2"
                }
            }
        }
    }
}
