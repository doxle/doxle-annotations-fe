use crate::Route;
use crate::blocks::api::BlockType;
use crate::blocks::state::state_create_block;
use crate::core::AppNavbar;
use dioxus::prelude::*;

const CREATE_BLOCK_CSS: &str = include_str!("create_block_page.css");

#[component]
pub fn CreateBlockPage(project_id: String) -> Element {
    let mut block_name = use_signal(String::new);
    let mut block_type = use_signal(|| BlockType::Building);
    let mut is_submitting = use_signal(|| false);
    let navigator = use_navigator();
    let project_id = use_signal(move || project_id.clone());

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        if *is_submitting.read() {
            crate::core::progress::show_info("Creating block ...");
            return;
        }

        let name = block_name.read().trim().to_string();
        if name.is_empty() {
            crate::core::progress::show_error("Block name is required");
            return;
        }

        let bt = block_type().clone();
        is_submitting.set(true);
        spawn(async move {
            match state_create_block(&project_id(), name.clone(), bt, None).await {
                Ok(_) => {
                    crate::core::progress::show_success(&format!("Block '{}' created", name));
                    navigator.push(Route::BlocksPage {
                        project_id: project_id().clone(),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to create block: {}", e);
                    crate::core::progress::show_error_persistent(&format!(
                        "Failed to create block: {}",
                        e
                    ));
                    is_submitting.set(false);
                }
            }
        });
    };

    rsx! {
        style { {CREATE_BLOCK_CSS} }
        AppNavbar {}
        div {
            class: "create-blocks-page",
            div {
                class: "blocks-form-container",
                // h1 {
                //     class: "create-blocks-title",
                //     "New Block"
                // }
                form {
                    class: "blocks-form",
                    onsubmit: on_submit,
                    autocomplete: "off",
                    div {
                        class: "blocks-form-group",
                        h1 {
                            class: "new-block-label",
                            "# NEW BLOCK"
                        }
                        div {
                            class: "input-with-icons",
                            input {
                                class: "blocks-name-input",
                                r#type: "text",
                                placeholder: "Enter block name",
                                value: "{block_name}",
                                oninput: move |e| block_name.set(e.value()),
                                autofocus: true,
                            }
                            div {
                                class: "input-divider",
                            }
                            div {
                                class: "block-type-icons",
                        button {
                            r#type: "button",
                            class: if block_type() == BlockType::Building { "block-icon-button block-icon-button-first active" } else { "block-icon-button block-icon-button-first" },
                            onclick: move |_| block_type.set(BlockType::Building),
                            img {
                                class: "block-icon light-icon",
                                src: asset!("/assets/icons/build-block-light.svg"),
                            }
                            img {
                                class: "block-icon dark-icon",
                                src: asset!("/assets/icons/build-block-dark.svg"),
                            }
                        }
                        button {
                            r#type: "button",
                            class: if block_type() == BlockType::Annotation { "block-icon-button block-icon-button-middle active" } else { "block-icon-button block-icon-button-middle" },
                            onclick: move |_| block_type.set(BlockType::Annotation),
                            img {
                                class: "block-icon light-icon",
                                src: asset!("/assets/icons/annotation-block-light.svg"),
                            }
                            img {
                                class: "block-icon dark-icon",
                                src: asset!("/assets/icons/annotation-block-dark.svg"),
                            }
                        }
                        button {
                            r#type: "button",
                            class: if block_type() == BlockType::File { "block-icon-button block-icon-button-last active" } else { "block-icon-button block-icon-button-last" },
                            onclick: move |_| block_type.set(BlockType::File),
                            img {
                                class: "block-icon light-icon",
                                src: asset!("/assets/icons/file-block-light.svg"),
                            }
                            img {
                                class: "block-icon dark-icon",
                                src: asset!("/assets/icons/file-block-dark.svg"),
                            }
                        }
                            }
                        }
                    }
                    if !block_name().is_empty() {
                        div {
                            class: "form-actions",
                            button {
                                class: "blocks-back-button",
                                r#type: "button",
                                onclick: move |_| { navigator.push(Route::BlocksPage { project_id: project_id().clone() }); },
                                "Back"
                            }
                            button {
                                class: "blocks-create-button",
                                r#type: "submit",
                                disabled: *is_submitting.read() || block_name().trim().is_empty(),
                                if *is_submitting.read() { "Creating..." } else { "Submit" }
                            }
                        }
                    }
                }
            }
        }
    }
}
