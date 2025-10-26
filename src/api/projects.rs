use serde::{Deserialize, Serialize};
use super::client::API_BASE_URL;
use super::auth;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub project_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: String,
    pub locked: bool,
    pub created_at: String,
}

// GET /projects - list user's projects
pub async fn list_projects() -> Result<Vec<Project>, String> {
    let start = js_sys::Date::now();
    web_sys::console::log_1(&"📋 Fetching projects list...".into());
    let token = auth::get_token().ok_or("No auth token found")?;
    
    let endpoint = format!("{}/projects", API_BASE_URL);
    web_sys::console::log_1(&format!("📡 GET {}", endpoint).into());
    
    let before_request = js_sys::Date::now();
    let client = reqwest::Client::new();
    let response = client
        .get(&endpoint)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("Network error: {}", e);
            web_sys::console::error_1(&error_msg.clone().into());
            error_msg
        })?;
    
    let after_request = js_sys::Date::now();
    web_sys::console::log_1(&format!("⏱️ Network request took: {}ms", after_request - before_request).into());
    web_sys::console::log_1(&format!("📥 Response status: {}", response.status()).into());

    let status = response.status();
    
    if status.is_success() {
        let before_parse = js_sys::Date::now();
        let projects = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse projects: {}", e))?;
        let after_parse = js_sys::Date::now();
        
        web_sys::console::log_1(&format!("⏱️ JSON parsing took: {}ms", after_parse - before_parse).into());
        web_sys::console::log_1(&format!("⏱️ Total API call took: {}ms", after_parse - start).into());
        web_sys::console::log_1(&"✅ Projects loaded successfully!".into());
        Ok(projects)
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        let error_msg = format!("Failed to get projects: {} - {}", status, error_text);
        web_sys::console::error_1(&error_msg.clone().into());
        Err(error_msg)
    }
}
