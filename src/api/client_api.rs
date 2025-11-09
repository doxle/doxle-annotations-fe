use super::auth_api;
use serde::{Deserialize, Serialize};

// Shared API configuration
// Local: http://localhost:9000
// Production: https://api.doxle.ai
pub const API_BASE_URL: &str = "https://api.doxle.ai";

// CloudFront CDN for image caching
pub const CLOUDFRONT_URL: &str = "https://d1flb4kxeu5kb6.cloudfront.net";

// Convert S3 URL to CloudFront URL for cached image loading
pub fn to_cloudfront_url(s3_url: &str) -> String {
    // Example S3 URL: https://doxle-annotations.s3.amazonaws.com/projects/.../image.jpg
    // Convert to: https://d1flb4kxeu5kb6.cloudfront.net/proxy-image/projects/.../image.jpg

    if let Some(path) = s3_url.split("doxle-annotations.s3.amazonaws.com/").nth(1) {
        format!("{}/proxy-image/{}", CLOUDFRONT_URL, path)
    } else if let Some(path) = s3_url.split("s3.amazonaws.com/doxle-annotations/").nth(1) {
        format!("{}/proxy-image/{}", CLOUDFRONT_URL, path)
    } else {
        // Fallback to original URL if parsing fails
        s3_url.to_string()
    }
}

// Helper to get auth header
pub fn auth_header() -> Result<String, String> {
    auth_api::get_token()
        .map(|token| format!("Bearer {}", token))
        .ok_or_else(|| "No auth token found".to_string())
}

// Helper to get user_id from JWT token
pub fn get_user_id() -> Option<String> {
    auth_api::get_token().and_then(|token| {
        // Decode JWT (split by '.' and get payload)
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        // Decode base64 payload
        use base64::{engine::general_purpose, Engine as _};
        let decoded = general_purpose::STANDARD.decode(parts[1]).ok()?;
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
    // Try to refresh token if it's about to expire
    let _ = try_refresh_if_needed().await;
    let token = auth_api::get_token().ok_or("No auth token found")?;
    let url = format!("{}{}", API_BASE_URL, endpoint);

    let client = reqwest::Client::new();
    let mut request = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token));

    // Add X-User-Id for local development
    if let Some(user_id) = get_user_id() {
        request = request.header("X-User-Id", user_id);
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Request failed: {}", error_text))
    }
}

// Generic POST helper
pub async fn post<T: Serialize, R: for<'de> Deserialize<'de>>(
    endpoint: &str,
    body: &T,
) -> Result<R, String> {
    // Try to refresh token if it's about to expire
    let _ = try_refresh_if_needed().await;
    let token = auth_api::get_token().ok_or("No auth token found")?;
    let url = format!("{}{}", API_BASE_URL, endpoint);

    let client = reqwest::Client::new();
    let mut request = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token));

    // Add X-User-Id for local development
    if let Some(user_id) = get_user_id() {
        request = request.header("X-User-Id", user_id);
    }

    let response = request
        .json(body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Request failed: {}", error_text))
    }
}

// Generic PATCH helper
pub async fn patch<T: Serialize, R: for<'de> Deserialize<'de>>(
    endpoint: &str,
    body: &T,
) -> Result<R, String> {
    // Try to refresh token if it's about to expire
    let _ = try_refresh_if_needed().await;
    let token = auth_api::get_token().ok_or("No auth token found")?;
    let url = format!("{}{}", API_BASE_URL, endpoint);

    let client = reqwest::Client::new();
    let mut request = client
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token));

    // Add X-User-Id for local development
    if let Some(user_id) = get_user_id() {
        request = request.header("X-User-Id", user_id);
    }

    let response = request
        .json(body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Request failed: {}", error_text))
    }
}

// Generic DELETE helper
pub async fn delete(endpoint: &str) -> Result<(), String> {
    // Try to refresh token if it's about to expire
    let _ = try_refresh_if_needed().await;
    let token = auth_api::get_token().ok_or("No auth token found")?;
    let url = format!("{}{}", API_BASE_URL, endpoint);

    let client = reqwest::Client::new();
    let mut request = client
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token));

    // Add X-User-Id for local development
    if let Some(user_id) = get_user_id() {
        request = request.header("X-User-Id", user_id);
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() || response.status() == reqwest::StatusCode::NO_CONTENT {
        Ok(())
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Delete failed: {}", error_text))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    #[serde(default)]
    pub name: String,
    pub email: String,
    pub company: Option<String>,
    pub role: String,
    pub created_at: String,
    pub last_login: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub company: Option<String>,
    pub role: String,
}

// Get current user profile (no auto-creation)
pub async fn get_current_user() -> Result<User, String> {
    tracing::info!("👤 Getting user profile");
    let token = auth_api::get_token().ok_or("No auth token found")?;
    get_user(&token).await
}

// Create user profile explicitly
pub async fn create_user_profile(
    name: String,
    email: String,
    company: Option<String>,
    role: String,
) -> Result<User, String> {
    tracing::info!("➕ Creating new user profile: {}", email);
    let token = auth_api::get_token().ok_or("No auth token found")?;

    let request_body = CreateUserRequest {
        name,
        email,
        company,
        role,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/users", API_BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        tracing::info!("✅ User profile created successfully");
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse user: {}", e))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        tracing::error!("❌ Failed to create user: {}", error_text);
        Err(format!("Failed to create user: {}", error_text))
    }
}

// GET /users/me
pub async fn get_user(token: &str) -> Result<User, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/users/me", API_BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse user: {}", e))
    } else {
        Err(format!("Failed to get user: {}", response.status()))
    }
}

// Check if token needs refresh and refresh it
async fn try_refresh_if_needed() -> Result<(), String> {
    use base64::{engine::general_purpose, Engine as _};

    let token = match auth_api::get_token() {
        Some(t) => t,
        None => return Ok(()), // No token, skip refresh
    };

    // Decode JWT to check expiration
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Ok(());
    }

    let decoded = match general_purpose::STANDARD.decode(parts[1]) {
        Ok(d) => d,
        Err(_) => return Ok(()),
    };

    let json_str = match String::from_utf8(decoded) {
        Ok(s) => s,
        Err(_) => return Ok(()),
    };

    let json: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(j) => j,
        Err(_) => return Ok(()),
    };

    // Get expiration time (exp claim is in seconds since epoch)
    let exp = match json.get("exp").and_then(|v| v.as_i64()) {
        Some(e) => e,
        None => return Ok(()),
    };

    let current_time = (js_sys::Date::now() / 1000.0) as i64;

    // Refresh if token expires in less than 2 minutes (120 seconds)
    if exp - current_time < 120 {
        tracing::info!("🔄 Token expiring soon, refreshing...");
        auth_api::refresh_access_token().await?;
    }

    Ok(())
}
