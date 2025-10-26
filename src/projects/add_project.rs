use dioxus::prelude::*;
use crate::api::{self, projects::Project};
use crate::state::{add_project, remove_project};

#[component]
pub fn AddProjectModal(show: Signal<bool>) -> Element {
    let mut project_name = use_signal(|| String::new());
    let mut project_type = use_signal(|| "annotation".to_string());
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    let handle_submit = move |e: Event<FormData>| {
        e.prevent_default();
        spawn(async move {
            *loading.write() = true;
            *error.write() = None;

            let create_req = serde_json::json!({
                "name": project_name.read().clone(),
                "type": project_type.read().clone()
            });

            match api::client::post::<serde_json::Value, Project>("/projects", &create_req).await {
                Ok(new_project) => {
                    tracing::info!("✅ Project created: {}", new_project.project_id);
                    
                    // Optimistically add to UI
                    add_project(new_project.clone());
                    tracing::info!("⚡ Project added to UI immediately");
                    
                    // Close modal
                    *show.write() = false;
                    // Reset form
                    *project_name.write() = String::new();
                    *project_type.write() = "annotation".to_string();
                    
                    // DynamoDB Stream will broadcast to other users
                }
                Err(e) => {
                    *error.write() = Some(format!("Failed to create project: {}", e));
                }
            }

            *loading.write() = false;
        });
    };

    let handle_close = move |_| {
        *show.write() = false;
        *error.write() = None;
        *project_name.write() = String::new();
        *project_type.write() = "annotation".to_string();
    };

    if !*show.read() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "modal-overlay",
            onclick: handle_close,

            div {
                class: "modal-content",
                onclick: move |e| e.stop_propagation(),

                div {
                    class: "modal-header",
                    h2 { "Create New Project" }
                    button {
                        class: "modal-close",
                        onclick: handle_close,
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

                    div {
                        class: "form-group",
                        label {
                            r#for: "project-type",
                            "Project Type"
                        }
                        select {
                            id: "project-type",
                            class: "form-input",
                            value: "{project_type}",
                            onchange: move |e| *project_type.write() = e.value(),
                            option { value: "annotation", "Annotation" }
                            option { value: "building", "Building" }
                            option { value: "vehicle", "Vehicle" }
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
                            onclick: handle_close,
                            "Cancel"
                        }
                        button {
                            r#type: "submit",
                            class: "btn-primary",
                            disabled: *loading.read(),
                            if *loading.read() {
                                "Creating..."
                            } else {
                                "Create Project"
                            }
                        }
                    }
                }
            }
        }
    }
}
