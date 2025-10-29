use serde::{Deserialize, Serialize};
use super::client;

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

// POST /blocks/{id}/images - create image record
pub async fn create_image(block_id: &str, url: String, order: Option<i32>) -> Result<Image, String> {
    let endpoint = format!("/blocks/{}/images", block_id);
    let request = CreateImageRequest { url, order };
    client::post::<CreateImageRequest, Image>(&endpoint, &request).await
}

// GET /blocks/{id}/images - list block images
pub async fn list_block_images(block_id: &str) -> Result<Vec<Image>, String> {
    let endpoint = format!("/blocks/{}/images", block_id);
    client::get::<Vec<Image>>(&endpoint).await
}
