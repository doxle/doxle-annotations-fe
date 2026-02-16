use crate::atoms::tasks::api;
use crate::atoms::tasks::model::Task;
use crate::atoms::media::api::upload_image_for_task;
use dioxus::prelude::*;


pub static TASKS:GlobalSignal<Vec<Task>> = Signal::global(|| Vec::new());
pub static TASKS_LOADING:GlobalSignal<bool> = Signal::global(|| false);
pub static TASKS_ERROR:GlobalSignal<Option<String>> = Signal::global(|| None);
pub static CURRENT_TASK_ID:GlobalSignal<Option<String>> = Signal::global(|| None);
pub static DELETE_CURRENT: GlobalSignal<usize> = Signal::global(|| 0);
pub static DELETE_TOTAL: GlobalSignal<usize> = Signal::global(|| 0);

pub async fn state_load_tasks(block_id:&str){
 	*TASKS_LOADING.write() = true;
 	*TASKS_ERROR.write() = None;

	match api::api_list_tasks(block_id).await {
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

 pub async fn state_load_tasks_silent(block_id:&str){
    match api::api_list_tasks(block_id).await {
        Ok(tasks_list) => { 
            *TASKS.write() = tasks_list;
        }
        Err(e)=>{
            tracing::error!("❌ Error loading tasks: {}", e);
        }
     }
 }

pub async fn state_create_task(block_id:&str, name:String) -> Result<Task, String> {
 	match api::api_create_task(block_id, name).await {
 		Ok(task) => {
 			TASKS.write().push(task.clone());
 			tracing::info!("✅ Task created: {}", task.task_name);
 			Ok(task)
 		}
 		Err(e) => {
 			tracing::error!("❌ Failed to create task: {}", e);
 			Err(e)
 		}
 	}
 }

pub async fn state_update_task_state(block_id:&str, task_id:&str, state:super::model::TaskState) {
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
 	if let Err(e) = api::api_update_task(block_id, task_id, None, Some(state), None, None).await {
 	 	tracing::error!("❌ Failed to update task state: {}", e);
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

pub async fn state_upload_task_file(block_id:&str, task_id:&str, file:web_sys::File)-> Result<String, String>{
    upload_image_for_task(block_id, task_id, file).await
}

pub async fn state_delete_task(block_id: &str, task_id: &str) {
    // Get images from the task before deleting
    let images: Vec<(String, String)> = TASKS.read()
        .iter()
        .find(|t| t.task_id == task_id)
        .map(|t| t.images.iter().map(|img| (img.image_id.clone(), block_id.to_string())).collect())
        .unwrap_or_default();

    let total = images.len() + 1; // images + task record
    *DELETE_CURRENT.write() = 0;
    *DELETE_TOTAL.write() = total;

    // Delete images one by one with progress
    for (i, (image_id, bid)) in images.iter().enumerate() {
        if let Err(e) = crate::atoms::media::api::api_delete_image(&bid, &image_id).await {
            tracing::error!("❌ Failed to delete image {}: {}", image_id, e);
        }
        *DELETE_CURRENT.write() = i + 1;
    }

    // Delete the task record
    match api::api_delete_task(block_id, task_id).await {
        Ok(_) => {
            TASKS.write().retain(|t| t.task_id != task_id);
            tracing::info!("✅ Task deleted: {}", task_id);
        }
        Err(e) => {
            tracing::error!("❌ Failed to delete task: {}", e);
        }
    }

    *DELETE_CURRENT.write() = total;
    *DELETE_CURRENT.write() = 0;
    *DELETE_TOTAL.write() = 0;
}

pub async fn state_rename_task(block_id: &str, task_id: &str, new_name: String) {
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
    if let Err(e) = api::api_update_task(block_id, task_id, Some(new_name), None, None, None).await {
        tracing::error!("❌ Failed to rename task: {}", e);
        // Rollback
        TASKS.write()[index].task_name = old_name;
    } else {
        tracing::info!("✅ Task renamed: {}", task_id);
    }
}
