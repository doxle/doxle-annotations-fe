use serde::{Deserialize, Serialize};
use super::client_api::{API_BASE_URL, auth_header};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Class {
    pub class_id: String,
    pub project_id: String,
    pub name: String,
    pub color: Option<String>,
    pub properties: Option<serde_json::Value>,
    pub count: u32,
}

#[derive(Debug, Serialize)]
pub struct CreateClassRequest {
    pub name: String,
    pub color: Option<String>,
    pub properties: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct UpdateClassRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub properties: Option<serde_json::Value>,
}

/// List all classes for a project
pub async fn list_classes(project_id: &str) -> Result<Vec<Class>, String> {
    let auth = auth_header()?;
    let url = format!("{}/projects/{}/classes", API_BASE_URL, project_id);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", &auth)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to list classes: {}", response.status()));
    }
    
    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse classes: {}", e))
}

/// Create a new class
pub async fn create_class(project_id: &str, request: CreateClassRequest) -> Result<Class, String> {
    let auth = auth_header()?;
    let url = format!("{}/projects/{}/classes", API_BASE_URL, project_id);
    
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", &auth)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to create class: {}", response.status()));
    }
    
    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse class: {}", e))
}

/// Update a class
pub async fn update_class(project_id: &str, class_id: &str, request: UpdateClassRequest) -> Result<Class, String> {
    let auth = auth_header()?;
    let url = format!("{}/projects/{}/classes/{}", API_BASE_URL, project_id, class_id);
    
    let client = reqwest::Client::new();
    let response = client
        .patch(&url)
        .header("Authorization", &auth)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to update class: {}", response.status()));
    }
    
    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse class: {}", e))
}

/// Delete a class
pub async fn delete_class(project_id: &str, class_id: &str) -> Result<(), String> {
    let auth = auth_header()?;
    let url = format!("{}/projects/{}/classes/{}", API_BASE_URL, project_id, class_id);
    
    let client = reqwest::Client::new();
    let response = client
        .delete(&url)
        .header("Authorization", &auth)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to delete class: {}", response.status()));
    }
    
    Ok(())
}
