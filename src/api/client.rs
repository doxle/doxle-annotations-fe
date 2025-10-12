use serde::{Deserialize, Serialize};
use super::cognito;

const API_BASE_URL: &str = "https://qxh6o5rw2b.execute-api.ap-southeast-2.amazonaws.com";

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub role: String,
}

// Get current user profile (with lazy initialization)
pub async fn get_or_create_user(email: &str) -> Result<User, String> {
    web_sys::console::log_1(&"👤 Getting or creating user profile...".into());
    let token = cognito::get_token().ok_or("No auth token found")?;

    // Try to get existing user
    web_sys::console::log_1(&"🔍 Checking if user exists...".into());
    match get_user(&token).await {
        Ok(user) => {
            web_sys::console::log_1(&"✅ User profile found!".into());
            Ok(user)
        },
        Err(e) => {
            web_sys::console::log_1(&format!("⚠️ User not found: {}. Creating profile...", e).into());
            // User doesn't exist, create profile
            create_user(&token, email).await
        }
    }
}

// GET /users/me
async fn get_user(token: &str) -> Result<User, String> {
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

// POST /users
async fn create_user(token: &str, email: &str) -> Result<User, String> {
    web_sys::console::log_1(&"➕ Creating new user profile...".into());
    let client = reqwest::Client::new();
    
    let request_body = CreateUserRequest {
        email: email.to_string(),
        role: "annotator".to_string(), // Default role
    };

    let response = client
        .post(format!("{}/users", API_BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    web_sys::console::log_1(&format!("📥 Create user response: {}", response.status()).into());

    if response.status().is_success() {
        web_sys::console::log_1(&"✅ User profile created successfully!".into());
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse user: {}", e))
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        web_sys::console::error_1(&format!("❌ Failed to create user: {}", error_text).into());
        Err(format!("Failed to create user: {}", error_text))
    }
}
