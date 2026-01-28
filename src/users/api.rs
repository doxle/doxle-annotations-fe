use serde::{Deserialize, Serialize};

use crate::api::auth_api::get_access_token;

pub const API_BASE_URL: &str = "https://api.doxle.ai";

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    #[serde(default)]
    pub user_name: String,
    pub user_email: String,
    pub user_company: Option<String>,
    pub user_role: String,
    pub user_created_at: String,
    pub user_last_login: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateUserRequest {
    pub user_name: String,
    pub user_email: String,
    pub user_company: Option<String>,
    pub user_role: String,
}

// GET /users/me
pub async fn get_current_user() -> Result<User, String> {
    tracing::info!("👤 Getting user profile");
    crate::shell::client::get::<User>("/users/me").await
}

// POST /users
pub async fn create_user_profile(
    user_name: String,
    user_email: String,
    user_company: Option<String>,
    user_role: String,
) -> Result<User, String> {
    tracing::info!("➕ Creating new user profile: {}", user_email);
    let token = get_access_token().ok_or("No auth token found")?;

    let request_body = CreateUserRequest {
        user_name,
        user_email,
        user_company,
        user_role,
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
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        tracing::error!("❌ Failed to create user: {}", error_text);
        Err(format!("Failed to create user: {}", error_text))
    }
}
