use dioxus::prelude::*;
use crate::Route;
use crate::core::{AppNavbar, ProtectedRoute};
use super::project_state::state_create_project;

const CSS: &str = include_str!("create_project.css");

#[component]
pub fn CreateProjectPage() -> Element {
    let mut name = use_signal(String::new);
    let mut submitting = use_signal(|| false);
    let nav = use_navigator();
    let has_text = name().trim().len() >= 3;

    let on_submit = move |e: Event<FormData>| {
        e.prevent_default();
        if *submitting.read() { return; }
        let val = name.read().trim().to_string();
        if val.is_empty() { return; }
        submitting.set(true);
        spawn(async move {
            match state_create_project(val, None, None).await {
                Ok(_) => { nav.push(Route::ProjectsPage {}); }
                Err(e) => {
                    crate::core::progress::show_error_persistent(&format!("Failed: {}", e));
                    submitting.set(false);
                }
            }
        });
    };

    rsx! {
        style { {CSS} }
        ProtectedRoute {
            AppNavbar {}
            div { class: "create-project-page",
                form { class: "create-project-form", onsubmit: on_submit, autocomplete: "off",
                    div { class: "project-form-group",
                        h1 { class: "new-project-label", "# NEW PROJECT" }
                        div { class: "project-input-container",
                            input {
                                class: "project-name-input",
                                r#type: "text",
                                placeholder: "Enter project name",
                                value: "{name}",
                                oninput: move |e| name.set(e.value()),
                                autofocus: true,
                            }
                        }
                    }
                    if !name().is_empty() {
                        div { class: "project-form-actions",
                            button {
                                class: "project-back-button",
                                r#type: "button",
                                onclick: move |_| { nav.push(Route::ProjectsPage {}); },
                                "Back"
                            }
                            button { r#type: "submit", class: "create-project-submit", disabled: *submitting.read() || name().trim().is_empty(),
                                if *submitting.read() { "Creating..." } else { "Submit" }
                            }
                        }
                    }
                }
            }
        }
    }
}
