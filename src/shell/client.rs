use serde::{Deserialize, Serialize};
use serde_json::json;
use gloo_net::http::Request;
use web_sys::RequestCredentials;

/// Distinguishes auth failures from transient/network errors
#[derive(Debug, Clone)]
pub enum ApiError {
    /// 401 after refresh attempt failed — user must sign in again
    Unauthorized(String),
    /// Any other error (network, 4xx/5xx, parse) — not an auth issue
    Other(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Unauthorized(msg) => write!(f, "{}", msg),
            ApiError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

// Shared API configuration — toggle comment for local vs deploy
pub const API_BASE_URL: &str = "http://localhost:9000"; // LOCAL
// pub const API_BASE_URL: &str = "https://api.doxle.ai"; // DEPLOY

// CloudFront CDN for image caching
pub const CLOUDFRONT_URL: &str = "https://d1flb4kxeu5kb6.cloudfront.net";


// If we get 401 we need to call refresh to refresh cookie from BE
async fn refresh_session() -> Result<(), String> {
    let url = format!("{}/refresh", API_BASE_URL);
    let resp = Request::post(&url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .json(&json!({}))
        .map_err(|e| format!("Failed to serialize refresh body: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error during refresh: {}", e))?;
    if resp.ok() { Ok(()) } else {
        let txt = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
        Err(format!("Refresh failed ({}): {}", resp.status(), txt))
    }

}

/// Handle 401 Unauthorized by clearing cookies (server-side) and redirecting to login
pub fn handle_unauthorized() {
    tracing::warn!("🔒 Received 401 Unauthorized, redirecting to login");

    #[cfg(target_arch = "wasm32")]
    {
        // Clear httpOnly cookies via backend
        dioxus::prelude::spawn(async {
            let _ = crate::api::logout().await;
        });

        if let Some(win) = web_sys::window() {
            let _ = win.location().set_href("/signin");
        }
    }
}




// Convert S3 URL to CloudFront URL for cached image loading
pub fn to_cloudfront_url(s3_url: &str) -> String {
    // Example S3 URL: https://doxle-annotations.s3.amazonaws.com/projects/.../image.jpg
    // Convert to: https://d1flb4kxeu5kb6.cloudfront.net/proxy-image/projects/.../image.jpg

    if let Some(path) = s3_url.split("doxle-app.s3.amazonaws.com/").nth(1) {
        format!("{}/proxy-image/{}", CLOUDFRONT_URL, path)
    } else if let Some(path) = s3_url.split("s3.amazonaws.com/doxle-app/").nth(1) {
        format!("{}/proxy-image/{}", CLOUDFRONT_URL, path)
    } else if !s3_url.starts_with("http") {
        // Treat as a raw S3 key (e.g. "annotations/blocks/.../image.png")
        format!("{}/proxy-image/{}", CLOUDFRONT_URL, s3_url)
    } else {
        // Fallback to original URL if parsing fails
        s3_url.to_string()
    }
}


// Generic GET helper
pub async fn get<R: for<'de> Deserialize<'de>>(endpoint: &str) -> Result<R, String> {
    get_typed(endpoint).await.map_err(|e| e.to_string())
}

/// GET that returns ApiError so callers can distinguish 401 from other errors
pub async fn get_typed<R: for<'de> Deserialize<'de>>(endpoint: &str) -> Result<R, ApiError> {
    let url = format!("{}{}", API_BASE_URL, endpoint);
    let mut tried_refresh = false;

    loop{
        let resp = Request::get(&url)
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(|e| ApiError::Other(format!("Network error: {}", e)))?;

        // If 401 try to get refresh from BE
        if resp.status() == 401 && !tried_refresh {
            tried_refresh = true;
            if refresh_session().await.is_ok() { continue; }
            return Err(ApiError::Unauthorized("Unauthorized - please log in again".into()));
        }
        if !resp.ok() {
            let txt = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
            return Err(ApiError::Other(format!("Request failed ({}): {}", resp.status(), txt)));
        }
        return resp.json().await.map_err(|e| ApiError::Other(format!("Failed to parse response: {}", e)));
    }
}

// Generic GET helper that returns raw text (for file downloads)
pub async fn get_text(endpoint: &str) -> Result<String, String> {
    let url = format!("{}{}", API_BASE_URL, endpoint);
    let mut tried_refresh = false;

    loop {
        let resp = Request::get(&url)
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if resp.status() == 401 && !tried_refresh {
            tried_refresh = true;
            if refresh_session().await.is_ok() { continue; }
            handle_unauthorized();
            return Err("Unauthorized - please log in again".into());
        }
        if !resp.ok() {
            let txt = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
            return Err(format!("Request failed ({}): {}", resp.status(), txt));
        }
        return resp.text().await.map_err(|e| format!("Failed to read response: {}", e));
    }
}

// Generic POST helper
pub async fn post<T: Serialize, R: for<'de> Deserialize<'de>>(
    endpoint: &str,
    body: &T,
) -> Result<R, String> {
    
    let url = format!("{}{}", API_BASE_URL, endpoint);
    let mut tried_refresh = false;

    loop {
        let resp = Request::post(&url)
            .credentials(RequestCredentials::Include)
            .header("Content-Type", "application/json")
            .json(body)
            .map_err(|e| format!("Failed to serialize body: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if resp.status() == 401 && !tried_refresh {
            tried_refresh = true;
            if refresh_session().await.is_ok() {
                continue;
            }
            handle_unauthorized();
            return Err("Unauthorized - please log in again".into());
        }
        if !resp.ok() {
            let txt = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
            return Err(format!("Request failed ({}): {}", resp.status(), txt));
        }
        return resp.json().await.map_err(|e| format!("Failed to parse response: {}", e));
    }

}

// Generic PATCH helper
pub async fn patch<T: Serialize, R: for<'de> Deserialize<'de>>(
    endpoint: &str,
    body: &T,
) -> Result<R, String> {
    
    let url = format!("{}{}", API_BASE_URL, endpoint);
    
    let mut tried_refresh = false;

    loop {
        let resp = Request::patch(&url)
            .credentials(RequestCredentials::Include)
            .header("Content-Type", "application/json")
            .json(body)
            .map_err(|e| format!("Failed to serialize body: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if resp.status() == 401 && !tried_refresh {
            tried_refresh = true;
            if refresh_session().await.is_ok() {
                continue;
            }
            handle_unauthorized();
            return Err("Unauthorized - please log in again".into());
        }
        if !resp.ok() {
            let txt = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
            return Err(format!("Request failed ({}): {}", resp.status(), txt));
        }

        return resp.json().await.map_err(|e| format!("Failed to parse response: {}", e));
    }

}


// Generic PATCH helper that doesn't expect a response body
pub async fn patch_no_response<T: Serialize>(endpoint: &str, body: &T) -> Result<(), String> {
    let url = format!("{}{}", API_BASE_URL, endpoint);

    let mut tried_refresh = false;

    loop {
        let resp = Request::patch(&url)
            .credentials(RequestCredentials::Include)
            .header("Content-Type", "application/json")
            .json(body)
            .map_err(|e| format!("Failed to serialize body: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if resp.status() == 401 && !tried_refresh {
            tried_refresh = true;
            if refresh_session().await.is_ok() {
                continue;
            }
            handle_unauthorized();
            return Err("Unauthorized - please log in again".into());
        }

        if resp.ok() || resp.status() == 204 {
            return Ok(());
        } else {
            let txt = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
            return Err(format!("Patch failed: {}", txt));
        }
    }

}


// Generic DELETE helper
pub async fn delete(endpoint: &str) -> Result<(), String> {
    let url = format!("{}{}", API_BASE_URL, endpoint);

    let mut tried_refresh = false;

    loop {
        let resp = Request::delete(&url)
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if resp.status() == 401 && !tried_refresh {
            tried_refresh = true;
            if refresh_session().await.is_ok() {
                continue;
            }
            handle_unauthorized();
            return Err("Unauthorized - please log in again".into());
        }

        if resp.ok() || resp.status() == 204 {
            return Ok(());
        } else {
            let txt = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
            return Err(format!("Delete failed: {}", txt));
        }
    }

   
}

