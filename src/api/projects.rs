use serde::{Deserialize, Serialize};
use super::client;

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
    web_sys::console::log_1(&"📋 Fetching projects list...".into());
    // Use shared client to add Authorization and X-User-Id automatically
    let projects: Vec<Project> = client::get("/projects").await?;
    web_sys::console::log_1(&"✅ Projects loaded successfully!".into());
    Ok(projects)
}
