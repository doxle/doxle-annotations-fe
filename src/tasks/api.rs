use crate::core::client;

use serde::Serialize;
use crate::media::Attachment;
use crate::tasks::model::Task;

/// POST /projects/{pid}/blocks/{block_id}/tasks/{task_id}/images
pub async fn api_create_task_attachment(project_id:&str, block_id:&str, task_id:&str, image_id: String, image_name: String, image_url:String, file_size: u64)->Result<Attachment,String> {
    #[derive(Debug, Serialize)]
    struct CreateTaskImageRequest {
        image_id: String,
        image_name: String,
        url: String,
        file_size: u64,
    }

    let endpoint = format!("/projects/{}/blocks/{}/tasks/{}/images", project_id, block_id, task_id);
    let body = CreateTaskImageRequest { 
        image_id,
        image_name,
        url: image_url,
        file_size,
    };
    client::post::<CreateTaskImageRequest, Attachment>(&endpoint, &body).await
}



/// GET /projects/{pid}/blocks/{block_id}/tasks/{task_id}/images
pub async fn api_list_task_attachments(project_id: &str, block_id: &str, task_id: &str) -> Result<Vec<crate::media::Attachment>, String> {
    let endpoint = format!("/projects/{}/blocks/{}/tasks/{}/images", project_id, block_id, task_id);
    client::get::<Vec<crate::media::Attachment>>(&endpoint).await
}

/// GET /projects/{pid}/blocks/{block_id}/tasks
pub async fn api_list_tasks(project_id: &str, block_id:&str) -> Result<Vec<Task>, String> {
	let endpoint = format!("/projects/{}/blocks/{}/tasks", project_id, block_id);
	client::get::<Vec<Task>>(&endpoint).await
}

///POST /projects/{pid}/blocks/{block_id}/tasks
pub async fn api_create_task(project_id: &str, block_id:&str, task_name:String)->Result<Task,String>{
	
	#[derive(Debug, Serialize)]
	struct CreateTaskRequest{
		task_name:String,
	}

	let endpoint = format!("/projects/{}/blocks/{}/tasks", project_id, block_id);
	let body = CreateTaskRequest {task_name};
	client::post::<CreateTaskRequest, Task>(&endpoint, &body).await
}

/// PATCH /projects/{pid}/blocks/{block_id}/tasks/{task_id}
pub async fn api_update_task(
	project_id: &str,
	block_id: &str,
	task_id: &str,
	name: Option<String>,
	state: Option<super::model::TaskState>,
	assignee: Option<String>,
	reviewer: Option<String>,
) -> Result<Task, String> {

	#[derive(Debug, Serialize)]
	struct UpdateTaskRequest {
		#[serde(skip_serializing_if = "Option::is_none")]
		task_name: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		task_state: Option<super::model::TaskState>,
		#[serde(skip_serializing_if = "Option::is_none")]
		assignee: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		checked_by: Option<String>,
	}

	let endpoint = format!("/projects/{}/blocks/{}/tasks/{}", project_id, block_id, task_id);
	let body = UpdateTaskRequest { task_name: name, task_state: state, assignee, checked_by: reviewer };
	client::patch::<UpdateTaskRequest, Task>(&endpoint, &body).await
}

/// Convenience: assign a task to a user
pub async fn api_assign_task(project_id: &str, block_id: &str, task_id: &str, user_id: &str) -> Result<Task, String> {
	api_update_task(project_id, block_id, task_id, None, None, Some(user_id.to_string()), None).await
}

/// Convenience: set reviewer for a task
pub async fn api_set_reviewer(project_id: &str, block_id: &str, task_id: &str, user_id: &str) -> Result<Task, String> {
	api_update_task(project_id, block_id, task_id, None, None, None, Some(user_id.to_string())).await
}

/// DELETE /projects/{pid}/blocks/{block_id}/tasks/{task_id}
pub async fn api_delete_task(project_id: &str, block_id: &str, task_id: &str) -> Result<(), String> {
    let endpoint = format!("/projects/{}/blocks/{}/tasks/{}", project_id, block_id, task_id);
    client::delete(&endpoint).await
}
