use crate::atoms::tasks::api;
use crate::atoms::tasks::model::Task;
use crate::atoms::media::api::upload_image_for_task;
use dioxus::prelude::*;


pub static TASKS:GlobalSignal<Vec<Task>> = Signal::global(|| Vec::new());
pub static TASKS_LOADING:GlobalSignal<bool> = Signal::global(|| false);
pub static TASKS_ERROR:GlobalSignal<Option<String>> = Signal::global(|| None);
pub static CURRENT_TASK_ID:GlobalSignal<Option<String>> = Signal::global(|| None);	

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

pub async fn state_update_task_state(block_id:&str, task_id:&str, state:String) {
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
 	if let Err(e) = api::api_update_task(block_id, task_id, None, Some(state)).await {
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
