use super::client_api;
use serde::{Deserialize, Serialize};

// Predefined color palette for labels (40 colors)
pub const LABEL_COLORS: [&str; 40] = [
    "#EF4444", "#F97316", "#F59E0B", "#EAB308", "#84CC16", "#22C55E", "#10B981", "#14B8A6",
    "#06B6D4", "#0EA5E9", "#3B82F6", "#6366F1", "#8B5CF6", "#A855F7", "#D946EF", "#EC4899",
    "#F43F5E", "#FB7185", "#FDA4AF", "#FCA5A5", "#FCD34D", "#BEF264", "#86EFAC", "#6EE7B7",
    "#5EEAD4", "#7DD3FC", "#93C5FD", "#A5B4FC", "#C4B5FD", "#E9D5FF", "#F0ABFC", "#F9A8D4",
    "#BE123C", "#C2410C", "#A16207", "#CA8A04", "#65A30D", "#16A34A", "#047857", "#0F766E",
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Label {
    #[serde(default)]
    pub label_id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub project_id: String,
    pub name: String,
    pub project_type: String,
    pub locked: bool,
    pub labels: Vec<Label>,
    pub created_at: String,
}

// GET /projects - list user's projects
pub async fn list_projects() -> Result<Vec<Project>, String> {
    web_sys::console::log_1(&"📋 Fetching projects list...".into());
    // Use shared client to add Authorization and X-User-Id automatically
    let projects: Vec<Project> = client_api::get("/projects").await?;
    web_sys::console::log_1(&"✅ Projects loaded successfully!".into());
    Ok(projects)
}
