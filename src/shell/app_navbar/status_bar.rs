use dioxus::prelude::*;
use crate::shell::status::{STATUS, StatusType};

const STATUS_CSS: &str = include_str!("status_bar.css");

#[component]
pub fn StatusBar() -> Element {
    let status_opt = STATUS();
    dioxus::logger::tracing::info!("🔔 StatusBar state: {:?}", status_opt);

    rsx! {
        style { {STATUS_CSS} }
        if let Some(ref status) = status_opt {
            div {
                class: match status.status_type {
                    StatusType::Success => "status-bar-pill status-success",
                    StatusType::Error => "status-bar-pill status-error",
                    StatusType::Info => "status-bar-pill status-info",
                },
                style: "position: fixed; top: 12px; left: 50%; transform: translateX(-50%); z-index: 999999;",
                "{status.message}"
            }
        }
    }
}
