use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use web_sys::window;

// fn ss_set(key:&str, val:&str){
//     if let Some(win) = window(){
//         if let Ok(Some(ss)) = win.session_storage(){
//             let _ = ss.set_item(key, val);
//         }
//     }
// }

// fn ss_get(key:&str)->Option<String>{
//     window()
//     .and_then(|w| w.session_storage().ok().flatten())
//     .and_then(|ss| ss.get_item(key).ok().flatten())

// }

fn ls_set(key:&str, val:&str){
    if let Some(win) = window(){
        if let Ok(Some(ls)) = win.local_storage(){
            let _ = ls.set_item(key, val);
        }
    }
}

fn ls_get(key:&str)->Option<String>{
    window()
    .and_then(|w| w.local_storage().ok().flatten())
    .and_then(|ls| ls.get_item(key).ok().flatten())

}




const API_BASE_URL: &str = "https://api.doxle.ai";

pub static ACCESS_TOKEN: GlobalSignal<Option<String>> = Signal::global(|| None);
pub static REFRESH_TOKEN: GlobalSignal<Option<String>> = Signal::global(|| None);

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
pub async fn authenticate(email: &str, password: &str) -> Result<AuthResult, String> {
    tracing::info!("🔐 Starting authentication for: {}", email);

    let request = SignInRequest {
        email: email.to_string(),
        password: password.to_string(),
    };

    let endpoint = format!("{}/login", API_BASE_URL);
    let client = reqwest::Client::new();

    let response = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
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
    let client = reqwest::Client::new();

    let response = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
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

// Token storage
pub fn store_access_token(access_token: &str) {
    *ACCESS_TOKEN.write() = Some(access_token.to_string());
     ls_set("access_token", access_token);
}

pub fn get_access_token() -> Option<String> {
    if let Some(t) = ACCESS_TOKEN.read().clone() {
        return Some(t);
    }
    if let Some(t) = ls_get("access_token") {
        *ACCESS_TOKEN.write() = Some(t.clone());
        return Some(t);
    }
    None
}

pub fn is_access_token_expired(token:&str) -> bool {
    let parts: Vec<&str> = token.split('.').collect();
    tracing::info!("jwt parts: {}", parts.len());
    if parts.len() != 3 {
        return true;
    }

    use base64::{engine::general_purpose, Engine as _};

    // header log
    if let Ok(hdr_bytes) = general_purpose::URL_SAFE_NO_PAD.decode(parts[0]) {
        if let Ok(hdr_str) = String::from_utf8(hdr_bytes) {
            tracing::info!("jwt header: {}", hdr_str);
        }
    }

    // payload decode (used for both logging + exp)
    let payload_bytes = match general_purpose::URL_SAFE_NO_PAD.decode(parts[1]) {
        Ok(d) => d,
        Err(_) => return true,
    };
    let payload_str = match String::from_utf8(payload_bytes) {
        Ok(s) => s,
        Err(_) => return true,
    };
    let json: serde_json::Value = match serde_json::from_str(&payload_str) {
        Ok(j) => j,
        Err(_) => return true,
    };

    let keys: Vec<_> = json.as_object()
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();
    let exp = json.get("exp").and_then(|x| x.as_i64());
    tracing::info!("jwt payload keys: {:?}, exp: {:?}", keys, exp);

    let exp = match exp {
        Some(e) => e,
        None => return true,
    };
    let now = (js_sys::Date::now() / 1000.0) as i64;
    exp <= now
}

pub fn get_access_token_exp(token: &str) -> Option<i64> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    use base64::{engine::general_purpose, Engine as _};

    let payload_bytes = general_purpose::URL_SAFE_NO_PAD.decode(parts[1]).ok()?;
    let payload_str = String::from_utf8(payload_bytes).ok()?;
    let json: serde_json::Value = serde_json::from_str(&payload_str).ok()?;
    json.get("exp").and_then(|x| x.as_i64())
}

pub fn clear_access_token() {
    *ACCESS_TOKEN.write() = None;
    *REFRESH_TOKEN.write() = None;
    // Also clear from local storage
    if let Some(win) = window() {
        if let Ok(Some(ls)) = win.local_storage() {
            let _ = ls.remove_item("access_token");
            let _ = ls.remove_item("refresh_token");
        }
    }
}

pub fn store_refresh_token(refresh_token: &str) {
    *REFRESH_TOKEN.write() = Some(refresh_token.to_string());
    ls_set("refresh_token", refresh_token);
}

pub fn get_refresh_token() -> Option<String> {
    if let Some(t) = REFRESH_TOKEN.read().clone() {
        return Some(t);
    }
    if let Some(t) = ls_get("refresh_token") {
        *REFRESH_TOKEN.write() = Some(t.clone());
        return Some(t);
    }
    None
}


// POST /refresh
pub async fn refresh_access_token() -> Result<AuthResult, String> {
    tracing::info!("🔄 Refreshing access token");

    let refresh_token = get_refresh_token().ok_or("No refresh token found")?;
    let request = json!({ "refresh_token": refresh_token });

    let endpoint = format!("{}/refresh", API_BASE_URL);
    let client = reqwest::Client::new();

    let response = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
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

    store_access_token(&auth_result.access_token);
    store_refresh_token(&auth_result.refresh_token);

    Ok(auth_result)
}










