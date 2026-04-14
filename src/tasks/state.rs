use crate::tasks::api;
use crate::tasks::model::Task;
use crate::media::api::upload_attachment_for_task;
use dioxus::prelude::*;


pub static TASKS:GlobalSignal<Vec<Task>> = Signal::global(|| Vec::new());
pub static TASKS_LOADING:GlobalSignal<bool> = Signal::global(|| false);
pub static TASKS_ERROR:GlobalSignal<Option<String>> = Signal::global(|| None);
pub static CURRENT_TASK_ID:GlobalSignal<Option<String>> = Signal::global(|| None);

pub async fn state_load_tasks(project_id: &str, block_id:&str){
 	*TASKS_LOADING.write() = true;
 	*TASKS_ERROR.write() = None;

	match api::api_list_tasks(project_id, block_id).await {
 		Ok(tasks_list) => {
 			let count = tasks_list.len();
 			*TASKS.write() = tasks_list;
 			tracing::info!("✅ Tasks loaded: {} items", count);
 		}
 		Err(e)=>{
 			let error_msg = e.clone();
 			*TASKS_ERROR.write() = Some(e);
 			tracing::error!("❌ Error loading tasks: {}", error_msg);
 		}
 	 }
 	  *TASKS_LOADING.write() = false;
 }

 pub async fn state_load_tasks_silent(project_id: &str, block_id:&str){
    match api::api_list_tasks(project_id, block_id).await {
        Ok(tasks_list) => { 
            *TASKS.write() = tasks_list;
        }
        Err(e)=>{
            tracing::error!("❌ Error loading tasks: {}", e);
        }
     }
 }

pub async fn state_create_task(project_id: &str, block_id:&str, name:String) -> Result<Task, String> {
 	match api::api_create_task(project_id, block_id, name).await {
 		Ok(task) => {
 			TASKS.write().push(task.clone());
 			tracing::info!("✅ Task created: {}", task.task_name);
 			crate::core::progress::show_success(&format!("Task '{}' created", task.task_name));
 			Ok(task)
 		}
 		Err(e) => {
 			tracing::error!("❌ Failed to create task: {}", e);
			crate::core::progress::show_error_persistent(&format!("Failed to create task: {}", e));
 			Err(e)
 		}
 	}
 }

/// Fetch full images for a task and update TASKS signal in-place
pub async fn state_load_task_images(project_id: &str, block_id: &str, task_id: &str) {
    match api::api_list_task_attachments(project_id, block_id, task_id).await {
        Ok(images) => {
            let count = images.len();
            let mut tasks = TASKS.write();
            if let Some(task) = tasks.iter_mut().find(|t| t.task_id == task_id) {
                task.images = images;
                task.image_count = count as u32;
            }
            tracing::info!("✅ Loaded {} images for task {}", count, task_id);
        }
        Err(e) => {
            tracing::error!("❌ Failed to load images for task {}: {}", task_id, e);
        }
    }
}

pub async fn state_update_task_state(project_id: &str, block_id:&str, task_id:&str, state:super::model::TaskState) {
 	 // Find task index and backup old state only
 	 let (index, old_state) = {
 	 	let tasks = TASKS.read();
 	 	match tasks.iter().position(|t| t.task_id == task_id) {
 	 		Some(idx) => (idx, tasks[idx].task_state.clone()),
 	 		None => return,
 	 	}
 	 };

 	 //Optimistic update
 	 TASKS.write()[index].task_state = state.clone();

 	 //API call
 	if let Err(e) = api::api_update_task(project_id, block_id, task_id, None, Some(state), None, None).await {
 	 	tracing::error!("❌ Failed to update task state: {}", e);
 	 	crate::core::progress::show_error_persistent(&format!("Failed to update task state: {}", e));
        // Rollback
        TASKS.write()[index].task_state = old_state;
 	 }


 }

pub fn state_set_current_task(task_id: Option<String>) {
    *CURRENT_TASK_ID.write() = task_id;
}

pub fn state_get_current_task_id() -> Option<String> {
    CURRENT_TASK_ID.read().clone()
}

pub async fn state_upload_task_file(project_id: &str, block_id:&str, task_id:&str, file:web_sys::File)-> Result<String, String>{
    upload_attachment_for_task(project_id, block_id, task_id, file).await
}

pub async fn state_delete_task(project_id: &str, block_id: &str, task_id: &str) {
    use crate::core::progress::{show_progress_danger, show_success, clear_status};

    // Get images with annotation counts from the task before deleting
    let images: Vec<(String, String, u32)> = TASKS.read()
        .iter()
        .find(|t| t.task_id == task_id)
        .map(|t| t.images.iter().map(|img| (
            img.image_id.clone(),
            block_id.to_string(),
            img.annotation_count,
        )).collect())
        .unwrap_or_default();

    let total = images.len() + 1; // images + task record
    let start = web_time::Instant::now();
    show_progress_danger(&format!("Deleting 0/{}", total), 0, total, 0);

    // Delete images one by one with progress
    for (i, (image_id, bid, ann_count)) in images.iter().enumerate() {
        show_progress_danger(
            &format!("Deleting img {}/{} ({} annotations)", i + 1, total, ann_count),
            i, total, start.elapsed().as_secs(),
        );
        if let Err(e) = crate::media::api::api_delete_attachment(project_id, &bid, &image_id).await {
            tracing::error!("❌ Failed to delete image {}: {}", image_id, e);
        }
        let done = i + 1;
        show_progress_danger(
            &format!("Deleting {}/{}", done, total),
            done, total, start.elapsed().as_secs(),
        );
    }

    // Delete the task record
    show_progress_danger(
        &format!("Deleting {}/{}", total, total),
        total, total, start.elapsed().as_secs(),
    );
    match api::api_delete_task(project_id, block_id, task_id).await {
        Ok(_) => {
            TASKS.write().retain(|t| t.task_id != task_id);
            tracing::info!("✅ Task deleted: {}", task_id);
            let elapsed = start.elapsed().as_secs();
            show_success(&format!("Deleted in {}s", elapsed));
        }
        Err(e) => {
            tracing::error!("❌ Failed to delete task: {}", e);
            crate::core::progress::show_error_persistent(&format!("Delete failed: {}", e));
        }
    }
}

pub async fn state_rename_task(project_id: &str, block_id: &str, task_id: &str, new_name: String) {
    // Find task index and backup old name
    let (index, old_name) = {
        let tasks = TASKS.read();
        match tasks.iter().position(|t| t.task_id == task_id) {
            Some(idx) => (idx, tasks[idx].task_name.clone()),
            None => return,
        }
    };

    // Optimistic update
    TASKS.write()[index].task_name = new_name.clone();

    // API call
    if let Err(e) = api::api_update_task(project_id, block_id, task_id, Some(new_name), None, None, None).await {
        tracing::error!("❌ Failed to rename task: {}", e);
        crate::core::progress::show_error_persistent(&format!("Failed to rename task: {}", e));
        // Rollback
        TASKS.write()[index].task_name = old_name;
    } else {
        tracing::info!("✅ Task renamed: {}", task_id);
    }
}
