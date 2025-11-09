use serde::{Deserialize, Serialize};
use super::client_api::{API_BASE_URL, auth_header};

// Re-export shared shapes for convenience
pub use crate::shared::shapes::{Point, Geometry};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Annotation {
    pub annotation_id: String,
    pub image_id: String,
    pub class_id: String,
    pub geometry: Geometry,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateAnnotationRequest {
    pub class_id: String,
    pub geometry: Geometry,
}

#[derive(Debug, Serialize)]
pub struct BatchCreateRequest {
    pub annotations: Vec<CreateAnnotationRequest>,
}

/// List all annotations for an image
pub async fn list_annotations(image_id: &str) -> Result<Vec<Annotation>, String> {
    let auth = auth_header()?;
    let url = format!("{}/images/{}/annotations", API_BASE_URL, image_id);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", &auth)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to list annotations: {}", response.status()));
    }
    
    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse annotations: {}", e))
}

/// Create a single annotation
pub async fn create_annotation(
    image_id: &str,
    project_id: &str,
    request: CreateAnnotationRequest,
) -> Result<Annotation, String> {
    let auth = auth_header()?;
    let url = format!(
        "{}/images/{}/annotations?project_id={}",
        API_BASE_URL, image_id, project_id
    );
    
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", &auth)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to create annotation: {}", response.status()));
    }
    
    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse annotation: {}", e))
}

/// Batch create annotations (more efficient)
pub async fn batch_create_annotations(
    image_id: &str,
    project_id: &str,
    request: BatchCreateRequest,
) -> Result<Vec<Annotation>, String> {
    let auth = auth_header()?;
    let url = format!(
        "{}/images/{}/annotations/batch?project_id={}",
        API_BASE_URL, image_id, project_id
    );
    
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", &auth)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to batch create annotations: {}", response.status()));
    }
    
    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse annotations: {}", e))
}

/// Delete an annotation
pub async fn delete_annotation(
    image_id: &str,
    annotation_id: &str,
    project_id: &str,
) -> Result<(), String> {
    let auth = auth_header()?;
    let url = format!(
        "{}/images/{}/annotations/{}?project_id={}",
        API_BASE_URL, image_id, annotation_id, project_id
    );
    
    let client = reqwest::Client::new();
    let response = client
        .delete(&url)
        .header("Authorization", &auth)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to delete annotation: {}", response.status()));
    }
    
    Ok(())
}
