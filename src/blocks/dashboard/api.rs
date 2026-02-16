use crate::shell::client;
use serde::{Deserialize, Serialize};


// Predefined color palette for labels (40 colors)
// Maximally spaced colors - each adjacent pair is visually distinct
// pub const LABEL_COLORS: [&str; 20] = [
//     "#EF4444", "#3B82F6", "#22C55E", "#F59E0B", "#8B5CF6",
//     "#06B6D4", "#EC4899", "#84CC16", "#F97316", "#6366F1",
//     "#14B8A6", "#EAB308", "#A855F7", "#0EA5E9", "#F43F5E",
//     "#10B981", "#D946EF", "#64748B", "#BE123C", "#047857",
// ];

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Label {
    #[serde(default)]
    pub label_id: String,
    pub name: String,
    pub color: String,
}


#[derive(Debug, Serialize)]
struct UpdateLabelRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
}



// PATCH /projects/{project_id}/labels/{label_id}
pub async fn api_rename_label(project_id: &str, label_id: &str, new_name: String) -> Result<Label, String> {
    let endpoint = format!("/projects/{}/labels/{}", project_id, label_id);
    let body = UpdateLabelRequest { name: Some(new_name), color: None };
    client::patch(&endpoint, &body).await
}

// PATCH /blocks/{block_id}/labels/{label_id}
pub async fn api_update_label_color(block_id: &str, label_id: &str, color: String) -> Result<BlockLabel, String> {
    let endpoint = format!("/blocks/{}/labels/{}", block_id, label_id);
    let body = serde_json::json!({ "label_color": color });
    client::patch(&endpoint, &body).await
}

// DELETE /projects/{project_id}/labels/{label_id}
pub async fn api_delete_label(project_id: &str, label_id: &str) -> Result<(), String> {
    let endpoint = format!("/projects/{}/labels/{}", project_id, label_id);
    client::delete(&endpoint).await
}

// Old project-based Block types removed - now using block-centric architecture


// ======================
// Block-Centric Architecture (no projects)
// ======================

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BlockType {
    Annotation,
    File,
    Building,
}

impl std::fmt::Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl BlockType {
    pub fn as_str(&self) -> &str {
        match self {
            BlockType::Annotation => "annotation",
            BlockType::File => "file",
            BlockType::Building => "building",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            BlockType::Annotation => "Annotation",
            BlockType::File => "File",
            BlockType::Building => "Building",
        }
    }

    pub fn from_str(s: &str) -> Option<BlockType> {
        match s {
            "annotation" => Some(BlockType::Annotation),
            "file" => Some(BlockType::File),
            "building" => Some(BlockType::Building),
            _ => None,
        }
    }
}

#[derive(Debug,  Serialize, Deserialize, Clone, PartialEq)]
pub struct Block {
    pub block_id: String,
    pub block_name: String,
    pub block_type: BlockType,
    pub block_company:Option<String>,
    pub block_state: String,
    pub block_locked: bool,
    pub image_count: u32,
    pub approved_image_count: u32,
    pub annotation_count: u32,
    #[serde(default)]
    pub labels: Vec<BlockLabel>,
    pub block_created_at: String,
    #[serde(default)]
    pub block_updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct CreateBlockRequest {
    pub block_name: String,
    pub block_type: BlockType,
    pub block_company: Option<String>,
}


/// GET /blocks -> list all blocks
pub async fn api_list_blocks() -> Result<Vec<Block>, String> {
    client::get::<Vec<Block>>("/blocks").await
}

/// POST /blocks -> create a new block
pub async fn api_create_block(block_name: String, block_type: BlockType, block_company: Option<String>) -> Result<Block, String> {
    let body = CreateBlockRequest { block_name, block_type, block_company };
    client::post::<CreateBlockRequest, Block>("/blocks", &body).await
}

/// PATCH /blocks/{id} - rename
pub async fn api_rename_block(block_id: &str, new_name: String) -> Result<Block, String> {
    let endpoint = format!("/blocks/{}", block_id);
    let body = serde_json::json!({ "block_name": new_name });
    client::patch(&endpoint, &body).await
}

/// DELETE /blocks/{id}
pub async fn api_delete_block(block_id: &str) -> Result<(), String> {
    let endpoint = format!("/blocks/{}", block_id);
    client::delete(&endpoint).await
}


// ======================
// Labels for a block (/blocks/{block_id}/labels)
// ======================

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BlockLabel {
    pub label_id: String,
    pub block_id: String,
    pub label_name: String,
    pub label_color: String,
    pub label_properties: Option<serde_json::Value>,
    pub label_count: u32
}

#[derive(Debug, Serialize)]
pub struct CreateLabelRequest {
    pub label_name: String,
    pub label_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_properties: Option<serde_json::Value>,
}


/// GET /blocks/{block_id}/labels
pub async fn api_get_labels(block_id:&str) -> Result<Vec<BlockLabel>, String> {
    let endpoint = format!("/blocks/{}/labels", block_id);
    client::get::<Vec<BlockLabel>>(&endpoint).await
}


/// POST /blocks/{block_id}/labels 
pub async fn api_create_label(block_id: &str, label_name: String, label_color:String) -> Result<BlockLabel, String> {
    let endpoint = format!("/blocks/{}/labels", block_id);
    let body = CreateLabelRequest {
        label_name,
        label_color: label_color,
        label_properties: None
    };
    client::post::<CreateLabelRequest, BlockLabel>(&endpoint, &body).await
}

/// PATCH /blocks/{block_id}/labels/{label_id} - update label_properties
pub async fn api_update_label_properties(block_id: &str, label_id: &str, properties: serde_json::Value) -> Result<BlockLabel, String> {
    let endpoint = format!("/blocks/{}/labels/{}", block_id, label_id);
    let body = serde_json::json!({ "label_properties": properties });
    client::patch(&endpoint, &body).await
}
