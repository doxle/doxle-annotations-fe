use dioxus::prelude::*;
use crate::api;

#[component]
pub fn ShareProjectModal(
    show: Signal<bool>,
    project_id: String,
) -> Element {
    let mut invite_link = use_signal(|| String::new());
    let mut invite_code = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| String::new());
    let mut copied = use_signal(|| false);
    let mut sending_email = use_signal(|| false);
    let mut email_sent = use_signal(|| false);
    let mut pending_invites = use_signal(|| Vec::<String>::new());
    
    // Load existing pending invites when modal opens
    use_effect(move || {
        if *show.read() {
            spawn(async move {
                // Fetch existing invites for this project
                // Note: This would need a backend endpoint like GET /projects/{project_id}/invites
                // For now, we'll just fetch all user invites
                match api::client::get::<Vec<serde_json::Value>>("/invites").await {
                    Ok(invites) => {
                        let pending: Vec<String> = invites
                            .iter()
                            .filter(|inv| {
                                inv.get("status")
                                    .and_then(|s| s.as_str())
                                    .map(|s| s == "pending")
                                    .unwrap_or(false)
                            })
                            .filter_map(|inv| inv.get("email").and_then(|e| e.as_str()).map(String::from))
                            .collect();
                        tracing::info!("Loaded {} pending invites", pending.len());
                        *pending_invites.write() = pending;
                    }
                    Err(e) => {
                        tracing::error!("Failed to load invites: {}", e);
                    }
                }
            });
        }
    });

    let send_invite_email = move |_| {
        let email_value = email.read().clone();
        let code = invite_code.read().clone();
        
        if email_value.is_empty() {
            *error.write() = "Please enter an email address".to_string();
            return;
        }
        
        spawn(async move {
            *sending_email.write() = true;
            *error.write() = String::new();
            *email_sent.write() = false;
            
            match api::client::post::<serde_json::Value, serde_json::Value>("/invites", &serde_json::json!({
                "email": email_value.clone(),
            })).await {
                Ok(_) => {
                    *email_sent.write() = true;
                    
                    // Add to pending invites list
                    pending_invites.write().push(email_value);
                    
                    *email.write() = String::new();
                    
                    // Reset success message after 3 seconds
                    spawn(async move {
                        gloo_timers::future::TimeoutFuture::new(3000).await;
                        *email_sent.write() = false;
                    });
                }
                Err(e) => {
                    *error.write() = format!("Failed to send email: {}", e);
                }
            }
            *sending_email.write() = false;
        });
    };

    let close_modal = move |_| {
        *show.write() = false;
        *email.write() = String::new();
        *error.write() = String::new();
        *copied.write() = false;
        *email_sent.write() = false;
        // Keep invite_link, invite_code, and pending_invites for session persistence
    };

    if !*show.read() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "modal-overlay",
            onclick: close_modal,

            div {
                class: "share-modal-content",
                onclick: move |e| e.stop_propagation(),

                // Header row
                div {
                    class: "share-modal-header",
                    
                    span {
                        class: "share-modal-title",
                        "Share this project"
                    }

                    div {
                        class: "share-modal-header-actions",
                        
                        button {
                            class: "share-link-button",
                            onclick: move |_| {
                                if invite_link.read().is_empty() {
                                    let pid = project_id.clone();
                                    spawn(async move {
                                        *loading.write() = true;
                                        *error.write() = String::new();
                                        *copied.write() = false;

                                        match api::client::post::<serde_json::Value, serde_json::Value>("/invites", &serde_json::json!({
                                            "email": "",
                                        })).await {
                                            Ok(response) => {
                                                if let Some(code) = response.get("invite_code").and_then(|c| c.as_str()) {
                                                    let link = format!("https://doxle.ai/signup?invite={}", code);
                                                    *invite_link.write() = link;
                                                    *invite_code.write() = code.to_string();
                                                } else {
                                                    *error.write() = "Failed to generate invite code".to_string();
                                                }
                                            }
                                            Err(e) => {
                                                *error.write() = format!("Error: {}", e);
                                            }
                                        }
                                        *loading.write() = false;
                                    });
                                } else {
                                    let link = invite_link.read().clone();
                                    if !link.is_empty() {
                                        #[cfg(target_arch = "wasm32")]
                                        {
                                            use wasm_bindgen::prelude::*;
                                            
                                            #[wasm_bindgen(inline_js = r#"
                                                export function copyToClipboard(text) {
                                                    if (navigator.clipboard && navigator.clipboard.writeText) {
                                                        navigator.clipboard.writeText(text).catch(err => console.error('Copy failed:', err));
                                                    }
                                                }
                                            "#)]
                                            extern "C" {
                                                fn copyToClipboard(text: &str);
                                            }
                                            
                                            copyToClipboard(&link);
                                        }
                                        *copied.write() = true;
                                        
                                        spawn(async move {
                                            gloo_timers::future::TimeoutFuture::new(2000).await;
                                            *copied.write() = false;
                                        });
                                    }
                                }
                            },
                            svg {
                                width: "14",
                                height: "14",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                path { d: "M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" }
                                path { d: "M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" }
                            }
                            span { 
                                if *copied.read() {
                                    "Copied!"
                                } else {
                                    "Copy Link"
                                }
                            }
                        }

                        button {
                            class: "share-close-button",
                            onclick: close_modal,
                            svg {
                                width: "16",
                                height: "16",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                line { x1: "18", y1: "6", x2: "6", y2: "18" }
                                line { x1: "6", y1: "6", x2: "18", y2: "18" }
                            }
                        }
                    }
                }

                // Divider
                div { class: "share-modal-divider" }

                // Email invite section
                div {
                    class: "share-email-section",

                    div {
                        class: "share-email-input-row",
                        
                        input {
                            class: "share-email-input",
                            r#type: "email",
                            placeholder: "Add email to invite",
                            value: "{email.read()}",
                            oninput: move |e| *email.write() = e.value(),
                        }

                        button {
                            class: if email.read().is_empty() {
                                "share-invite-button disabled"
                            } else {
                                "share-invite-button"
                            },
                            onclick: send_invite_email,
                            disabled: email.read().is_empty() || *sending_email.read(),
                            if *sending_email.read() {
                                "Sending..."
                            } else {
                                "Invite"
                            }
                        }
                    }

                    if *email_sent.read() {
                        div {
                            class: "share-email-success",
                            "✓ Invite sent!"
                        }
                    }
                }

                // Who has access section
                div {
                    class: "share-access-section",
                    
                    span {
                        class: "share-access-title",
                        "Who has access"
                    }

                    // Current user
                    div {
                        class: "share-user-row",
                        
                        div {
                            class: "share-user-avatar",
                            "U"
                        }

                        span {
                            class: "share-user-name",
                            "You"
                        }
                    }

                    // Pending invites
                    for invited_email in pending_invites.read().iter() {
                        div {
                            class: "share-user-row",
                            
                            div {
                                class: "share-user-avatar share-user-avatar-pending",
                                "{invited_email.chars().next().unwrap_or('?').to_uppercase()}"
                            }

                            div {
                                class: "share-user-info",
                                span {
                                    class: "share-user-name",
                                    "{invited_email}"
                                }
                                span {
                                    class: "share-user-status",
                                    "Pending"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
