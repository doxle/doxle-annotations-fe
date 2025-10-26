use serde::{Deserialize, Serialize};
use super::client::API_BASE_URL;
use super::auth;

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
    let token = auth::get_token().ok_or("No auth token found")?;
    
    let endpoint = format!("{}/blocks/{}/images", API_BASE_URL, block_id);
    let request = CreateImageRequest { url, order };
    
    let client = reqwest::Client::new();
    let response = client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        let image = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse image: {}", e))?;
        Ok(image)
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Failed to create image: {}", error_text))
    }
}

// GET /blocks/{id}/images - list block images
pub async fn list_block_images(block_id: &str) -> Result<Vec<Image>, String> {
    let token = auth::get_token().ok_or("No auth token found")?;
    
    let endpoint = format!("{}/blocks/{}/images", API_BASE_URL, block_id);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&endpoint)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        let images = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse images: {}", e))?;
        Ok(images)
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Failed to get images: {}", error_text))
    }
}
