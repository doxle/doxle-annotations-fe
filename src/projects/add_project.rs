use crate::api::{
    self,
    projects_api::{Label, Project, LABEL_COLORS},
};
use crate::state::add_project;
use dioxus::prelude::*;

#[component]
pub fn AddProjectModal(show: Signal<bool>) -> Element {
    let mut project_name = use_signal(|| String::new());
    let mut labels = use_signal(|| Vec::<Label>::new());
    let mut label_name = use_signal(|| String::new());
    let mut label_color_index = use_signal(|| 0_usize);
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
                "project_type": project_type.read().clone(),
                "labels": labels.read().clone(),
            });

            match api::client_api::post::<serde_json::Value, Project>("/projects", &create_req)
                .await
            {
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
                    *labels.write() = Vec::new();
                    *label_name.write() = String::new();
                    *label_color_index.write() = 0;

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
        *labels.write() = Vec::new();
        *label_name.write() = String::new();
        *label_color_index.write() = 0;
    };

    let handle_add_label = move |_: Event<MouseData>| {
        let name = label_name();
        let current_idx = label_color_index();
        let color = LABEL_COLORS[current_idx].to_string();

        if !name.trim().is_empty() {
            let mut current_labels = labels();
            current_labels.push(Label {
                name: name.trim().to_string(),
                color,
            });
            labels.set(current_labels);
            label_name.set(String::new());
            label_color_index.set((current_idx + 1) % LABEL_COLORS.len());
        }
    };

    let mut handle_remove_label = move |index: usize| {
        let mut current_labels = labels.read().clone();
        current_labels.remove(index);
        *labels.write() = current_labels;
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
                    div {
                        class: "form-group",
                        label { "Labels (at least 1 required)" }

                        // Display existing labels
                        div {
                            class: "labels-list",
                            for (index, label) in labels.read().iter().enumerate() {
                                div {
                                    key: "{index}",
                                    class: "label-chip",
                                    style: "--label-color: {label.color};",
                                    span {
                                        "{label.name}"
                                    }
                                    button {
                                        r#type: "button",
                                        onclick: move |_| handle_remove_label(index),
                                        "×"
                                    }
                                }
                            }
                        }
                        // Add new label input
                        div {
                            class: "label-input-row",
                            input {
                                class: "form-input",
                                r#type: "text",
                                placeholder: "Label name (e.g., Door, Window)",
                                value: "{label_name}",
                                oninput: move |e| *label_name.write() = e.value(),
                                onkeydown: move |e| {
                                    if e.key() == Key::Enter {
                                        e.prevent_default();
                                        let current_name = label_name();
                                        let current_color_idx = label_color_index();
                                        let color = LABEL_COLORS[current_color_idx].to_string();
                                        if !current_name.trim().is_empty() {
                                            let mut current_labels = labels();
                                            current_labels.push(Label {
                                                name: current_name.trim().to_string(),
                                                color,
                                            });
                                            labels.set(current_labels);
                                            label_name.set(String::new());
                                            label_color_index.set((current_color_idx + 1) % LABEL_COLORS.len());
                                        }
                                    }
                                },
                            }
                            button {
                                r#type: "button",
                                class: "btn-secondary",
                                onclick: handle_add_label,
                                "Add Label"
                            }
                        }
                        // Color palette grid
                        div {
                            class: "color-palette-container",
                            div {
                                class: "color-palette-label",
                                "Choose color:"
                            }
                            div {
                                class: "color-palette-grid",
                                for (idx, color) in LABEL_COLORS.iter().enumerate() {
                                    button {
                                        key: "{idx}",
                                        r#type: "button",
                                        class: if *label_color_index.read() == idx { "color-swatch selected" } else { "color-swatch" },
                                        style: "--swatch-color: {color};",
                                        onclick: move |_| *label_color_index.write() = idx,
                                        title: "{color}",
                                    }
                                }
                            }
                        }
                    }

                    // Validation warning if no labels
                    if labels.read().is_empty() {
                        div {
                            class: "label-warning",
                            "⚠️ At least one label is required to create a project"
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
                            disabled: *loading.read() || labels.read().is_empty(),
                            if *loading.read() {
                                "Creating..."
                            } else if labels.read().is_empty() {
                                "Add at least 1 label"
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
