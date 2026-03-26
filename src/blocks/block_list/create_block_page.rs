use crate::Route;
use crate::blocks::block_list::api::BlockType;
use crate::blocks::block_list::state::state_create_block;
use crate::shell::AppNavbar;
use dioxus::prelude::*;

const CREATE_BLOCK_CSS: &str = include_str!("create_block_page.css");
const TYPEWRITER_JS: &str = include_str!("../../../js/create_block_typewriter.js");
const CHECKMARK: Asset = asset!("/assets/icons/checkmark_light.svg");

#[component]
pub fn CreateBlockPage(project_id: String) -> Element {
    let mut block_name = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);
    let navigator = use_navigator();
    let project_id = use_signal(move || project_id.clone());
    let has_text = block_name().trim().len() >= 3;

    use_effect(move || {
        document::eval(TYPEWRITER_JS);
    });

    use_effect(move || {
        let js = r#"
            document.addEventListener('blocksubmit', function handler(e) {
                var hidden = document.getElementById('create-block-hidden');
                if (hidden) {
                    var nativeSet = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
                    nativeSet.call(hidden, e.detail);
                    hidden.dispatchEvent(new Event('input', { bubbles: true }));
                    var form = document.getElementById('create-block-form');
                    if (form) form.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
                }
                document.removeEventListener('blocksubmit', handler);
            });
        "#;
        document::eval(js);
    });

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        if *is_submitting.read() {
            crate::shell::progress::show_info("Creating block ...");
            return;
        }

        let name = block_name.read().trim().to_string();
        if name.is_empty() {
            crate::shell::progress::show_error("Block name is required");
            return;
        }

        is_submitting.set(true);
        spawn(async move {
            match state_create_block(&project_id(), name.clone(), BlockType::Annotation, None).await {
                Ok(_) => {
                    crate::shell::progress::show_success(&format!("Block '{}' created", name));
                    navigator.push(Route::BlocksPage {
                        project_id: project_id().clone(),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to create block: {}", e);
                    crate::shell::progress::show_error_persistent(&format!(
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
            form {
                id: "create-block-form",
                class: "create-block-form",
                onsubmit: on_submit,
                autocomplete: "off",
                div {
                    id: "create-block-input",
                    class: "create-block-input",
                    contenteditable: "true",
                    spellcheck: "false",
                }
                input {
                    id: "create-block-hidden",
                    r#type: "hidden",
                    value: "{block_name}",
                    oninput: move |e| block_name.set(e.value()),
                }
                if has_text {
                    button {
                        r#type: "submit",
                        class: "create-block-submit",
                        disabled: *is_submitting.read(),
                        img { src: CHECKMARK }
                    }
                } else {
                    div { class: "create-block-submit-placeholder" }
                }
            }
        }
    }
}
