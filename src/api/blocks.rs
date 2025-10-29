use serde::{Deserialize, Serialize};
use super::client;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
    let endpoint = format!("/projects/{}/blocks", project_id);
    client::get::<Vec<Block>>(&endpoint).await
}

// POST /projects/{id}/blocks - create block
pub async fn create_block(project_id: &str, name: String) -> Result<Block, String> {
    let endpoint = format!("/projects/{}/blocks", project_id);
    let request = CreateBlockRequest { name };
    client::post::<CreateBlockRequest, Block>(&endpoint, &request).await
}

// DELETE /blocks/{id} - delete block
pub async fn delete_block(block_id: &str) -> Result<(), String> {
    let endpoint = format!("/blocks/{}", block_id);
    client::delete(&endpoint).await
}
