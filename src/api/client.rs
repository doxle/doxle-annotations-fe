use serde::{Deserialize, Serialize};
use super::auth;

// Shared API configuration
// Local: http://localhost:9000
// Production: https://qxh6o5rw2b.execute-api.ap-southeast-2.amazonaws.com
pub const API_BASE_URL: &str = "http://localhost:9000";

// Helper to get auth header
pub fn auth_header() -> Result<String, String> {
    auth::get_token()
        .map(|token| format!("Bearer {}", token))
        .ok_or_else(|| "No auth token found".to_string())
}

// Generic POST helper
pub async fn post<T: Serialize, R: for<'de> Deserialize<'de>>(
    endpoint: &str,
    body: &T,
) -> Result<R, String> {
    let token = auth::get_token().ok_or("No auth token found")?;
    let url = format!("{}{}", API_BASE_URL, endpoint);
    
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if response.status().is_success() {
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Request failed: {}", error_text))
    }
}

// Generic PATCH helper
pub async fn patch<T: Serialize, R: for<'de> Deserialize<'de>>(
    endpoint: &str,
    body: &T,
) -> Result<R, String> {
    let token = auth::get_token().ok_or("No auth token found")?;
    let url = format!("{}{}", API_BASE_URL, endpoint);
    
    let client = reqwest::Client::new();
    let response = client
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if response.status().is_success() {
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Request failed: {}", error_text))
    }
}

// Generic DELETE helper
pub async fn delete(endpoint: &str) -> Result<(), String> {
    let token = auth::get_token().ok_or("No auth token found")?;
    let url = format!("{}{}", API_BASE_URL, endpoint);
    
    let client = reqwest::Client::new();
    let response = client
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if response.status().is_success() || response.status() == reqwest::StatusCode::NO_CONTENT {
        Ok(())
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Delete failed: {}", error_text))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub name: String,
    pub email: String,
    pub company: Option<String>,
    pub role: String,
    pub created_at: String,
    pub last_login: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub company: Option<String>,
    pub role: String,
}

// Get current user profile (no auto-creation)
pub async fn get_current_user() -> Result<User, String> {
    tracing::info!("👤 Getting user profile");
    let token = auth::get_token().ok_or("No auth token found")?;
    get_user(&token).await
}

// Create user profile explicitly
pub async fn create_user_profile(name: String, email: String, company: Option<String>, role: String) -> Result<User, String> {
    tracing::info!("➕ Creating new user profile: {}", email);
    let token = auth::get_token().ok_or("No auth token found")?;
    
    let request_body = CreateUserRequest {
        name,
        email,
        company,
        role,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/users", API_BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        tracing::info!("✅ User profile created successfully");
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse user: {}", e))
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        tracing::error!("❌ Failed to create user: {}", error_text);
        Err(format!("Failed to create user: {}", error_text))
    }
}

// GET /users/me
pub async fn get_user(token: &str) -> Result<User, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/users/me", API_BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse user: {}", e))
    } else {
        Err(format!("Failed to get user: {}", response.status()))
    }
}

