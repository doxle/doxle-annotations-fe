use serde::{Deserialize, Serialize};

// API Gateway endpoint
const API_BASE_URL: &str = "https://qxh6o5rw2b.execute-api.ap-southeast-2.amazonaws.com";

#[derive(Debug, Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthResult {
    pub access_token: String,
    pub id_token: String,
    pub refresh_token: String,
    pub expires_in: i32,
}

pub async fn authenticate(email: &str, password: &str) -> Result<AuthResult, String> {
    web_sys::console::log_1(&format!("🔐 Starting authentication for: {}", email).into());

    let request = LoginRequest {
        email: email.to_string(),
        password: password.to_string(),
    };

    let endpoint = format!("{}/login", API_BASE_URL);
    web_sys::console::log_1(&format!("📡 Calling auth endpoint: {}", endpoint).into());

    let client = reqwest::Client::new();
    
    let response = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("❌ Network error: {}", e);
            web_sys::console::error_1(&error_msg.clone().into());
            error_msg
        })?;

    web_sys::console::log_1(&format!("📥 Response status: {}", response.status()).into());

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        let error_msg = format!("❌ Authentication failed: {}", error_text);
        web_sys::console::error_1(&error_msg.clone().into());
        return Err(error_msg);
    }

    let response_text = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
    web_sys::console::log_1(&format!("📄 Raw response: {}", response_text).into());

    let auth_result: AuthResult = serde_json::from_str(&response_text)
        .map_err(|e| {
            let error_msg = format!("❌ Failed to parse response: {}", e);
            web_sys::console::error_1(&error_msg.clone().into());
            error_msg
        })?;

    web_sys::console::log_1(&"✅ Authentication successful!".into());
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
