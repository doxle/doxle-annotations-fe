use crate::public::estimate_types::{PageLayout, ScaleData};
use dioxus::prelude::*;
use std::collections::HashMap;

pub type Segment = ((f64, f64), (f64, f64));

#[derive(Debug, Clone, PartialEq)]
pub struct CabinetryData {
    pub page_id: String,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
pub struct CabinetryClickResult {
    pub next_start_point: Option<(f64, f64)>,
    pub next_page_id: Option<String>,
    pub new_segment: Option<(String, Segment)>,
}

fn segment_length_px(segment: Segment) -> f64 {
    let ((x1, y1), (x2, y2)) = segment;
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
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

pub fn total_cabinetry_lm_m(
    cabinetry: &HashMap<String, CabinetryData>,
    scales: &HashMap<String, ScaleData>,
) -> f64 {
    cabinetry
        .iter()
        .map(|(page_id, data)| {
            let total_px = data
                .segments
                .iter()
                .map(|segment| segment_length_px(*segment))
                .fold(0.0_f64, |sum, value| sum + value);
            length_px_to_meters(total_px, scales.get(page_id))
        })
        .fold(0.0_f64, |sum, value| sum + value)
}

pub fn compute_live_cabinetry_lm_m(
    cabinetry: &HashMap<String, CabinetryData>,
    start_point: Option<(f64, f64)>,
    cursor_world: Option<(f64, f64)>,
    page_id: Option<&str>,
    scales: &HashMap<String, ScaleData>,
) -> f64 {
    let base = total_cabinetry_lm_m(cabinetry, scales);
    let (Some(start), Some(cursor), Some(pid)) = (start_point, cursor_world, page_id) else {
        return base;
    };
    let preview_px = segment_length_px((start, cursor));
    base + length_px_to_meters(preview_px, scales.get(pid))
}

pub fn handle_cabinetry_click(
    pages: &[PageLayout],
    current_start_point: Option<(f64, f64)>,
    current_page_id: Option<&str>,
    click_world: (f64, f64),
) -> CabinetryClickResult {
    let (wx, wy) = click_world;
    let clicked_page = pages.iter().find(|page| {
        wy >= page.y_offset
            && wy < page.y_offset + page.display_height as f64
    });

    let Some(page) = clicked_page else {
        return CabinetryClickResult {
            next_start_point: current_start_point,
            next_page_id: current_page_id.map(ToOwned::to_owned),
            new_segment: None,
        };
    };

    let clicked_page_id = page.page_num.to_string();
    let click_point = (wx, wy);

    match current_start_point {
        None => CabinetryClickResult {
            next_start_point: Some(click_point),
            next_page_id: Some(clicked_page_id),
            new_segment: None,
        },
        Some(start) => {
            if current_page_id != Some(clicked_page_id.as_str()) {
                return CabinetryClickResult {
                    next_start_point: Some(click_point),
                    next_page_id: Some(clicked_page_id),
                    new_segment: None,
                };
            }

            let distance = segment_length_px((start, click_point));
            if distance <= 0.0 {
                return CabinetryClickResult {
                    next_start_point: None,
                    next_page_id: Some(clicked_page_id),
                    new_segment: None,
                };
            }

            CabinetryClickResult {
                next_start_point: None,
                next_page_id: Some(clicked_page_id.clone()),
                new_segment: Some((clicked_page_id, (start, click_point))),
            }
        }
    }
}

#[component]
pub fn CabinetryPanelSection(
    cabinetry: HashMap<String, CabinetryData>,
    page_scales: HashMap<String, ScaleData>,
    live_lm_m: Option<f64>,
    has_active_segment: bool,
) -> Element {
    let saved_lm_m = total_cabinetry_lm_m(&cabinetry, &page_scales);
    let display_lm_m = if has_active_segment {
        live_lm_m.unwrap_or(saved_lm_m)
    } else if saved_lm_m > 0.0 {
        saved_lm_m
    } else {
        0.0
    };

    rsx! {
        p {
            class: "estimate-panel-description",
            "Draw cabinetry extents as separate broken line segments."
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
