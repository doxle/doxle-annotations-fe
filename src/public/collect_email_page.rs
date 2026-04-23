use crate::Route;
use dioxus::prelude::*;
use super::upload_api::{UploadedFileInfo, create_public_project};

const CSS: &str = include_str!("collect_email_page.css");

#[component]
pub fn CollectEmailPage() -> Element {
    let mut project_name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut submitting = use_signal(|| false);
    let nav = use_navigator();

    let has_email = !email().trim().is_empty() && email().contains('@');
    let has_name = project_name().trim().len() >= 3;
    let can_submit = has_name && has_email;

    rsx! {
        style { {CSS} }
        div {
            class: "collect-email-page",
            div {
                class: "collect-email-card",

                h1 {
                    class: "collect-email-title",
                    "Almost there"
                }
                p {
                    class: "collect-email-subtitle",
                    "Your personalised budget estimate will be delivered to your email. Please ensure your email is correct so we can get it to you."
                }

                form {
                    class: "collect-email-form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        if !can_submit || *submitting.read() {
                            return;
                        }
                        submitting.set(true);
                        let name_val = project_name().trim().to_string();
                        let email_val = email().trim().to_string();

                        spawn(async move {
                            // Load uploaded files from localStorage
                            let files: Vec<UploadedFileInfo> = {
                                #[cfg(target_arch = "wasm32")]
                                {
                                    web_sys::window()
                                        .and_then(|w| w.local_storage().ok().flatten())
                                        .and_then(|s| s.get_item("doxle_uploaded_files").ok().flatten())
                                        .and_then(|json| serde_json::from_str(&json).ok())
                                        .unwrap_or_default()
                                }
                                #[cfg(not(target_arch = "wasm32"))]
                                { Vec::new() }
                            };

                            match create_public_project(&name_val, &email_val, files).await {
                                Ok(bootstrap) => {
                                    // Store project context in localStorage
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        if let Some(window) = web_sys::window() {
                                            if let Ok(Some(storage)) = window.local_storage() {
                                                let _ = storage.set_item("doxle_public_email", &bootstrap.project_email);
                                                let _ = storage.set_item("doxle_public_block_id", &bootstrap.block_id);
                                                // Extract and save image_id before cleanup
                                                if let Ok(Some(files_json)) = storage.get_item("doxle_uploaded_files") {
                                                    if let Ok(files) = serde_json::from_str::<Vec<serde_json::Value>>(&files_json) {
                                                        let image_ids: Vec<String> = files.iter()
                                                            .filter_map(|f| f.get("image_id").and_then(|v| v.as_str()).map(|s| s.to_string()))
                                                            .collect();
                                                        if let Ok(ids_json) = serde_json::to_string(&image_ids) {
                                                            let _ = storage.set_item("doxle_image_ids", &ids_json);
                                                        }
                                                    }
                                                }
                                                let _ = storage.remove_item("doxle_uploaded_files");
                                            }
                                        }
                                    }
                                    crate::core::progress::show_success("Project created");
                                    nav.push(Route::DesignProjectPage { project_id: bootstrap.project_id });
                                }
                                Err(e) => {
                                    crate::core::progress::show_error_persistent(&format!("Failed: {}", e));
                                    submitting.set(false);
                                }
                            }
                        });
                    },
                    autocomplete: "off",

                    input {
                        class: "collect-email-input",
                        r#type: "text",
                        placeholder: "project name",
                        value: "{project_name}",
                        oninput: move |e| project_name.set(e.value()),
                        autofocus: true,
                    }
                    input {
                        class: "collect-email-input",
                        r#type: "email",
                        placeholder: "email",
                        value: "{email}",
                        oninput: move |e| email.set(e.value()),
                    }
                    button {
                        class: if can_submit { "collect-email-submit-btn has-email" } else { "collect-email-submit-btn" },
                        r#type: "submit",
                        disabled: !can_submit || *submitting.read(),
                        if *submitting.read() { "Creating..." } else { "Continue" }
                    }
                }
            }
        }
    }
}
