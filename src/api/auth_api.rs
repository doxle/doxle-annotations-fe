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

#[derive(Debug, Serialize)]
struct CreateInviteRequest {
    email: String,
    role: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InviteResponse {
    pub invite_code: String,
    pub email: String,
    pub role: String,
    pub expires_at: String,
    pub created_at: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SignupResponse {
    pub message: String,
    #[serde(default)]
    pub role: Option<String>,
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
pub async fn signup(email: &str, password: &str, invite_code: &str) -> Result<SignupResponse, String> {
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

    let signup_resp: SignupResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse signup response: {}", e))?;

    Ok(signup_resp)
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

// POST /invites - create invite and send email
pub async fn create_invite(email: &str, role: &str) -> Result<InviteResponse, String> {
    tracing::info!("📨 Creating invite for: {} with role: {}", email, role);

    let request = CreateInviteRequest {
        email: email.to_string(),
        role: role.to_string(),
    };

    let endpoint = format!("{}/invites", API_BASE_URL);

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

    let invite: InviteResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(invite)
}

// GET /invites - list all invites (admin-only)
pub async fn list_invites() -> Result<Vec<InviteResponse>, String> {
    tracing::info!("📋 Listing invites");
    crate::shell::client::get::<Vec<InviteResponse>>("/invites").await
}

// DELETE /invites/{code} - delete invite (admin-only)
pub async fn delete_invite(invite_code: &str) -> Result<(), String> {
    tracing::info!("🗑️ Deleting invite: {}", invite_code);
    let endpoint = format!("/invites/{}", invite_code);
    crate::shell::client::delete(&endpoint).await
}


