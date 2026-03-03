use dioxus::prelude::*;
use super::models::{CommentThread, Comment};
use crate::users::state::USER;

const CSS: &str = include_str!("comment_dialog.css");

fn get_initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

#[component]
pub fn CommentDialog(
    screen_x: f64,
    screen_y: f64,
    thread: Option<CommentThread>,
    on_post: EventHandler<String>,
    on_resolve: EventHandler<()>,
    on_delete: EventHandler<()>,
    on_close: EventHandler<()>,
) -> Element {
    let mut reply_text = use_signal(|| String::new());
    let has_text = !reply_text().trim().is_empty();

    // Position dialog to the right of pin
    let viewport_w = web_sys::window()
        .and_then(|w| Some(w.inner_width().ok()?.as_f64()?))
        .unwrap_or(1200.0);
    let viewport_h = web_sys::window()
        .and_then(|w| Some(w.inner_height().ok()?.as_f64()?))
        .unwrap_or(800.0);

    let pin_x = screen_x;
    let pin_y = screen_y;
    let dlg_w = 260.0;
    let dialog_x = if pin_x + 28.0 + dlg_w > viewport_w - 10.0 {
        (pin_x - dlg_w - 4.0).max(10.0)
    } else {
        pin_x + 28.0
    };
    let dialog_y = if pin_y + 36.0 > viewport_h - 10.0 {
        (pin_y - 36.0).max(10.0)
    } else {
        (pin_y - 18.0).max(10.0)
    };

    let has_comments = thread.as_ref().map_or(false, |t| !t.comments.is_empty());
    let is_persisted = thread.as_ref().map_or(false, |t| t.persisted);
    let is_thread_resolved = thread.as_ref().map_or(false, |t| t.resolved);
    let placeholder = if has_comments { "Reply..." } else { "Add a comment..." };

    rsx! {
        style { {CSS} }

        // Backdrop
        div {
            class: "comment-dialog-backdrop",
            onclick: move |_| on_close.call(()),
        }

        // Dialog: just the input (+ thread if replies exist)
        div {
            class: "comment-dialog",
            style: "left: {dialog_x}px; top: {dialog_y}px;",
            onclick: move |evt| evt.stop_propagation(),

            // Header (only when thread has comments and persisted to server)
            if has_comments && is_persisted {
                div {
                    class: "comment-dialog-header",
                    span { class: "comment-dialog-title", "Comment" }
                    div {
                        class: "comment-dialog-actions",
                        // Delete
                        button {
                            class: "comment-action-btn",
                            title: "Delete thread",
                            onclick: move |_| on_delete.call(()),
                            svg {
                                width: "14", height: "14", view_box: "0 0 24 24",
                                fill: "none", stroke: "currentColor", stroke_width: "1.5",
                                stroke_linecap: "round", stroke_linejoin: "round",
                                path { d: "M3 6h18" }
                                path { d: "M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2" }
                                path { d: "M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6" }
                            }
                        }
                        // Resolve
                        button {
                            class: if is_thread_resolved { "comment-action-btn resolved" } else { "comment-action-btn" },
                            title: "Resolve",
                            onclick: move |_| on_resolve.call(()),
                            svg {
                                width: "14", height: "14", view_box: "0 0 24 24",
                                fill: "none", stroke: "currentColor", stroke_width: "1.5",
                                stroke_linecap: "round", stroke_linejoin: "round",
                                path { d: "M20 6L9 17l-5-5" }
                            }
                        }
                        // Close
                        button {
                            class: "comment-action-btn",
                            title: "Close",
                            onclick: move |_| on_close.call(()),
                            svg {
                                width: "14", height: "14", view_box: "0 0 24 24",
                                fill: "none", stroke: "currentColor", stroke_width: "1.5",
                                stroke_linecap: "round", stroke_linejoin: "round",
                                path { d: "M18 6L6 18" }
                                path { d: "M6 6l12 12" }
                            }
                        }
                    }
                }
            }

            // Thread (only if comments exist)
            if let Some(thread) = &thread {
                if !thread.comments.is_empty() {
                div {
                    class: "comment-dialog-thread",
                    for comment in thread.comments.iter() {
                        {
                            let initials = get_initials(&comment.user_name);
                            rsx! {
                                div {
                                    class: "comment-entry",
                                    key: "{comment.id}",
                                    div { class: "comment-avatar", "{initials}" }
                                    div {
                                        class: "comment-body",
                                        div {
                                            class: "comment-meta",
                                            span { class: "comment-author", "{comment.user_name}" }
                                            span { class: "comment-time", "{comment.created_at}" }
                                        }
                                        div { class: "comment-text", "{comment.text}" }
                                    }
                                }
                            }
                        }
                    }
                }
                }
            }

            // Input + circle button
            div {
                class: "comment-input-wrapper",
                input {
                    class: "comment-input",
                    placeholder: placeholder,
                    value: "{reply_text}",
                    onmounted: move |evt: MountedEvent| async move {
                        let _ = evt.set_focus(true).await;
                    },
                    oninput: move |e| reply_text.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter && !reply_text().trim().is_empty() {
                            on_post.call(reply_text().trim().to_string());
                            reply_text.set(String::new());
                        }
                    },
                }
                button {
                    class: if has_text { "comment-submit-btn active" } else { "comment-submit-btn" },
                    onclick: move |_| {
                        if !reply_text().trim().is_empty() {
                            on_post.call(reply_text().trim().to_string());
                            reply_text.set(String::new());
                        }
                    },
                    svg {
                        width: "12",
                        height: "12",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2.5",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        path { d: "M12 19V5" }
                        path { d: "M5 12L12 5L19 12" }
                    }
                }
            }
        }
    }
}
