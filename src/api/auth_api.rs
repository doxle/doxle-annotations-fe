use serde::{Deserialize, Serialize};
use serde_json::json;

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
        let error_msg =
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
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

// Store refresh token in browser local storage
pub fn store_refresh_token(refresh_token: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("doxle_refresh_token", refresh_token);
        }
    }
}

// Get refresh token from browser local storage
pub fn get_refresh_token() -> Option<String> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item("doxle_refresh_token")
        .ok()?
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
            let _ = storage.remove_item("doxle_refresh_token");
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
        let error_msg =
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
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

///Refresh access token using refresh token
pub async fn refresh_access_token() -> Result<AuthResult, String> {
    tracing::info!("🔄 Refreshing access token");

    let refresh_token = get_refresh_token().ok_or("No refresh token found")?;

    let request = json! ({
        "refresh_token": refresh_token,
    });

    let endpoint = format!("{}/refresh", API_BASE_URL);
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

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        tracing::error!("Token refresh failed: {}", error_text);
        return Err(error_text);
    }

    let auth_result: AuthResult = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

    // Store the new tokens
    store_token(&auth_result.id_token);
    store_refresh_token(&auth_result.refresh_token);
    tracing::info!("✅ Token refresh successful!");
    Ok(auth_result)
}
