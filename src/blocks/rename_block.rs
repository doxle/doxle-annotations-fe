use crate::api::blocks_api::Block;
use crate::state::{rename_block, load_project_blocks, BLOCKS};
use dioxus::prelude::*;

const RENAME_BLOCK_CSS: &str = include_str!("rename_block.css");

#[component]
pub fn RenameBlockModal(
    show_dialog: Signal<bool>,
    block_id: String,
    project_id: String,
    current_name: String,
) -> Element {
    let mut block_name = use_signal(|| current_name.clone());
    let original_name = use_signal(|| current_name.clone());
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    let handle_submit = move |e: Event<FormData>| {
        e.prevent_default();
        let pid = project_id.clone();
        let bid = block_id.clone(); //Clone for closure inside spawn

        spawn(async move {
            // inner move closures here
            *loading.write() = true;
            *error.write() = None;

            let new_name = block_name.read().clone();
            tracing::info!("Renaming block {} to {}", bid, new_name);

            // Close modal immediately
            *show_dialog.write() = false;
            *loading.write() = false;

            // Call state layer function - it handles optimistic update + rollback
            let pid = pid.clone();
            rename_block(&pid, &bid, new_name.clone()).await;

            *loading.write() = false;
        });
    };

    if !*show_dialog.read() {
        return rsx! {};
    }

    rsx! {
        style { {RENAME_BLOCK_CSS} }
        div {
            class: "rename-block-overlay",
            onclick: move |_| {
                *show_dialog.write() = false;
                *error.write() = None;
                *block_name.write() = original_name.read().clone();
            },

            div {
                class: "rename-block-content",
                onclick: move |e| e.stop_propagation(),

                div {
                    class: "rename-block-header",
                    h2 { "Rename Block" }
                    button {
                        class: "rename-block-close",
                        onclick: move |_| {
                            *show_dialog.write() = false;
                            *error.write() = None;
                            *block_name.write() = original_name.read().clone();
                        },
                        "×"
                    }
                }

                form {
                    class: "rename-block-body",
                    onsubmit: handle_submit,

                    div {
                        class: "rename-block-form-group",
                        label {
                            r#for: "block-name",
                            "Block Name"
                        }
                        input {
                            id: "block-name",
                            class: "rename-block-input",
                            r#type: "text",
                            placeholder: "Enter block name",
                            value: "{block_name}",
                            required: true,
                            oninput: move |e| *block_name.write() = e.value(),
                        }
                    }

                    if let Some(err) = error.read().as_ref() {
                        div {
                            class: "rename-block-error",
                            "{err}"
                        }
                    }

                    div {
                        class: "rename-block-footer",
                        button {
                            r#type: "button",
                            class: "rename-block-btn-secondary",
                            onclick: move |_| {
                                *show_dialog.write() = false;
                                *error.write() = None;
                                *block_name.write() = original_name.read().clone();
                            },
                            "Cancel"
                        }
                        button {
                            r#type: "submit",
                            class: "rename-block-btn-primary",
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
