use dioxus::prelude::*;
use crate::Route;
use crate::shell::{AppNavbar, ProtectedRoute};
use super::state::state_create_project;

const CSS: &str = include_str!("create_project_page.css");
const TYPEWRITER_JS: &str = include_str!("../../js/create_project_typewriter.js");
const CHECKMARK: Asset = asset!("/assets/icons/checkmark_light.svg");

#[component]
pub fn CreateProjectPage() -> Element {
    let mut name = use_signal(String::new);
    let mut submitting = use_signal(|| false);
    let nav = use_navigator();
    let has_text = name().trim().len() >= 3;

    use_effect(move || {
        document::eval(TYPEWRITER_JS);
    });

    // Listen for submit from JS
    use_effect(move || {
        let js = r#"
            document.addEventListener('projectsubmit', function handler(e) {
                var hidden = document.getElementById('create-project-hidden');
                if (hidden) {
                    var nativeSet = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
                    nativeSet.call(hidden, e.detail);
                    hidden.dispatchEvent(new Event('input', { bubbles: true }));
                    // Submit the form
                    var form = document.getElementById('create-project-form');
                    if (form) form.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
                }
                document.removeEventListener('projectsubmit', handler);
            });
        "#;
        document::eval(js);
    });

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
                    crate::shell::progress::show_error_persistent(&format!("Failed: {}", e));
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
                form { id: "create-project-form", class: "create-project-form", onsubmit: on_submit, autocomplete: "off",
                    div {
                        id: "create-project-input",
                        class: "create-project-input",
                        contenteditable: "true",
                        spellcheck: "false",
                    }
                    // Hidden input syncs contenteditable text to Dioxus state
                    input {
                        id: "create-project-hidden",
                        r#type: "hidden",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                    if has_text {
                        button { r#type: "submit", class: "create-project-submit", disabled: *submitting.read(),
                            img { src: CHECKMARK }
                        }
                    } else {
                        div { class: "create-project-submit-placeholder" }
                    }
                }
            }
        }
    }
}
