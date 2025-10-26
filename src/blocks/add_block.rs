use dioxus::prelude::*;
use dioxus::html::HasFileData;
use web_sys::File;
use crate::state::blocks::BLOCKS;
use crate::api::uploads;
use crate::state::load_block_images;

#[derive(Clone)]
pub struct BlockFormData {
    pub name: String,
    pub assigned_to: Option<String>,
    pub images: Vec<String>, // URLs or base64 strings for now
}

#[derive(Clone)]
struct PendingUpload {
    file: File,
    preview_url: String,
}

#[component]
pub fn AddBlockModal(show: Signal<bool>, project_id: String, on_add: EventHandler<String>) -> Element {
    let mut block_name = use_signal(|| String::new());
    let mut assigned_to = use_signal(|| None::<String>);
    let mut pending_uploads = use_signal(|| Vec::<PendingUpload>::new());
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);
    let mut upload_progress = use_signal(|| None::<String>);

    let handle_submit = move |e: Event<FormData>| {
        e.prevent_default();
        
        let project_id = project_id.clone();
        let block_name_val = block_name.read().clone();
        let uploads = pending_uploads.read().clone();
        
        tracing::info!("Submit clicked. Block name: {}, Files: {}", block_name_val, uploads.len());
        
        spawn(async move {
            *loading.write() = true;
            *error.write() = None;
            
            tracing::info!("Creating block: {}", block_name_val);
            
            // First create the block
            on_add.call(block_name_val.clone());
            
            // Wait a bit for block creation
            gloo_timers::future::TimeoutFuture::new(500).await;
            
            // Get the created block ID from state
            let block_id = match BLOCKS.read().iter()
                .find(|b| b.name == block_name_val)
                .map(|b| b.block_id.clone()) {
                    Some(id) => {
                        tracing::info!("Found block ID: {}", id);
                        id
                    },
                    None => {
                        tracing::error!("Failed to find created block");
                        *error.write() = Some("Failed to get block ID".to_string());
                        *loading.write() = false;
                        return;
                    }
            };
            
            // Upload all files if any
            if !uploads.is_empty() {
                tracing::info!("Starting upload of {} files", uploads.len());
                *upload_progress.write() = Some(format!("Preparing uploads..."));
                
                for (idx, pending) in uploads.iter().enumerate() {
                    let progress_msg = format!("Uploading {} of {}...", idx + 1, uploads.len());
                    tracing::info!("{}", progress_msg);
                    *upload_progress.write() = Some(progress_msg);
                    
                    match uploads::upload_file(&project_id, &block_id, pending.file.clone()).await {
                        Ok(image_id) => {
                            tracing::info!("✓ Uploaded image: {}", image_id);
                        }
                        Err(e) => {
                            tracing::error!("✗ Upload failed: {}", e);
                            *error.write() = Some(format!("Upload failed: {}", e));
                            *loading.write() = false;
                            *upload_progress.write() = None;
                            return;
                        }
                    }
                }
                
                tracing::info!("All uploads completed!");
                
                // Reload images for the block to show them in the UI
                load_block_images(&block_id).await;
            }
            
            // Close modal and reset
            *show.write() = false;
            *block_name.write() = String::new();
            *assigned_to.write() = None;
            *pending_uploads.write() = Vec::new();
            *loading.write() = false;
            *upload_progress.write() = None;
        });
    };

    let handle_close = move |_| {
        *show.write() = false;
        *error.write() = None;
        *block_name.write() = String::new();
        *assigned_to.write() = None;
        *pending_uploads.write() = Vec::new();
        *upload_progress.write() = None;
    };

    if !*show.read() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "add-block-modal",
            onclick: handle_close,

            div {
                class: "add-block-modal-content",
                onclick: move |e| e.stop_propagation(),

                div {
                    class: "add-block-modal-header",
                    h2 { "Create New Block" }
                    button {
                        class: "add-block-modal-close",
                        onclick: handle_close,
                        "×"
                    }
                }

                form {
                    class: "add-block-modal-body",
                    onsubmit: handle_submit,

                    div {
                        class: "add-block-form-group",
                        label {
                            r#for: "block-name",
                            "Block Name *"
                        }
                        input {
                            id: "block-name",
                            class: "add-block-form-input",
                            r#type: "text",
                            placeholder: "Enter block name",
                            value: "{block_name}",
                            required: true,
                            oninput: move |e| *block_name.write() = e.value(),
                        }
                    }
                    
                    div {
                        class: "add-block-form-group",
                        label {
                            r#for: "assigned-to",
                            "Assign To"
                        }
                        select {
                            id: "assigned-to",
                            class: "add-block-form-select",
                            value: "{assigned_to.read().as_ref().unwrap_or(&String::new())}",
                            onchange: move |e| {
                                let val = e.value();
                                *assigned_to.write() = if val.is_empty() { None } else { Some(val) };
                            },
                            option { value: "", "Unassigned" }
                            option { value: "user1", "Kim Beale" }
                            option { value: "user2", "John Smith" }
                            option { value: "user3", "Sarah Jones" }
                        }
                    }
                    
                    div {
                        class: "add-block-form-group",
                        label { "Images" }
                        div {
                            class: "add-block-dropzone",
                            ondragover: move |e| {
                                e.prevent_default();
                            },
                            ondrop: move |e| {
                                e.prevent_default();
                                async move {
                                    let file_data_vec = (*e.data()).files();
                                    tracing::info!("Files dropped: {}", file_data_vec.len());
                                    
                                    for file_data in file_data_vec {
                                        let file_name = file_data.name();
                                        tracing::info!("Processing dropped: {}", file_name);
                                        
                                        match file_data.read_bytes().await {
                                            Ok(contents) => {
                                                let array = js_sys::Uint8Array::from(&contents[..]);
                                                let file_array = js_sys::Array::new();
                                                file_array.push(&array);
                                                
                                                let mime = if file_name.ends_with(".png") {
                                                    "image/png"
                                                } else if file_name.ends_with(".jpg") || file_name.ends_with(".jpeg") {
                                                    "image/jpeg"
                                                } else {
                                                    "image/*"
                                                };
                                                
                                                let mut blob_props = web_sys::BlobPropertyBag::new();
                                                blob_props.set_type(mime);
                                                
                                                if let Ok(blob) = web_sys::Blob::new_with_u8_array_sequence_and_options(
                                                    &file_array,
                                                    &blob_props
                                                ) {
                                                    let mut file_props = web_sys::FilePropertyBag::new();
                                                    file_props.set_type(mime);
                                                    
                                                    if let Ok(file) = web_sys::File::new_with_blob_sequence_and_options(
                                                        &js_sys::Array::of1(&blob),
                                                        &file_name,
                                                        &file_props
                                                    ) {
                                                        let url = web_sys::Url::create_object_url_with_blob(&file)
                                                            .unwrap_or_default();
                                                        
                                                        pending_uploads.write().push(PendingUpload {
                                                            file,
                                                            preview_url: url,
                                                        });
                                                        
                                                        tracing::info!("Added dropped: {} to uploads", file_name);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to read dropped file {}: {:?}", file_name, e);
                                            }
                                        }
                                    }
                                }
                            },
                            
                            label {
                                class: "add-block-dropzone-label",
                                
                                input {
                                    class: "add-block-file-input-hidden",
                                    r#type: "file",
                                    multiple: true,
                                    accept: "image/*",
                                    
                                    onchange: move |evt| {
                                        async move {
                                            let file_data_vec = evt.files();
                                            tracing::info!("Files selected: {}", file_data_vec.len());
                                            
                                            for file_data in file_data_vec {
                                                let file_name = file_data.name();
                                                tracing::info!("Processing: {}", file_name);
                                                
                                                // Read file contents as bytes
                                                match file_data.read_bytes().await {
                                                    Ok(contents) => {
                                                        // Create File object from bytes
                                                        let array = js_sys::Uint8Array::from(&contents[..]);
                                                        let file_array = js_sys::Array::new();
                                                        file_array.push(&array);
                                                        
                                                        let mime = if file_name.ends_with(".png") {
                                                            "image/png"
                                                        } else if file_name.ends_with(".jpg") || file_name.ends_with(".jpeg") {
                                                            "image/jpeg"
                                                        } else {
                                                            "image/*"
                                                        };
                                                        
                                                        let mut blob_props = web_sys::BlobPropertyBag::new();
                                                        blob_props.set_type(mime);
                                                        
                                                        if let Ok(blob) = web_sys::Blob::new_with_u8_array_sequence_and_options(
                                                            &file_array,
                                                            &blob_props
                                                        ) {
                                                            let mut file_props = web_sys::FilePropertyBag::new();
                                                            file_props.set_type(mime);
                                                            
                                                            if let Ok(file) = web_sys::File::new_with_blob_sequence_and_options(
                                                                &js_sys::Array::of1(&blob),
                                                                &file_name,
                                                                &file_props
                                                            ) {
                                                                let url = web_sys::Url::create_object_url_with_blob(&file)
                                                                    .unwrap_or_default();
                                                                
                                                                pending_uploads.write().push(PendingUpload {
                                                                    file,
                                                                    preview_url: url,
                                                                });
                                                                
                                                                tracing::info!("Added: {} to uploads", file_name);
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        tracing::error!("Failed to read file {}: {:?}", file_name, e);
                                                    }
                                                }
                                            }
                                        }
                                    },
                                }
                                
                                if pending_uploads.read().is_empty() {
                                    div {
                                        class: "add-block-dropzone-content",
                                        svg {
                                            class: "add-block-dropzone-icon",
                                            width: "48",
                                            height: "48",
                                            view_box: "0 0 24 24",
                                            fill: "none",
                                            stroke: "currentColor",
                                            stroke_width: "1.5",
                                            path {
                                                d: "M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5"
                                            }
                                        }
                                        p { 
                                            class: "add-block-dropzone-text", 
                                            "Drag and drop images or click to browse" 
                                        }
                                        p { 
                                            class: "add-block-dropzone-hint", 
                                            "PNG, JPG - supports large files" 
                                        }
                                    }
                                } else {
                                    // Selected files summary
                                    {
                                        let count = pending_uploads.read().len();
                                        let total_bytes: f64 = pending_uploads.read().iter().map(|p| p.file.size()).sum();
                                        let mb = total_bytes / (1024.0*1024.0);
                                        rsx!{
                                            div { class: "add-block-selected-info", "Selected: {count} files ({mb:.1} MB)" }
                                        }
                                    }
                                    div {
                                        class: "add-block-images-preview",
                                        for (idx, pending) in pending_uploads.read().iter().enumerate() {
                                            div {
                                                key: "{idx}",
                                                class: "add-block-image-item",
                                                img {
                                                    src: "{pending.preview_url}",
                                                    alt: "Image {idx}"
                                                }
                                                button {
                                                    class: "add-block-image-remove",
                                                    onclick: move |_| {
                                                        pending_uploads.write().remove(idx);
                                                    },
                                                    "×"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(progress) = upload_progress.read().as_ref() {
                        div {
                            class: "add-block-upload-progress",
                            "{progress}"
                        }
                    }
                    
                    if let Some(err) = error.read().as_ref() {
                        div {
                            class: "add-block-form-error",
                            "{err}"
                        }
                    }

                    div {
                        class: "add-block-modal-footer",
                        button {
                            r#type: "button",
                            class: "add-block-btn-cancel",
                            onclick: handle_close,
                            "Cancel"
                        }
                        button {
                            r#type: "submit",
                            class: "add-block-btn-submit",
                            disabled: *loading.read(),
                            if *loading.read() {
                                "Creating..."
                            } else {
                                "Create Block"
                            }
                        }
                    }
                }
            }
        }
    }
}
