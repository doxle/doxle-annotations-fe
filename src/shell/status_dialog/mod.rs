mod live_timer;

use dioxus::prelude::*;
use crate::shell::progress::{STATUS, StatusType};
use live_timer::LiveTimer;

const STATUS_DIALOG_CSS: &str = include_str!("status_dialog.css");

#[component]
pub fn StatusDialog() -> Element {
    let status_opt = STATUS();

    rsx! {
        style { {STATUS_DIALOG_CSS} }
        if let Some(ref status) = status_opt {
            if let Some(ref progress) = status.progress {
                // Progress mode: card with live timer + log lines + progress bar
                {
                    let chars = status.message.len();
                    let duration = (chars as f64 * 0.02).max(0.15);
                    let pct = if progress.total > 0 {
                        (progress.current as f64 / progress.total as f64 * 100.0) as u32
                    } else { 0 };
                    let fill_class = match status.status_type {
                        StatusType::Danger => "status-dialog-progress-fill status-dialog-progress-fill-danger",
                        _ => "status-dialog-progress-fill",
                    };
                    let log_lines = &status.log;
                    rsx! {
                        div {
                            class: "status-dialog",
                            LiveTimer {}
                            if !log_lines.is_empty() {
                                div { class: "status-dialog-log",
                                    for line in log_lines.iter() {
                                        div {
                                            key: "{line}",
                                            class: "status-dialog-log-line",
                                            "{line}"
                                        }
                                    }
                                }
                            }
                            div {
                                key: "{status.message}",
                                class: "status-dialog-message typewriter",
                                style: "--chars: {chars}; --duration: {duration}s",
                                "{status.message}"
                            }
                            div { class: "status-dialog-progress-bar",
                                div {
                                    class: fill_class,
                                    style: "width: {pct}%",
                                }
                            }
                        }
                    }
                }
            } else {
            // Plain message mode — typewriter on inner span so container stays full-size
                {
                    let chars = status.message.len();
                    let duration = (chars as f64 * 0.02).max(0.15);
                    rsx! {
                        div {
                            class: match status.status_type {
                                StatusType::Success => "status-dialog-plain status-dialog-success",
                                StatusType::Error => "status-dialog-plain status-dialog-error",
                                StatusType::Info => "status-dialog-plain",
                                StatusType::Danger => "status-dialog-plain status-dialog-error",
                            },
                            span {
                                key: "{status.message}",
                                class: "typewriter",
                                style: "--chars: {chars}; --duration: {duration}s",
                                "{status.message}"
                            }
                        }
                    }
                }
            }
        }
    }
}
