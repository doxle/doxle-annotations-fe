use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use web_sys::RequestCredentials;
use crate::core::client::API_BASE_URL;
use crate::Route;
use std::str::FromStr;

#[derive(Debug, Serialize)]
struct SignInRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct SignupRequest {
    email: String,
    password: String,
    access_token: String,
}

#[derive(Debug, Serialize)]
struct ConfirmSignupRequest {
    email: String,
    code: String,
}

#[derive(Debug, Serialize)]
struct ContactRequest {
    email: String,
    message: String,
}

#[derive(Debug, Serialize)]
pub struct CreateInviteRequest {
    email: String,
    project_id: String,
    allowed_block_ids: Vec<String>,
    permission: String,
    target_path: String,
    expires_days: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InviteResponse {
    pub invite_code: String,
    #[serde(default)]
    pub email: Option<String>,
    pub project_id: String,
    #[serde(default)]
    pub allowed_block_ids: Vec<String>,
    pub permission: String,
    pub target_path: String,
    pub expires_at: String,
    pub created_at: String,
    pub status: String,
    #[serde(default)]
    pub accepted_at: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SignupResponse {
    pub message: String,
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
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub access_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PendingSignupState {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateInvitePayload {
    pub email: String,
    pub project_id: String,
    pub allowed_block_ids: Vec<String>,
    pub permission: String,
    pub target_path: String,
    pub expires_days: i64,
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
    if let Some(access_token) = session.access_token.as_ref() {
        store_local_access_token(access_token);
    }
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Some(refresh_token) = session.refresh_token.as_ref() {
                    let _ = storage.set_item("refresh_token", refresh_token);
                }
                if let Some(username) = session.username.as_ref() {
                    let _ = storage.set_item("cognito_username", username);
                }
            }
        }
    }

    Ok(session)
}

// POST /signup
pub async fn signup(email: &str, password: &str, access_token: &str) -> Result<SignupResponse, String> {
    tracing::info!("📝 Starting signup for: {}", email);

    let request = SignupRequest {
        email: email.to_string(),
        password: password.to_string(),
        access_token: access_token.to_string(),
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

pub async fn confirm_signup(email: &str, code: &str) -> Result<(), String> {
    let request = ConfirmSignupRequest {
        email: email.to_string(),
        code: code.to_string(),
    };

    let endpoint = format!("{}/signup/confirm", API_BASE_URL);

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

// POST /invites - create access link and send email
pub async fn create_invite(payload: CreateInvitePayload) -> Result<InviteResponse, String> {
    tracing::info!("📨 Creating invite for: {}", payload.email);

    let request = CreateInviteRequest {
        email: payload.email,
        project_id: payload.project_id,
        allowed_block_ids: payload.allowed_block_ids,
        permission: payload.permission,
        target_path: payload.target_path,
        expires_days: payload.expires_days,
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

pub async fn get_invite(invite_code: &str) -> Result<InviteResponse, String> {
    let endpoint = format!("/invites/{}", invite_code);
    crate::core::client::get::<InviteResponse>(&endpoint).await
}

pub async fn accept_invite(invite_code: &str) -> Result<InviteResponse, String> {
    let endpoint = format!("{}/invites/{}/accept", API_BASE_URL, invite_code);

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
        let error_msg = serde_json::from_str::<ErrorResponse>(&error_text)
            .map(|er| er.message)
            .unwrap_or(error_text);
        return Err(error_msg);
    }

    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

// GET /invites - list all invites (admin-only)
pub async fn list_invites() -> Result<Vec<InviteResponse>, String> {
    tracing::info!("📋 Listing invites");
    crate::core::client::get::<Vec<InviteResponse>>("/invites").await
}

// DELETE /invites/{code} - delete invite (admin-only)
pub async fn delete_invite(invite_code: &str) -> Result<(), String> {
    tracing::info!("🗑️ Deleting invite: {}", invite_code);
    let endpoint = format!("/invites/{}", invite_code);
    crate::core::client::delete(&endpoint).await
}

pub fn store_access_token(token: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.session_storage() {
                let _ = storage.set_item("access_token", token);
                let _ = storage.set_item("invite_code", token);
            }
        }
    }
}

pub fn store_local_access_token(token: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("local_access_token", token);
            }
        }
    }
}

pub fn get_local_access_token() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(token)) = storage.get_item("local_access_token") {
                    if !token.trim().is_empty() {
                        return Some(token);
                    }
                }
            }
        }
    }
    None
}

pub fn has_persisted_session_hint() -> bool {
    if get_local_access_token().is_some() {
        return true;
    }
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let has_refresh_token = storage
                    .get_item("refresh_token")
                    .ok()
                    .flatten()
                    .map(|value| !value.trim().is_empty())
                    .unwrap_or(false);
                let has_username = storage
                    .get_item("cognito_username")
                    .ok()
                    .flatten()
                    .map(|value| !value.trim().is_empty())
                    .unwrap_or(false);
                return has_refresh_token || has_username;
            }
        }
    }
    false
}

pub fn clear_local_access_token() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.remove_item("local_access_token");
            }
        }
    }
}

pub fn get_stored_access_token() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.session_storage() {
                if let Ok(Some(token)) = storage.get_item("access_token") {
                    if !token.trim().is_empty() {
                        return Some(token);
                    }
                }
                if let Ok(Some(token)) = storage.get_item("invite_code") {
                    if !token.trim().is_empty() {
                        return Some(token);
                    }
                }
            }
        }
    }
    None
}

pub fn clear_stored_access_token() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.session_storage() {
                let _ = storage.remove_item("access_token");
                let _ = storage.remove_item("invite_code");
            }
        }
    }
}

pub fn store_pending_signup(state: &PendingSignupState) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(serialized) = serde_json::to_string(state) {
                    let _ = storage.set_item("pending_signup", &serialized);
                }
            }
        }
    }
}

pub fn get_pending_signup() -> Option<PendingSignupState> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(serialized)) = storage.get_item("pending_signup") {
                    if let Ok(state) = serde_json::from_str::<PendingSignupState>(&serialized) {
                        return Some(state);
                    }
                }
            }
        }
    }
    None
}

pub fn clear_pending_signup() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.remove_item("pending_signup");
            }
        }
    }
}

pub fn navigate_to_path(path: &str) -> Result<Route, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Missing redirect path.".to_string());
    }

    Route::from_str(trimmed)
        .map_err(|_| format!("Unsupported redirect path: {}", trimmed))
}


