use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use web_sys::{window, RequestCredentials};
use wasm_bindgen::JsCast;

const API_BASE_URL: &str = "https://api.doxle.ai";

#[derive(Debug, Serialize)]
struct SignInRequest {
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

// POST /login (backend endpoint)
// Cookies are set automatically by the browser from Set-Cookie headers
pub async fn authenticate(email: &str, password: &str) -> Result<AuthResult, String> {
    tracing::info!("🔐 Starting authentication for: {}", email);

    let request = SignInRequest {
        email: email.to_string(),
        password: password.to_string(),
    };

    let endpoint = format!("{}/login", API_BASE_URL);

    let response = Request::post(&endpoint)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .json(&request)
        .map_err(|e| format!("Failed to serialize: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        let error_msg = serde_json::from_str::<ErrorResponse>(&error_text)
            .map(|er| er.message)
            .unwrap_or(error_text);
        return Err(error_msg);
    }

    let auth_result: AuthResult = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(auth_result)
}

// POST /signup
pub async fn signup(email: &str, password: &str, invite_code: &str) -> Result<(), String> {
    tracing::info!("📝 Starting signup for: {}", email);

    let request = SignupRequest {
        email: email.to_string(),
        password: password.to_string(),
        invite_code: invite_code.to_string(),
    };

    let endpoint = format!("{}/signup", API_BASE_URL);

    let response = Request::post(&endpoint)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .json(&request)
        .map_err(|e| format!("Failed to serialize: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        let error_msg = serde_json::from_str::<ErrorResponse>(&error_text)
            .map(|er| er.message)
            .unwrap_or(error_text);
        return Err(error_msg);
    }

    Ok(())
}

// With httpOnly cookies, we don't need to store tokens in localStorage
// The browser handles cookie storage automatically
// These functions are kept for backward compatibility but are mostly no-ops now

pub fn get_access_token() -> Option<String> {
    // Access token is now in a cookie - we can read it from document.cookie
    // since it's not httpOnly (for user_id extraction)
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(document) = window().and_then(|w| w.document()) {
            // Cast Document to HtmlDocument to access cookie methods
            if let Ok(html_doc) = document.dyn_into::<web_sys::HtmlDocument>() {
                if let Ok(cookies) = html_doc.cookie() {
                    for cookie in cookies.split(';') {
                        let cookie = cookie.trim();
                        if cookie.starts_with("access_token=") {
                            return cookie.strip_prefix("access_token=").map(|s| s.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn clear_access_token() {
    // Clear cookies by setting them to expire
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(document) = window().and_then(|w| w.document()) {
            // Cast Document to HtmlDocument to access cookie methods
            if let Ok(html_doc) = document.dyn_into::<web_sys::HtmlDocument>() {
                // Clear access_token cookie
                let _ = html_doc.set_cookie("access_token=; Domain=.doxle.ai; Path=/; Max-Age=0; SameSite=None; Secure");
                // Note: refresh_token is httpOnly so we can't clear it from JS
                // It will be cleared by the backend on logout
            }
        }
    }
}

// POST /refresh - now relies on httpOnly cookie
pub async fn refresh_access_token() -> Result<AuthResult, String> {
    tracing::info!("🔄 Refreshing access token");

    let endpoint = format!("{}/refresh", API_BASE_URL);

    // Send empty body - refresh token is in httpOnly cookie
    let response = Request::post(&endpoint)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body("{}")
        .map_err(|e| format!("Failed to create request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(error_text);
    }

    let auth_result: AuthResult = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

    // Cookies are automatically set by browser from Set-Cookie headers
    Ok(auth_result)
}










