use serde::{Deserialize, Serialize};

// API Gateway endpoint
const API_BASE_URL: &str = "https://api.doxle.ai";

#[derive(Debug, Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct SignupRequest {
    email: String,
    password: String,
    invite_code: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthResult {
    pub access_token: String,
    pub id_token: String,
    pub refresh_token: String,
    pub expires_in: i32,
}

pub async fn authenticate(email: &str, password: &str) -> Result<AuthResult, String> {
    tracing::info!("🔐 Starting authentication for: {}", email);

    let request = LoginRequest {
        email: email.to_string(),
        password: password.to_string(),
    };

    let endpoint = format!("{}/login", API_BASE_URL);
    tracing::info!("📡 Calling auth endpoint: {}", endpoint);

    let client = reqwest::Client::new();

    let response = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("Network error: {}", e);
            tracing::error!("{}", error_msg);
            error_msg
        })?;

    tracing::info!("📥 Response status: {}", response.status());

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        
        // Try to parse the error response JSON
        let error_msg = if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
            error_response.message
        } else {
            error_text
        };
        
        tracing::error!("Authentication failed: {}", error_msg);
        return Err(error_msg);
    }

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    tracing::info!("📄 Raw response: {}", response_text);

    let auth_result: AuthResult = serde_json::from_str(&response_text).map_err(|e| {
        let error_msg = format!("Failed to parse response: {}", e);
        tracing::error!("{}", error_msg);
        error_msg
    })?;

    tracing::info!("✅ Authentication successful!");
    Ok(auth_result)
}

// Store token in browser local storage
pub fn store_token(id_token: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("doxle_auth_token", id_token);
        }
    }
}

// Get token from browser local storage
pub fn get_token() -> Option<String> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item("doxle_auth_token")
        .ok()?
}

// Clear token from browser local storage
pub fn clear_token() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("doxle_auth_token");
        }
    }
}

// Signup new user with Cognito
pub async fn signup(email: &str, password: &str, invite_code: &str) -> Result<(), String> {
    tracing::info!("📝 Starting signup for: {}", email);

    let request = SignupRequest {
        email: email.to_string(),
        password: password.to_string(),
        invite_code: invite_code.to_string(),
    };

    let endpoint = format!("{}/signup", API_BASE_URL);
    tracing::info!("📡 Calling signup endpoint: {}", endpoint);

    let client = reqwest::Client::new();

    let response = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("Network error: {}", e);
            tracing::error!("{}", error_msg);
            error_msg
        })?;

    tracing::info!("📥 Response status: {}", response.status());

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        
        // Try to parse the error response JSON
        let error_msg = if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
            error_response.message
        } else {
            error_text
        };
        
        tracing::error!("Signup failed: {}", error_msg);
        return Err(error_msg);
    }

    tracing::info!("✅ Signup successful!");
    Ok(())
}
