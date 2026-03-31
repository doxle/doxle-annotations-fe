use dioxus::prelude::*;

const EDIT_TASK_MODAL_CSS: &str = r#"
.edit-task-modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
}
.edit-task-modal {
    background: var(--bg-primary);
    border-radius: 8px;
    padding: 24px;
    min-width: 320px;
    max-width: 90vw;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}
.edit-task-modal-title {
    font-size: 18px;
    font-weight: 500;
    margin-bottom: 16px;
    color: var(--text-primary);
}
.edit-task-modal-input {
    width: 100%;
    padding: 12px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 16px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    margin-bottom: 20px;
}
.edit-task-modal-input:focus {
    outline: none;
    border-color: var(--accent-blue);
}
.edit-task-modal-buttons {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
}
.edit-task-modal-btn {
    padding: 10px 20px;
    border-radius: 4px;
    font-size: 14px;
    cursor: pointer;
    border: none;
}
.edit-task-modal-btn-cancel {
    background: var(--bg-tertiary);
    color: var(--text-primary);
}
.edit-task-modal-btn-save {
    background: var(--accent-blue);
    color: white;
}
.edit-task-modal-btn-save:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}
"#;

#[component]
pub fn EditTaskModal(
    task_id: String,
    current_name: String,
    on_save: EventHandler<String>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut name = use_signal(|| current_name.clone());
    let is_valid = !name().trim().is_empty() && name().trim() != current_name;

    rsx! {
        style { {EDIT_TASK_MODAL_CSS} }
        div {
            class: "edit-task-modal-overlay",
            onclick: move |_| on_cancel.call(()),
            div {
                class: "edit-task-modal",
                onclick: move |e| e.stop_propagation(),
                h2 { class: "edit-task-modal-title", "Edit Task Name" }
                input {
                    class: "edit-task-modal-input",
                    r#type: "text",
                    value: "{name}",
                    autofocus: true,
                    oninput: move |e| name.set(e.value().clone()),
                    onkeypress: move |e| {
                        if e.key() == Key::Enter && is_valid {
                            on_save.call(name().trim().to_string());
                        }
                    }
                }
                div {
                    class: "edit-task-modal-buttons",
                    button {
                        class: "edit-task-modal-btn edit-task-modal-btn-cancel",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "edit-task-modal-btn edit-task-modal-btn-save",
                        disabled: !is_valid,
                        onclick: move |_| {
                            if is_valid {
                                on_save.call(name().trim().to_string());
                            }
                        },
                        "Save"
                    }
                }
            }
        }
    }
}
