use dioxus::prelude::*;
use crate::api::{self, projects::Project};
use crate::state::load_projects;

#[component]
pub fn RenameProjectModal(
    show: Signal<bool>,
    project_id: String,
    current_name: String,
) -> Element {
    let mut project_name = use_signal(|| current_name.clone());
    let original_name = use_signal(|| current_name.clone());
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    let pid = project_id.clone();
    let handle_submit = move |e: Event<FormData>| {
        e.prevent_default();
        let pid = pid.clone();
        spawn(async move {
            *loading.write() = true;
            *error.write() = None;

            let new_name = project_name.read().clone();
            tracing::info!("Renaming project {} to {}", pid, new_name);
            
            let update_req = serde_json::json!({
                "name": new_name,
            });

            match api::client::patch::<serde_json::Value, Project>(
                &format!("/projects/{}", pid),
                &update_req
            ).await {
                Ok(_) => {
                    tracing::info!("Rename successful, reloading projects");
                    load_projects().await;
                    *show.write() = false;
                }
                Err(e) => {
                    tracing::error!("Rename failed: {}", e);
                    *error.write() = Some(format!("Failed to rename project: {}", e));
                }
            }

            *loading.write() = false;
        });
    };

    if !*show.read() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| {
                *show.write() = false;
                *error.write() = None;
                *project_name.write() = original_name.read().clone();
            },

            div {
                class: "modal-content",
                onclick: move |e| e.stop_propagation(),

                div {
                    class: "modal-header",
                    h2 { "Rename Project" }
                    button {
                        class: "modal-close",
                        onclick: move |_| {
                            *show.write() = false;
                            *error.write() = None;
                            *project_name.write() = original_name.read().clone();
                        },
                        "×"
                    }
                }

                form {
                    class: "modal-body",
                    onsubmit: handle_submit,

                    div {
                        class: "form-group",
                        label {
                            r#for: "project-name",
                            "Project Name"
                        }
                        input {
                            id: "project-name",
                            class: "form-input",
                            r#type: "text",
                            placeholder: "Enter project name",
                            value: "{project_name}",
                            required: true,
                            oninput: move |e| *project_name.write() = e.value(),
                        }
                    }

                    if let Some(err) = error.read().as_ref() {
                        div {
                            class: "form-error",
                            "{err}"
                        }
                    }

                    div {
                        class: "modal-footer",
                        button {
                            r#type: "button",
                            class: "btn-secondary",
                            onclick: move |_| {
                                *show.write() = false;
                                *error.write() = None;
                                *project_name.write() = original_name.read().clone();
                            },
                            "Cancel"
                        }
                        button {
                            r#type: "submit",
                            class: "btn-primary",
                            disabled: *loading.read(),
                            if *loading.read() {
                                "Renaming..."
                            } else {
                                "Rename"
                            }
                        }
                    }
                }
            }
        }
    }
}
