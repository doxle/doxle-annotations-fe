use crate::public::estimate_types::{PageLayout, ScaleData};
use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct EWallsData {
    pub page_id: String,
    pub points: Vec<(f64, f64)>,
    pub closed: bool,
}

#[derive(Debug, Clone)]
pub struct EWallsClickResult {
    pub next_points: Vec<(f64, f64)>,
    pub next_page_id: Option<String>,
    pub closed_path: Option<(String, Vec<(f64, f64)>)>,
}

pub fn polyline_length_px(points: &[(f64, f64)], closed: bool) -> f64 {
    if points.len() < 2 {
        return 0.0;
    }

    let mut total = 0.0_f64;
    for i in 1..points.len() {
        let (x1, y1) = points[i - 1];
        let (x2, y2) = points[i];
        total += ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    }

    if closed && points.len() >= 3 {
        let (sx, sy) = points[0];
        let (lx, ly) = points[points.len() - 1];
        total += ((sx - lx).powi(2) + (sy - ly).powi(2)).sqrt();
    }

    total
}

pub fn length_px_to_meters(length_px: f64, scale: Option<&ScaleData>) -> f64 {
    let Some(scale) = scale else {
        return 0.0;
    };
    if scale.px_per_meter <= 0.0 {
        return 0.0;
    }
    length_px / scale.px_per_meter
}

pub fn compute_live_ewalls_lm_m(
    active_points: &[(f64, f64)],
    cursor_world: Option<(f64, f64)>,
    page_id: Option<&str>,
    scales: &HashMap<String, ScaleData>,
) -> f64 {
    let Some(pid) = page_id else {
        return 0.0;
    };
    let mut points = active_points.to_vec();
    if let Some(cursor) = cursor_world {
        points.push(cursor);
    }
    let length_px = polyline_length_px(&points, false);
    length_px_to_meters(length_px, scales.get(pid))
}

pub fn compute_saved_ewalls_lm_m(
    ewalls: &HashMap<String, EWallsData>,
    scales: &HashMap<String, ScaleData>,
) -> f64 {
    ewalls
        .iter()
        .map(|(page_id, path)| {
            let length_px = polyline_length_px(&path.points, path.closed);
            length_px_to_meters(length_px, scales.get(page_id))
        })
        .fold(0.0_f64, |sum, value| sum + value)
}

pub fn handle_ewalls_click(
    pages: &[PageLayout],
    current_points: &[(f64, f64)],
    current_page_id: Option<&str>,
    click_world: (f64, f64),
    zoom: f64,
) -> EWallsClickResult {
    let (wx, wy) = click_world;
    let clicked_page = pages.iter().find(|page| {
        wy >= page.y_offset
            && wy < page.y_offset + page.display_height as f64
    });

    let Some(page) = clicked_page else {
        return EWallsClickResult {
            next_points: current_points.to_vec(),
            next_page_id: current_page_id.map(ToOwned::to_owned),
            closed_path: None,
        };
    };

    let clicked_page_id = page.page_num.to_string();
    let mut next_points = current_points.to_vec();
    let next_page_id = Some(clicked_page_id.clone());

    if next_points.is_empty() {
        next_points.push((wx, wy));
        return EWallsClickResult {
            next_points,
            next_page_id,
            closed_path: None,
        };
    }

    if current_page_id != Some(clicked_page_id.as_str()) {
        return EWallsClickResult {
            next_points: vec![(wx, wy)],
            next_page_id,
            closed_path: None,
        };
    }

    if next_points.len() >= 3 {
        if let Some(&(fx, fy)) = next_points.first() {
            let snap_threshold = 15.0 / zoom.max(0.0001);
            let dist = ((wx - fx).powi(2) + (wy - fy).powi(2)).sqrt();
            if dist < snap_threshold {
                let closed_points = next_points.clone();
                return EWallsClickResult {
                    next_points: Vec::new(),
                    next_page_id: None,
                    closed_path: Some((clicked_page_id, closed_points)),
                };
            }
        }
    }

    next_points.push((wx, wy));
    EWallsClickResult {
        next_points,
        next_page_id,
        closed_path: None,
    }
}

#[component]
pub fn EWallsPanelSection(
    ewalls: HashMap<String, EWallsData>,
    page_scales: HashMap<String, ScaleData>,
    live_lm_m: Option<f64>,
    active_drawing_points: usize,
) -> Element {
    let saved_lm_m = compute_saved_ewalls_lm_m(&ewalls, &page_scales);
    let display_lm_m = if active_drawing_points > 0 {
        live_lm_m.unwrap_or(0.0)
    } else if saved_lm_m > 0.0 {
        saved_lm_m
    } else {
        0.0
    };

    rsx! {
        p {
            class: "estimate-panel-description",
            "Trace the external wall perimeter as a continuous line."
        }
        div {
            class: "estimate-panel-scale-list estimate-footprint-area-list",
            div {
                class: "estimate-panel-scale-row estimate-footprint-area-row",
                span {
                    class: "estimate-footprint-area-value",
                    "{display_lm_m:.5} m"
                }
            }
        }
    }
}
