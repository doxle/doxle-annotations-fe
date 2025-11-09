use super::client_api;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Image {
    pub image_id: String,
    pub block_id: String,
    pub url: String,
    pub locked: bool,
    pub order: Option<i32>,
    pub uploaded_at: String,
}

#[derive(Debug, Serialize)]
pub struct CreateImageRequest {
    pub url: String,
    pub order: Option<i32>,
}

// POST /projects/{pid}/blocks/{id}/images - create image record
pub async fn create_image(
    project_id: &str,
    block_id: &str,
    url: String,
    order: Option<i32>,
) -> Result<Image, String> {
    let endpoint = format!("/projects/{}/blocks/{}/images", project_id, block_id);
    let request = CreateImageRequest { url, order };
    client_api::post::<CreateImageRequest, Image>(&endpoint, &request).await
}

// GET /projects/{pid}/blocks/{id}/images - list block images
pub async fn list_block_images(project_id: &str, block_id: &str) -> Result<Vec<Image>, String> {
    let endpoint = format!("/projects/{}/blocks/{}/images", project_id, block_id);
    client_api::get::<Vec<Image>>(&endpoint).await
}
