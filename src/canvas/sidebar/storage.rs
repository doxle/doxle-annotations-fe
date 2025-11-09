use crate::state::PROJECTS;
use dioxus::prelude::ReadableExt;
use serde::{de::DeserializeOwned, Serialize};
use web_sys::window;

use super::types::ClassItem;

pub fn load_json<T: DeserializeOwned>(key: &str) -> Option<T> {
    let storage = window()?.local_storage().ok()??;
    let s = storage.get_item(key).ok()??;
    serde_json::from_str(&s).ok()
}

pub fn save_json<T: Serialize>(key: &str, value: &T) {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(s) = serde_json::to_string(value) {
            let _ = storage.set_item(key, &s);
        }
    }
}

pub fn load_f32(key: &str) -> Option<f32> {
    let storage = window()?.local_storage().ok()??;
    let s = storage.get_item(key).ok()??;
    s.parse::<f32>().ok()
}

pub fn save_f32(key: &str, v: f32) {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(key, &format!("{}", v));
    }
}

fn classes_key(project_id: &str) -> String {
    format!("classes:{}", project_id)
}
fn active_key(project_id: &str) -> String {
    format!("active_class:{}", project_id)
}

pub fn get_active_or_first_class_id(project_id: &str) -> Option<String> {
    let pid = project_id.to_string();
    let projects = PROJECTS.read();
    let proj = projects.iter().find(|p| p.project_id == pid)?;
    proj.labels.first().map(|l| l.name.clone()) // Use label name as the class id
}

pub fn increment_class_count(project_id: &str, class_id: &str, delta: i32) {}
