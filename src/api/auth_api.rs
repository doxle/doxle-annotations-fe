use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use web_sys::RequestCredentials;
use crate::shell::client::API_BASE_URL;

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

#[derive(Debug, Serialize)]
struct ContactRequest {
    email: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SessionResponse {
    pub message: String,
    pub expires_in: i32,
}

// POST /login (backend endpoint)
// httpOnly cookies are set automatically by the browser from Set-Cookie headers
pub async fn authenticate(email: &str, password: &str) -> Result<SessionResponse, String> {
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

    let session: SessionResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(session)
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

// POST /contact - send contact form message
pub async fn send_contact(email: &str, message: &str) -> Result<(), String> {
    tracing::info!("📧 Sending contact message from: {}", email);

    let request = ContactRequest {
        email: email.to_string(),
        message: message.to_string(),
    };

    let endpoint = format!("{}/contact", API_BASE_URL);

    let response = Request::post(&endpoint)
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

pub async fn logout() -> Result<(), String> {
    tracing::info!("🚪 Logging out");

    let endpoint = format!("{}/logout", API_BASE_URL);

    let response = Request::post(&endpoint)
        .credentials(RequestCredentials::Include)
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

    Ok(())
}










