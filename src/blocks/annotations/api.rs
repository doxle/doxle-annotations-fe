
use serde::{Serialize, Deserialize};
use crate::shell::client;
use crate::atoms::svg_canvas::{Geometry};


// ============================================
// API Response Types
// ============================================

#[derive(Debug, Clone, Deserialize)]
pub struct ApiAnnotation {
    pub annotation_id: String,
    pub image_id: String,
    pub label_id: String,
    pub geometry: Geometry,
}

// ============================================
// API Request Types
// ============================================

#[derive(Debug, Clone, Serialize)]
pub struct CreateAnnotationPayload {
    label_id:String,
    geometry:Geometry,
}

#[derive(Debug, Serialize)]
pub struct UpdateAnnotationPayload {
    label_id:String
}

#[derive(Debug, Serialize)]
pub struct UpdateGeometryPayload {
    geometry:Geometry,
}

// ============================================
// API Calls
// ============================================

pub async fn api_list_annotations(image_id: &str) -> Result<Vec<ApiAnnotation>, String> {
    let endpoint = format!("/images/{}/annotations", image_id);
    client::get(&endpoint).await
}

pub async fn api_create_annotation(block_id:&str, image_id:&str, label_id:&str, geometry:Geometry) -> Result<String, String> {
    let endpoint = format!("/images/{}/annotations?block_id={}", image_id, block_id);
    let payload = CreateAnnotationPayload {
        label_id:label_id.to_string(), 
        geometry:geometry,
    };

    let response: Result<ApiAnnotation, String> = client::post(&endpoint, &payload).await;

    match response {
        Ok(ann) => Ok(ann.annotation_id),
        Err(e) => {
            tracing::error!("Failed to create annotation: {}", e);
            Err(e)
        }
    }
}

pub async fn api_update_annotation_label(block_id:&str, image_id:&str, annotation_id:&str, label_id:&str) -> Result<(), String>{
    let endpoint = format!("/images/{}/annotations/{}?block_id={}", image_id, annotation_id, block_id);
    let payload = UpdateAnnotationPayload {label_id:label_id.to_string()};
    client::patch_no_response(&endpoint, &payload).await
}

pub async fn api_delete_annotation(block_id:&str, image_id:&str, annotation_id:&str)->Result<(),String>{
    let endpoint = format!("/images/{}/annotations/{}?block_id={}", image_id, annotation_id, block_id);
    client::delete(&endpoint).await
}

pub async fn api_update_geometry(block_id:&str, image_id:&str, annotation_id:&str, geometry:Geometry)-> Result<(), String>{
    let endpoint = format!("/images/{}/annotations/{}?block_id={}", image_id, annotation_id, block_id);
    let payload = UpdateGeometryPayload {geometry};
    client::patch_no_response(&endpoint, &payload).await
}

// ============================================
// Comment / Thread API Types
// ============================================

use super::models::{ApiCommentThread, ApiComment};

#[derive(Debug, Serialize)]
struct CreateThreadRequest {
    metadata: Option<String>,
    text: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateCommentRequest {
    text: String,
}

#[derive(Debug, Serialize)]
struct UpdateThreadRequest {
    resolved: Option<bool>,
}

// ============================================
// Comment / Thread API Calls
// ============================================

/// List all threads (with comments) for a parent resource
pub async fn api_list_threads(parent_id: &str) -> Result<Vec<ApiCommentThread>, String> {
    let endpoint = format!("/comments/{}/threads", parent_id);
    client::get(&endpoint).await
}

/// Create a new thread under a parent resource
pub async fn api_create_thread(
    parent_id: &str,
    metadata: Option<String>,
    text: Option<String>,
) -> Result<ApiCommentThread, String> {
    let endpoint = format!("/comments/{}/threads", parent_id);
    let payload = CreateThreadRequest { metadata, text };
    client::post(&endpoint, &payload).await
}

/// Add a comment to an existing thread
pub async fn api_add_comment(
    parent_id: &str,
    thread_id: &str,
    text: &str,
) -> Result<ApiComment, String> {
    let endpoint = format!("/comments/{}/threads/{}/comments", parent_id, thread_id);
    let payload = CreateCommentRequest { text: text.to_string() };
    client::post(&endpoint, &payload).await
}

/// Delete a thread and all its comments
pub async fn api_delete_thread(parent_id: &str, thread_id: &str) -> Result<(), String> {
    let endpoint = format!("/comments/{}/threads/{}", parent_id, thread_id);
    client::delete(&endpoint).await
}

/// Resolve or unresolve a thread
pub async fn api_resolve_thread(parent_id: &str, thread_id: &str, resolved: bool) -> Result<(), String> {
    let endpoint = format!("/comments/{}/threads/{}", parent_id, thread_id);
    let payload = UpdateThreadRequest { resolved: Some(resolved) };
    client::patch_no_response(&endpoint, &payload).await
}
