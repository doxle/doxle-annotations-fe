use crate::shell::client;

use serde::Serialize;
use crate::atoms::media::Image;
use crate::atoms::tasks::model::Task;

/// POST /blocks/{block_id}/tasks/{task_id}/images
pub async fn api_create_task_image(block_id:&str, task_id:&str, image_id: String, image_name: String, image_url:String)->Result<Image,String> {
    #[derive(Debug, Serialize)]
    struct CreateTaskImageRequest {
        image_id: String,
        image_name: String,
        url: String,
    }

    let endpoint = format!("/blocks/{}/tasks/{}/images", block_id, task_id);
    let body = CreateTaskImageRequest { 
        image_id,
        image_name,
        url: image_url,
    };
    client::post::<CreateTaskImageRequest, Image>(&endpoint, &body).await
}



/// GET /blocks/block_id/tasks
pub async fn api_list_tasks(block_id:&str) -> Result<Vec<Task>, String> {
	let endpoint = format!("/blocks/{}/tasks", block_id);
	client::get::<Vec<Task>>(&endpoint).await
}

///POST /blocks/{block_id}/tasks
pub async fn api_create_task(block_id:&str, task_name:String)->Result<Task,String>{
	
	#[derive(Debug, Serialize)]
	struct CreateTaskRequest{
		task_name:String,
	}

	let endpoint = format!("/blocks/{}/tasks", block_id);
	let body = CreateTaskRequest {task_name};
	client::post::<CreateTaskRequest, Task>(&endpoint, &body).await
}

/// PATCH /blocks/{block_id}/tasks/{task_id}
pub async fn api_update_task(block_id:&str, task_id:&str, name:Option<String>, state:Option<String>) -> Result <Task,String>{

	#[derive(Debug, Serialize)]
	struct UpdateTaskRequest{
		task_name:Option<String>,
		task_state:Option<String>,
	}

	let endpoint = format!("/blocks/{}/tasks/{}", block_id, task_id);
	let body = UpdateTaskRequest {task_name:name, task_state:state};
	client::patch::<UpdateTaskRequest, Task>(&endpoint, &body).await

}

/// DELETE /blocks/{block_id}/tasks/{task_id}
pub async fn api_delete_task(block_id: &str, task_id: &str) -> Result<(), String> {
    let endpoint = format!("/blocks/{}/tasks/{}", block_id, task_id);
    client::delete(&endpoint).await
}
