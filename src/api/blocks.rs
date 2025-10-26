use serde::{Deserialize, Serialize};
use super::client::API_BASE_URL;
use super::auth;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub block_id: String,
    pub project_id: String,
    pub name: String,
    pub state: String,
    pub locked: bool,
    pub assigned_to: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct CreateBlockRequest {
    pub name: String,
}

// GET /projects/{id}/blocks - list project blocks
pub async fn list_project_blocks(project_id: &str) -> Result<Vec<Block>, String> {
    let token = auth::get_token().ok_or("No auth token found")?;
    
    let endpoint = format!("{}/projects/{}/blocks", API_BASE_URL, project_id);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&endpoint)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        let blocks = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse blocks: {}", e))?;
        Ok(blocks)
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Failed to get blocks: {}", error_text))
    }
}

// POST /projects/{id}/blocks - create block
pub async fn create_block(project_id: &str, name: String) -> Result<Block, String> {
    let token = auth::get_token().ok_or("No auth token found")?;
    
    let endpoint = format!("{}/projects/{}/blocks", API_BASE_URL, project_id);
    let request = CreateBlockRequest { name };
    
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
        let block = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse block: {}", e))?;
        Ok(block)
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Failed to create block: {}", error_text))
    }
}

// DELETE /blocks/{id} - delete block
pub async fn delete_block(block_id: &str) -> Result<(), String> {
    let token = auth::get_token().ok_or("No auth token found")?;
    
    let endpoint = format!("{}/blocks/{}", API_BASE_URL, block_id);
    
    let client = reqwest::Client::new();
    let response = client
        .delete(&endpoint)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Failed to delete block: {}", error_text))
    }
}