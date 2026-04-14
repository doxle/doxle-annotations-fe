use dioxus::prelude::*;
use crate::core::{THEME, Theme, AppNavbar, BottomBar};
use dioxus::logger::tracing::error;
use crate::tasks::create_task_handler::{handle_create_task, PendingUpload};
use crate::Route;

const CREATE_TASK_CSS: &str = include_str!("create_task_page.css");
const TASKS_ICON_LIGHT: Asset = asset!("/assets/icons/tasks-light.svg");
const TASKS_ICON_DARK: Asset = asset!("/assets/icons/tasks-dark.svg");
const CLOSE_ICON_LIGHT: Asset = asset!("/assets/icons/close-light.svg");
const CLOSE_ICON_DARK: Asset = asset!("/assets/icons/close-dark.svg");



fn format_size(bytes: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    if bytes >= GB {
        format!("{:.2} GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes / MB)
    } else {
        format!("{:.2} KB", bytes / KB)
    }
}




#[derive(Props, Clone, PartialEq)]
pub struct CreateTaskPageProps {
    pub project_id: String,
    pub block_id: String,
    pub block_name: String,
    pub block_type: String,
}

#[component]
pub fn CreateTaskPage(props: CreateTaskPageProps) -> Element {
    let mut task_name = use_signal(String::new);
    let mut pending_uploads = use_signal(|| Vec::<PendingUpload>::new());
    let mut is_submitting = use_signal(|| false);
    let mut upload_current = use_signal(|| 0usize);
    let mut upload_total = use_signal(|| 0usize);
    let nav = use_navigator();
    
    let project_id = props.project_id.clone();
    let block_id = props.block_id.clone();
    let block_name = props.block_name.clone();
    let block_type = props.block_type.clone();
    let project_id_for_back = project_id.clone();
    let block_id_for_back = block_id.clone();
    let block_name_for_back = block_name.clone();
    let block_type_for_back = block_type.clone();

    let is_dark = THEME() == Theme::Dark;
    let tasks_icon = if is_dark { TASKS_ICON_DARK } else { TASKS_ICON_LIGHT };
    let close_icon = if is_dark { CLOSE_ICON_DARK } else { CLOSE_ICON_LIGHT };

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        evt.stop_propagation();
        // is_submitting.set(true); // So create task button cannot be pressed twice

        if *is_submitting.read() { 
            crate::core::progress::show_info("Creating task ....");
            return; 
        }

        let name = task_name.read().trim().to_string();
        if name.is_empty() { 
            crate::core::progress::show_error("Task name is required:");
            return; 
        }

        let images_to_upload = pending_uploads.read().clone();
        if images_to_upload.is_empty() {
            error!("Cannot create task without images");
            crate::core::progress::show_error("Cannot create task without images:");
            return;
        }

        let project_id_clone = project_id.clone();
        let block_id_clone = block_id.clone();
        let block_name_clone = block_name.clone();
        let block_type_clone = block_type.clone();
        let nav = nav.clone();

        spawn(async move {
            handle_create_task(
                project_id_clone,
                block_id_clone,
                block_name_clone,
                block_type_clone,
                name,
                images_to_upload,
                nav,
                is_submitting,
                upload_current,
                upload_total,
            ).await;
        });
    };

    rsx! {
        style { {CREATE_TASK_CSS} }
        AppNavbar {}
        div {
            class: "create-tasks-page",
            div {
                class: "tasks-form-container",
                div {
                    class: "tasks-form-group",
                    h1 { class: "create-tasks-title", "# NEW TASK" }
                }
                form {
                    class: "tasks-form",
                    onsubmit: handle_submit,
                    autocomplete: "off",

                    div {
                        class: "tasks-form-group",
                        input {
                            class: "tasks-name-input",
                            r#type: "text",
                            autocomplete: "off",
                            placeholder: "Task name",
                            value: "{task_name}",
                            oninput: move |e| *task_name.write() = e.value(),
                            disabled: *is_submitting.read(),
                        }
                    }

                    div {
                        class: "tasks-form-group",
                        
                        div {
                            class: "tasks-file-upload-container",
                            
                            // Hidden file input
                            input {
                                r#type: "note",
                                id: "task-file-input",
                                class: "hidden-file-input",
                                multiple: true,
                                accept: "image/*",
                                onchange: move |evt| {
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        use wasm_bindgen::JsCast;
                                        if let Some(web_evt) = evt.downcast::<web_sys::Event>() {
                                            if let Some(target) = web_evt.target() {
                                                if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
                                                    if let Some(files) = input.files() {
                                                        for i in 0..files.length() {
                                                            if let Some(file) = files.get(i) {
                                                                let name = file.name();
                                                                pending_uploads.write().push(PendingUpload { file, name });
                                                            }
                                                        }

                                                        // Calculate sum
                                                        let total_bytes: f64 = pending_uploads.iter().map(|u| u.file.size()).sum();
                                                        // Show status
                                                        crate::core::progress::show_info_for(&format!("Total upload size: {}", format_size(total_bytes)), 1);


                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Custom outline button
                            label {
                                r#for: "task-file-input",
                                class: "choose-image-btn",
                                "Choose images"
                            }

                            // Selected files list
                            if !pending_uploads.read().is_empty() {
                                div {
                                    class: "selected-files-list",
                                    for (idx, upload) in pending_uploads.read().iter().enumerate() {
                                        div {
                                            key: "{idx}",
                                            class: "selected-file-item",
                                            span { class: "file-name", "{upload.name}" }
                                            button {
                                                r#type: "button",
                                                class: "remove-file-btn",
                                                onclick: move |_| {
                                                    pending_uploads.write().remove(idx); 
                                                    // Recalculate sum when imgs are removed
                                                    let total_bytes: f64 = pending_uploads.iter().map(|u| u.file.size()).sum();
                                                    crate::core::progress::show_info(&format!("Total upload size: {}", format_size(total_bytes)));
                                                },
                                                img {
                                                    src: close_icon,
                                                    class: "remove-file-icon",
                                                    alt: "Remove"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Upload progress bar
                    if *is_submitting.read() && *upload_total.read() > 0 {
                        div {
                            class: "upload-progress-container",
                            div {
                                class: "upload-progress-bar",
                                div {
                                    class: "upload-progress-fill",
                                    style: "width: {(*upload_current.read() as f64 / *upload_total.read() as f64 * 100.0) as u32}%",
                                }
                            }
                            span {
                                class: "upload-progress-text",
                                "Uploading {upload_current}/{upload_total}"
                            }
                        }
                    }

                    if !task_name().is_empty() {
                        div {
                            class: "form-actions",
                            button {
                                r#type: "button",
                                class: "tasks-back-button",
                                disabled: *is_submitting.read(),
                                onclick: move |_| {
                                    nav.push(Route::TasksListPage {
                                        project_id: project_id_for_back.clone(),
                                        block_id: block_id_for_back.clone(),
                                        block_name: crate::core::route_utils::encode_route_segment(&block_name_for_back),
                                        block_type: block_type_for_back.clone(),
                                    });
                                },
                                "Back"
                            }
                            button {
                                r#type: "submit",
                                class: "tasks-create-button",
                                disabled: *is_submitting.read(),
                                if *is_submitting.read() { "Uploading..." } else { "Submit" }
                            }
                        }
                    }
                }
            }
        }
        BottomBar { project_id: props.project_id.clone() }
    }
}
