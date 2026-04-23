use super::upload_api::{public_upload_file, create_public_project, UploadedFileInfo};
use crate::core::{Theme, THEME};
use crate::Route;
use dioxus::prelude::*;
use futures::stream::{self, StreamExt};

const CSS: &str = include_str!("upload_plans_page.css");
const FILE_LIGHT_ICON: Asset = asset!("/assets/icons/file-block-light.svg");
const FILE_DARK_ICON: Asset = asset!("/assets/icons/file-block-dark.svg");
const DRAFT_PROJECT_NAME_KEY: &str = "doxle_public_project_name_draft";
const DRAFT_EMAIL_KEY: &str = "doxle_public_email_draft";
const DRAFT_UPLOADED_FILES_KEY: &str = "doxle_uploaded_files";
const LAST_PUBLIC_PROJECT_ID_KEY: &str = "doxle_public_last_project_id";
#[cfg(target_arch = "wasm32")]
fn upload_draft_storage_enabled() -> bool {
    web_sys::window()
        .and_then(|w| w.location().hostname().ok())
        .map(|host| {
            let host = host.to_lowercase();
            host != "doxle.ai" && !host.ends_with(".doxle.ai")
        })
        .unwrap_or(false)
}

#[cfg(not(target_arch = "wasm32"))]
fn upload_draft_storage_enabled() -> bool {
    false
}

#[derive(Clone, PartialEq)]
struct SelectedFile {
    name: String,
    size_mb: f64,
    file: web_sys::File,
}

#[component]
pub fn UploadPlansPage() -> Element {
    let mut selected_files: Signal<Vec<SelectedFile>> = use_signal(Vec::new);
    let mut total_size_mb: Signal<f64> = use_signal(|| 0.0);
    let mut is_uploading = use_signal(|| false);
    let mut is_submitting = use_signal(|| false);
    let mut uploaded_files: Signal<Vec<UploadedFileInfo>> = use_signal(|| {
        #[cfg(target_arch = "wasm32")]
        {
            if upload_draft_storage_enabled() {
                web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                    .and_then(|s| s.get_item(DRAFT_UPLOADED_FILES_KEY).ok().flatten())
                    .and_then(|json| serde_json::from_str::<Vec<UploadedFileInfo>>(&json).ok())
                    .unwrap_or_default()
            } else {
                Vec::new()
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Vec::new()
        }
    });
    let mut project_name = use_signal(|| {
        #[cfg(target_arch = "wasm32")]
        {
            if upload_draft_storage_enabled() {
                web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                    .and_then(|s| s.get_item(DRAFT_PROJECT_NAME_KEY).ok().flatten())
                    .unwrap_or_default()
            } else {
                String::new()
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            String::new()
        }
    });
    let mut email = use_signal(|| {
        #[cfg(target_arch = "wasm32")]
        {
            if upload_draft_storage_enabled() {
                web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                    .and_then(|s| s.get_item(DRAFT_EMAIL_KEY).ok().flatten())
                    .unwrap_or_default()
            } else {
                String::new()
            }
        }
        #[cfg(not(all(target_arch = "wasm32", debug_assertions)))]
        {
            String::new()
        }
    });
    let nav = use_navigator();
    let file_icon = if THEME() == Theme::Dark { FILE_DARK_ICON } else { FILE_LIGHT_ICON };

    let selected_len = selected_files.read().len();
    let uploaded_len = uploaded_files.read().len();
    let has_selected_files = selected_len > 0;
    let has_uploaded_files = uploaded_len > 0;
    let has_any_files = has_selected_files || has_uploaded_files;
    let file_count = if has_selected_files { selected_len } else { uploaded_len };
    let total_display_size_mb = if has_selected_files {
        *total_size_mb.read()
    } else {
        uploaded_files
            .read()
            .iter()
            .map(|f| f.file_size as f64 / 1024.0 / 1024.0)
            .sum::<f64>()
    };
    let has_email = !email().trim().is_empty() && email().contains('@');
    let has_name = project_name().trim().len() >= 3;
    let can_submit = has_uploaded_files && has_name && has_email && !is_submitting();

    rsx! {
        style { {CSS} }
        div {
            class: "upload-plans-page",
            div {
                class: "upload-plans-container",

                if !has_any_files {
                    // ===== UPLOAD VIEW =====
                    h1 {
                        class: "upload-plans-title",
                        "Upload your plans"
                    }
                    div {
                        class: "upload-plans-subtitle",
                        "Select multiple PDFs at once using the shift key"
                    }
                    div {
                        class: "upload-plans-zone",
                        div {
                            class: "upload-plans-zone-empty",
                            label {
                                class: "upload-plans-browse-label",
                                "Browse"
                                input {
                                    class: "upload-plans-browse-input",
                                    r#type: "file",
                                    accept: ".pdf,application/pdf",
                                    multiple: true,
                                    onchange: move |evt| {
                                        #[cfg(target_arch = "wasm32")]
                                        {
                                            use wasm_bindgen::JsCast;
                                            if let Some(web_evt) = evt.downcast::<web_sys::Event>() {
                                                if let Some(target) = web_evt.target() {
                                                    if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
                                                        if let Some(files) = input.files() {
                                                            let mut new_files = Vec::new();
                                                            let mut files_for_upload: Vec<web_sys::File> = Vec::new();
                                                            let mut size_total = 0.0f64;
                                                            for i in 0..files.length() {
                                                                if let Some(file) = files.get(i) {
                                                                    let size_mb = file.size() / 1024.0 / 1024.0;
                                                                    size_total += size_mb;
                                                                    files_for_upload.push(file.clone());
                                                                    new_files.push(SelectedFile {
                                                                        name: file.name(),
                                                                        size_mb,
                                                                        file,
                                                                    });
                                                                }
                                                            }
                                                            if !new_files.is_empty() {
                                                                selected_files.set(new_files);
                                                                total_size_mb.set(size_total);

                                                                // Start S3 upload immediately
                                                                is_uploading.set(true);
                                                                spawn(async move {
                                                                    let futs = files_for_upload.into_iter().enumerate().map(|(idx, file)| {
                                                                        async move {
                                                                            let name = file.name();
                                                                            let result = public_upload_file(file).await;
                                                                            (idx, name, result)
                                                                        }
                                                                    });
                                                                    let total = futs.len();
                                                                    crate::core::progress::show_progress(&format!("Uploading 0/{}", total), 0, total, 0);
                                                                    let mut stream = stream::iter(futs).buffer_unordered(4);
                                                                    let mut finished = 0usize;
                                                                    let mut failed = 0usize;
                                                                    let started = web_time::Instant::now();
                                                                    let mut results: Vec<UploadedFileInfo> = Vec::new();

                                                                    while let Some((_idx, _name, result)) = stream.next().await {
                                                                        finished += 1;
                                                                        match result {
                                                                            Ok(info) => results.push(info),
                                                                            Err(_) => { failed += 1; }
                                                                        }
                                                                        crate::core::progress::show_progress(
                                                                            &format!("Uploading {}/{}", finished, total),
                                                                            finished, total, started.elapsed().as_secs(),
                                                                        );
                                                                    }

                                                                    if failed > 0 {
                                                                        crate::core::progress::show_error_persistent(
                                                                            &format!("{} of {} uploads failed", failed, total),
                                                                        );
                                                                    } else {
                                                                        crate::core::progress::show_success(
                                                                            &format!("Uploaded {} plans", total),
                                                                        );
                                                                    }
                                                                    #[cfg(target_arch = "wasm32")]
                                                                    {
                                                                        if upload_draft_storage_enabled() {
                                                                            if let Some(window) = web_sys::window() {
                                                                                if let Ok(Some(storage)) = window.local_storage() {
                                                                                    if let Ok(serialized) = serde_json::to_string(&results) {
                                                                                        let _ = storage.set_item(DRAFT_UPLOADED_FILES_KEY, &serialized);
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                    uploaded_files.set(results);
                                                                    is_uploading.set(false);
                                                                });
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            span {
                                class: "upload-plans-hint",
                                "Working Drawings, Engineering, Soil Report, Drainage etc"
                            }
                        }
                    }
                } else {
                    // ===== DETAILS VIEW (after files selected) =====
                    h1 {
                        class: "upload-plans-title",
                        "Verifying Email"
                    }
                    div {
                        class: "upload-plans-subtitle",
                        span { class: "upload-plans-subtitle-count", "{file_count}" }
                        span { class: "upload-plans-subtitle-label", " files selected / Total Size: " }
                        span { class: "upload-plans-subtitle-size", "{total_display_size_mb:.2}" }
                        span { class: "upload-plans-subtitle-label", " MB" }
                        if is_uploading() {
                            span { class: "upload-plans-subtitle-label", " · uploading..." }
                        }
                    }

                    // File attachment list with blue bg
                    div {
                        class: "upload-plans-attachments",
                        if has_selected_files {
                            for sf in selected_files.read().iter() {
                                div {
                                    class: "upload-plans-attachment-row",
                                    img {
                                        class: "upload-plans-file-icon",
                                        src: file_icon,
                                    }
                                    span {
                                        class: "upload-plans-file-name",
                                        "{sf.name}"
                                    }
                                    span {
                                        class: "upload-plans-file-size",
                                        "{sf.size_mb:.1} MB"
                                    }
                                }
                            }
                        } else {
                            for uf in uploaded_files.read().iter() {
                                div {
                                    class: "upload-plans-attachment-row",
                                    img {
                                        class: "upload-plans-file-icon",
                                        src: file_icon,
                                    }
                                    span {
                                        class: "upload-plans-file-name",
                                        "{uf.file_name}"
                                    }
                                    span {
                                        class: "upload-plans-file-size",
                                        "{(uf.file_size as f64 / 1024.0 / 1024.0):.1} MB"
                                    }
                                }
                            }
                        }
                    }

                    div {
                        class: "upload-plans-details",
                        input {
                            class: "upload-plans-input",
                            r#type: "text",
                            placeholder: "project name",
                            value: "{project_name}",
                            oninput: move |e| {
                                let value = e.value();
                                project_name.set(value.clone());
                                #[cfg(target_arch = "wasm32")]
                                {
                                    if upload_draft_storage_enabled() {
                                        if let Some(window) = web_sys::window() {
                                            if let Ok(Some(storage)) = window.local_storage() {
                                                let _ = storage.set_item(DRAFT_PROJECT_NAME_KEY, &value);
                                            }
                                        }
                                    }
                                }
                            },
                            autofocus: true,
                        }
                        input {
                            class: "upload-plans-input",
                            r#type: "email",
                            placeholder: "email",
                            value: "{email}",
                            oninput: move |e| {
                                let value = e.value();
                                email.set(value.clone());
                                #[cfg(all(target_arch = "wasm32", debug_assertions))]
                                {
                                    if upload_draft_storage_enabled() {
                                        if let Some(window) = web_sys::window() {
                                            if let Ok(Some(storage)) = window.local_storage() {
                                                let _ = storage.set_item(DRAFT_EMAIL_KEY, &value);
                                            }
                                        }
                                    }
                                }
                            },
                        }
                        p {
                            class: "upload-plans-email-hint",
                            "Your personalised budget estimate will be delivered to your email. Please ensure your email is correct so we can get it to you."
                        }
                    }
                    div {
                        class: "upload-plans-actions",
                        button {
                            class: "upload-plans-undo-btn",
                            disabled: is_uploading(),
                            onclick: move |_| {
                                selected_files.write().clear();
                                total_size_mb.set(0.0);
                                uploaded_files.write().clear();
                                project_name.set(String::new());
                                email.set(String::new());
                                #[cfg(all(target_arch = "wasm32", debug_assertions))]
                                {
                                    if upload_draft_storage_enabled() {
                                        if let Some(window) = web_sys::window() {
                                            if let Ok(Some(storage)) = window.local_storage() {
                                                let _ = storage.remove_item(DRAFT_PROJECT_NAME_KEY);
                                                let _ = storage.remove_item(DRAFT_EMAIL_KEY);
                                                let _ = storage.remove_item(DRAFT_UPLOADED_FILES_KEY);
                                            }
                                        }
                                    }
                                }
                            },
                            "Back"
                        }
                        button {
                            class: "upload-plans-submit-btn",
                            disabled: !can_submit,
                            onclick: move |_| {
                                if !can_submit { return; }

                                let name_val = project_name().trim().to_string();
                                let email_val = email().trim().to_string();
                                let files = uploaded_files().clone();

                                is_submitting.set(true);

                                spawn(async move {
                                    crate::core::progress::show_info("Creating project...");

                                    match create_public_project(&name_val, &email_val, files).await {
                                        Ok(bootstrap) => {
                                            #[cfg(target_arch = "wasm32")]
                                            {
                                                if let Some(window) = web_sys::window() {
                                                    if let Ok(Some(storage)) = window.local_storage() {
                                                        let _ = storage.set_item(LAST_PUBLIC_PROJECT_ID_KEY, &bootstrap.project_id);
                                                    }
                                                }
                                            }
                                            crate::core::progress::show_success("Project created");
                                            // TODO: restore DesignProjectPage for production
                                            nav.push(Route::EstimatePage { project_id: bootstrap.project_id });
                                        }
                                        Err(e) => {
                                            crate::core::progress::show_error_persistent(&format!("Failed: {}", e));
                                            is_submitting.set(false);
                                        }
                                    }
                                });
                            },
                            if is_submitting() { "Creating..." } else if is_uploading() { "Uploading..." } else { "Submit" }
                        }
                    }
                }
            }
        }
    }
}
