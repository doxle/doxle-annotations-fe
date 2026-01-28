use dioxus::prelude::*;
use crate::shell::status::{STATUS, StatusType};

const STATUS_CSS: &str = include_str!("status_bar.css");

#[component]
pub fn StatusBar() -> Element {
    let status_opt = STATUS.read();
    
    if let Some(s) = status_opt.as_ref() {
        dioxus::logger::tracing::info!("🔔 StatusBar rendering: {}", s.message);
    }

    rsx! {
        style { {STATUS_CSS} }
        if let Some(status) = status_opt.as_ref() {
            div {
                class: match status.status_type {
                    StatusType::Success => "status-bar-pill status-success",
                    StatusType::Error => "status-bar-pill status-error",
                    StatusType::Info => "status-bar-pill status-info",
                },
                "{status.message}"
            }
        }
    }
}
