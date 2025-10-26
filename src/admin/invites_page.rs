use crate::api;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct CreateInviteRequest {
    email: String,
    expires_days: i64,
}

#[derive(Debug, Deserialize, Clone)]
struct InviteResponse {
    invite_code: String,
    email: String,
    expires_at: String,
    status: String,
}

#[component]
pub fn AdminInvitesPage() -> Element {
    let mut email = use_signal(|| String::new());
    let mut expires_days = use_signal(|| String::from("7"));
    let mut created_invite = use_signal(|| Option::<InviteResponse>::None);
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        
        let email_value = email();
        let expires_days_value = expires_days().parse::<i64>().unwrap_or(7);
        
        spawn(async move {
            is_loading.set(true);
            error_message.set(None);
            created_invite.set(None);
            
            let request = CreateInviteRequest {
                email: email_value.clone(),
                expires_days: expires_days_value,
            };
            
            let client = reqwest::Client::new();
            let response = client
                .post("http://localhost:9000/invites")
                .header("Content-Type", "application/json")
                .header("X-User-Id", "admin") // In production, use real auth token
                .json(&request)
                .send()
                .await;
            
            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.json::<InviteResponse>().await {
                            Ok(invite) => {
                                created_invite.set(Some(invite));
                                email.set(String::new());
                                expires_days.set(String::from("7"));
                            }
                            Err(e) => {
                                error_message.set(Some(format!("Failed to parse response: {}", e)));
                            }
                        }
                    } else {
                        let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        error_message.set(Some(error_text));
                    }
                }
                Err(e) => {
                    error_message.set(Some(format!("Network error: {}", e)));
                }
            }
            
            is_loading.set(false);
        });
    };

    let copy_invite_link = move |invite_code: String| {
        let link = format!("http://localhost:8080/signup?code={}", invite_code);
        
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
    };

    rsx! {
        div {
            class: "admin-container",
            
            div {
                class: "admin-content",
                
                h1 {
                    class: "admin-title",
                    "Generate Invite Codes"
                }
                
                p {
                    class: "admin-description",
                    "Create invitation codes to allow new users to sign up for Doxle."
                }
                
                form {
                    onsubmit: handle_submit,
                    class: "admin-form",
                    
                    div {
                        class: "admin-field",
                        label {
                            class: "admin-label",
                            "Email Address"
                        }
                        input {
                            class: "admin-input",
                            r#type: "email",
                            placeholder: "user@example.com",
                            required: true,
                            value: "{email}",
                            oninput: move |e| email.set(e.value()),
                            disabled: is_loading(),
                        }
                    }
                    
                    div {
                        class: "admin-field",
                        label {
                            class: "admin-label",
                            "Expires In (days)"
                        }
                        input {
                            class: "admin-input",
                            r#type: "number",
                            placeholder: "7",
                            min: "1",
                            max: "365",
                            required: true,
                            value: "{expires_days}",
                            oninput: move |e| expires_days.set(e.value()),
                            disabled: is_loading(),
                        }
                    }
                    
                    button {
                        class: "admin-submit-button",
                        r#type: "submit",
                        disabled: is_loading(),
                        if is_loading() {
                            "Generating..."
                        } else {
                            "Generate Invite Code"
                        }
                    }
                }
                
                if let Some(error) = error_message() {
                    div {
                        class: "admin-error",
                        "{error}"
                    }
                }
                
                if let Some(invite) = created_invite() {
                    div {
                        class: "admin-success-card",
                        
                        h3 {
                            class: "admin-success-title",
                            "✓ Invite Created Successfully"
                        }
                        
                        div {
                            class: "admin-invite-details",
                            
                            div {
                                class: "admin-detail-row",
                                span {
                                    class: "admin-detail-label",
                                    "Email:"
                                }
                                span {
                                    class: "admin-detail-value",
                                    "{invite.email}"
                                }
                            }
                            
                            div {
                                class: "admin-detail-row",
                                span {
                                    class: "admin-detail-label",
                                    "Invite Code:"
                                }
                                span {
                                    class: "admin-detail-value admin-code",
                                    "{invite.invite_code}"
                                }
                            }
                            
                            div {
                                class: "admin-detail-row",
                                span {
                                    class: "admin-detail-label",
                                    "Expires:"
                                }
                                span {
                                    class: "admin-detail-value",
                                    "{invite.expires_at}"
                                }
                            }
                        }
                        
                        div {
                            class: "admin-invite-link-section",
                            label {
                                class: "admin-detail-label",
                                "Signup Link:"
                            }
                            div {
                                class: "admin-link-container",
                                input {
                                    class: "admin-link-input",
                                    r#type: "text",
                                    readonly: true,
                                    value: "http://localhost:8080/signup?code={invite.invite_code}"
                                }
                                button {
                                    class: "admin-copy-button",
                                    onclick: move |_| copy_invite_link(invite.invite_code.clone()),
                                    "Copy Link"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
