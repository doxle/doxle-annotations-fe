use crate::core::client::CLOUDFRONT_URL;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const CLASSIFICATION_ORDER: &[&str] =
    &["floor_plan", "elevation", "electrical", "site_plan", "other"];
pub const PAGE_GAP: f64 = 150.0;
pub const ESTIMATE_CANVAS_ID: &str = "estimate-single-canvas";

#[derive(Debug, Clone, Deserialize)]
pub struct ResultJson {
    pub pages: Vec<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PageInfo {
    pub page_num: usize,
    #[serde(default)]
    pub classification: String,
    pub display_key: String,
    pub display_width: u32,
    pub display_height: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageLayout {
    pub page_num: usize,
    pub classification: String,
    pub image_url: String,
    pub display_width: u32,
    pub display_height: u32,
    pub y_offset: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScalePoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SavedScaleResponse {
    pub reference_start: ScalePoint,
    pub reference_end: ScalePoint,
    pub real_distance_m: f64,
    pub px_per_meter: f64,
    pub page_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveScaleRequest {
    pub reference_start: ScalePoint,
    pub reference_end: ScalePoint,
    pub real_distance_m: f64,
    pub page_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScaleData {
    pub page_id: String,
    pub point1: (f64, f64),
    pub point2: (f64, f64),
    pub real_distance_m: f64,
    pub px_per_meter: f64,
}

#[derive(Clone, PartialEq)]
pub struct ClassifiedSection {
    pub classification: String,
    pub label: String,
    pub pages: Vec<PageInfo>,
}

pub fn classification_rank(classification: &str) -> usize {
    CLASSIFICATION_ORDER
        .iter()
        .position(|value| *value == classification)
        .unwrap_or(CLASSIFICATION_ORDER.len())
}

pub fn build_page_layouts(pages: Vec<PageInfo>) -> Vec<PageLayout> {

    let mut y_offset = 0.0_f64;
    let mut layouts = Vec::with_capacity(pages.len());
    for page in pages {
        layouts.push(PageLayout {
            page_num: page.page_num,
            classification: page.classification.clone(),
            image_url: format!("{}/cdn/app/{}", CLOUDFRONT_URL, page.display_key),
            display_width: page.display_width,
            display_height: page.display_height,
            y_offset,
        });
        y_offset += page.display_height as f64 + PAGE_GAP;
    }
    layouts
}

pub fn canvas_content_bounds(pages: &[PageLayout]) -> (f64, f64) {
    let content_width = pages
        .iter()
        .map(|page| page.display_width as f64)
        .fold(0.0_f64, f64::max);
    let content_height = pages
        .last()
        .map(|page| page.y_offset + page.display_height as f64)
        .unwrap_or(0.0);
    (content_width, content_height)
}

pub fn detect_page_id_from_world_y(pages: &[PageLayout], click_y: f64) -> Option<String> {
    pages
        .iter()
        .find(|page| {
            click_y >= page.y_offset && click_y < page.y_offset + page.display_height as f64
        })
        .map(|page| page.page_num.to_string())
}

pub fn classification_label(classification: &str) -> &str {
    match classification {
        "floor_plan" => "Floor Plan",
        "elevation" => "Elevations",
        "electrical" => "Electrical",
        "site_plan" => "Site Plan",
        "other" => "Other",
        _ => "Other",
    }
}
