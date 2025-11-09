use super::client_api;
use serde::{Deserialize, Serialize};

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
    client_api::get::<Vec<Block>>(&endpoint).await
}

// POST /projects/{id}/blocks - create block
pub async fn create_block(project_id: &str, name: String) -> Result<Block, String> {
    let endpoint = format!("/projects/{}/blocks", project_id);
    let request = CreateBlockRequest { name };
    client_api::post::<CreateBlockRequest, Block>(&endpoint, &request).await
}

// PATCH /projects/{id}/blocks/{id} - rename block
pub async fn rename_block(
    project_id: &str,
    block_id: &str,
    new_name: String,
) -> Result<Block, String> {
    let endpoint = format!("/projects/{}/blocks/{}", project_id, block_id);
    let request = serde_json::json!({ "name": new_name });
    client_api::patch(&endpoint, &request).await
}

// DELETE /projects/{id}/blocks/{id} - delete block
pub async fn delete_block(project_id: &str, block_id: &str) -> Result<(), String> {
    let endpoint = format!("/projects/{}/blocks/{}", project_id, block_id);
    client_api::delete(&endpoint).await
}
