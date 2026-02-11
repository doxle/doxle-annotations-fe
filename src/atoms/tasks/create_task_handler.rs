

use dioxus::prelude::*;
use dioxus::logger::tracing::{info, error};
use crate::atoms::tasks::state::{state_create_task, state_upload_task_file};
use crate::Route;
use futures::stream::{self, StreamExt};
use dioxus::router::Navigator;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, PartialEq)]
pub struct PendingUpload {
    pub file: web_sys::File,
    pub name: String,
}

pub async fn handle_create_task(
    block_id: String,
    block_name: String,
    block_type: String,
    task_name: String,
    uploads: Vec<PendingUpload>,
    nav: Navigator,
    mut is_submitting: Signal<bool>,
    mut upload_current: Signal<usize>,
    mut upload_total: Signal<usize>,
) {
    is_submitting.set(true);
    let total_uploads = uploads.len();
    upload_total.set(total_uploads);
    upload_current.set(0);

    match state_create_task(&block_id, task_name.clone()).await {
        Ok(task) => {
            info!("✅ Task created: {}", task.task_name);

             if !uploads.is_empty() {
                info!("Uploading {} images for task {}", uploads.len(), task.task_id);
                
                let uploads_iter = uploads.into_iter().map(|upload| {
                    let b_id = block_id.clone();
                    let t_id = task.task_id.clone();
                    async move {
                        state_upload_task_file(&b_id, &t_id, upload.file).await
                    }
                });
                
                let mut stream = stream::iter(uploads_iter).buffer_unordered(3); // Limit concurrency
                let mut success_count = 0;
                let mut fail_count = 0;

                
                while let Some(res) = stream.next().await {
                     match res {
                        Ok(_) => {
                            success_count +=1;
                            upload_current.set(success_count + fail_count);
                            info!("✅ Image {}/{} uploaded", success_count + fail_count, total_uploads);
                        }
                        Err(e) => {
                            fail_count +=1;
                            upload_current.set(success_count + fail_count);
                            error!("❌ Failed to upload image: {}", e);
                        }
                    }
                }
                if fail_count > 0 {
                    crate::shell::status::show_error(&format!("Uploaded {} images, {} failed", success_count, fail_count));
                } else {
                    crate::shell::status::show_success(&format!("✅ Uploaded {} images", success_count));
                }
            }

            nav.push(Route::TasksListPage { block_id, block_name, block_type });
        },
        Err(e) => {
            error!("❌ Failed to create task: {}", e);
            crate::shell::status::show_error(&format!("Failed to create task: {}", e));
        }
    }
    
    // is_submitting.set(false);
}
