

use dioxus::prelude::*;
use dioxus::logger::tracing::{info, error};
use crate::tasks::state::{state_create_task, state_upload_task_file};
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
    project_id: String,
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
    let total_steps = total_uploads + 1;
    upload_total.set(total_uploads);
    upload_current.set(0);
    let operation_started = web_time::Instant::now();
    crate::core::progress::show_progress(
        &format!("Creating task 0/{}", total_steps),
        0,
        total_steps,
        0,
    );

    match state_create_task(&project_id, &block_id, task_name.clone()).await {
        Ok(task) => {
            info!("✅ Task created: {}", task.task_name);
            crate::core::progress::show_progress(
                &format!("Task created 1/{}", total_steps),
                1,
                total_steps,
                operation_started.elapsed().as_secs(),
            );

             if !uploads.is_empty() {
                info!("Uploading {} images for task {}", uploads.len(), task.task_id);
                
                let uploads_iter = uploads.into_iter().map(|upload| {
                    let p_id = project_id.clone();
                    let b_id = block_id.clone();
                    let t_id = task.task_id.clone();
                    async move {
                        state_upload_task_file(&p_id, &b_id, &t_id, upload.file).await
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
                    let completed_uploads = success_count + fail_count;
                    crate::core::progress::show_progress(
                        &format!(
                            "Uploading images {}/{}{}",
                            completed_uploads,
                            total_uploads,
                            if fail_count > 0 {
                                format!(" · {} failed", fail_count)
                            } else {
                                String::new()
                            }
                        ),
                        1 + completed_uploads,
                        total_steps,
                        operation_started.elapsed().as_secs(),
                    );
                }
                if fail_count > 0 {
                    crate::core::progress::show_error_persistent(&format!("Uploaded {} images, {} failed", success_count, fail_count));
                } else {
                    crate::core::progress::show_success(&format!("✅ Uploaded {} images", success_count));
                }
            }

            nav.push(Route::TasksListPage { project_id: project_id.clone(), block_id, block_name, block_type });
        },
        Err(e) => {
            error!("❌ Failed to create task: {}", e);
            crate::core::progress::show_error_persistent(&format!("Failed to create task: {}", e));
        }
    }
    
    // is_submitting.set(false);
}
