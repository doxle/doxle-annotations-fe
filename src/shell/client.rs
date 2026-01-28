use crate::api::auth_api::{get_access_token, refresh_access_token};
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use web_sys::RequestCredentials;

// Shared API configuration
// Local: http://localhost:9000
// Production: https://api.doxle.ai
pub const API_BASE_URL: &str = "https://api.doxle.ai";
// For local backend development, temporarily switch to:
// pub const API_BASE_URL: &str = "http://localhost:9000";

// CloudFront CDN for image caching
pub const CLOUDFRONT_URL: &str = "https://d1flb4kxeu5kb6.cloudfront.net";



/// Handle 401 Unauthorized by clearing token and redirecting to login
pub fn handle_unauthorized(){
    tracing::warn!("🔒 Received 401 Unauthorized, redirecting to login");
    crate::api::clear_access_token();
    #[cfg(target_arch="wasm32")]
    {
        if let Some(win) = web_sys::window(){
            let _ = win.location().set_href("/signin");
        }
    }
}


/// Check response status and handle auth errors
/// Returns Ok(response) if successful, Err if failed

async fn check_response(response:reqwest::Response)-> Result<reqwest::Response, String>{
    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        handle_unauthorized();
        return Err("Unauthorized - please log in again".to_string());
    }

    if !status.is_success(){
       let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Request failed ({}): {}", status.as_u16(), error_text)); 
    }
    Ok(response)
}



// Convert S3 URL to CloudFront URL for cached image loading
pub fn to_cloudfront_url(s3_url: &str) -> String {
    // Example S3 URL: https://doxle-annotations.s3.amazonaws.com/projects/.../image.jpg
    // Convert to: https://d1flb4kxeu5kb6.cloudfront.net/proxy-image/projects/.../image.jpg

    if let Some(path) = s3_url.split("doxle-app.s3.amazonaws.com/").nth(1) {
        format!("{}/proxy-image/{}", CLOUDFRONT_URL, path)
    } else if let Some(path) = s3_url.split("s3.amazonaws.com/doxle-app/").nth(1) {
        format!("{}/proxy-image/{}", CLOUDFRONT_URL, path)
    } else {
        // Fallback to original URL if parsing fails
        s3_url.to_string()
    }
}

// Helper to get auth header
pub fn auth_header() -> Result<String, String> {
    get_access_token()
        .map(|token| format!("Bearer {}", token))
        .ok_or_else(|| "No auth token found".to_string())
}

// Helper to get user_id from JWT token
pub fn get_user_id() -> Option<String> {
    get_access_token().and_then(|token| {
        // Decode JWT (split by '.' and get payload)
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        // Decode base64 payload
        use base64::{engine::general_purpose, Engine as _};
        let decoded = general_purpose::URL_SAFE_NO_PAD.decode(parts[1]).ok()?;
        let json_str = String::from_utf8(decoded).ok()?;

        // Parse JSON and extract 'sub' claim
        let json: serde_json::Value = serde_json::from_str(&json_str).ok()?;
        json.get("sub")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    })
}

// Generic GET helper
pub async fn get<R: for<'de> Deserialize<'de>>(endpoint: &str) -> Result<R, String> {
   
    
    let url = format!("{}{}", API_BASE_URL, endpoint);



   let response = gloo_net::http::Request::get(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status() == 401 {
        handle_unauthorized();
        return Err("Unauthorized - please log in again".to_string());
    }

    if !response.ok() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Request failed ({}): {}", response.status(), error_text));
    }

    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

// Generic POST helper
pub async fn post<T: Serialize, R: for<'de> Deserialize<'de>>(
    endpoint: &str,
    body: &T,
) -> Result<R, String> {
    
    let url = format!("{}{}", API_BASE_URL, endpoint);

    let response = Request::post(&url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .json(body)
        .map_err(|e| format!("Failed to serialize body: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status() == 401 {
        handle_unauthorized();
        return Err("Unauthorized - please log in again".to_string());
    }

    if !response.ok() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Request failed ({}): {}", response.status(), error_text));
    }

    
    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

// Generic PATCH helper
pub async fn patch<T: Serialize, R: for<'de> Deserialize<'de>>(
    endpoint: &str,
    body: &T,
) -> Result<R, String> {
    
    let url = format!("{}{}", API_BASE_URL, endpoint);
    
    let response = Request::patch(&url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .json(body)
        .map_err(|e| format!("Failed to serialize body: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status() == 401 {
        handle_unauthorized();
        return Err("Unauthorized - please log in again".to_string());
    }

    if !response.ok() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Request failed ({}): {}", response.status(), error_text));
    }
    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))

}


// Generic PATCH helper that doesn't expect a response body
pub async fn patch_no_response<T: Serialize>(endpoint: &str, body: &T) -> Result<(), String> {
    let url = format!("{}{}", API_BASE_URL, endpoint);

    let response = Request::patch(&url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .json(body)
        .map_err(|e| format!("Failed to serialize body: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status() == 401 {
        handle_unauthorized();
        return Err("Unauthorized - please log in again".to_string());
    }

    if response.ok() || response.status() == 204 {
        Ok(())
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Patch failed: {}", error_text))
    }

}


// Generic DELETE helper
pub async fn delete(endpoint: &str) -> Result<(), String> {
    let url = format!("{}{}", API_BASE_URL, endpoint);

    let response = Request::delete(&url)
        .credentials(RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status() == 401 {
        handle_unauthorized();
        return Err("Unauthorized - please log in again".to_string());
    }

    if response.ok() || response.status() == 204 {
        Ok(())
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Delete failed: {}", error_text))
    }


   
}
