use crate::shell::client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Project {
    pub project_id: String,
    pub project_name: String,
    #[serde(default)]
    pub project_company: Option<String>,
    #[serde(default)]
    pub project_status: String,
    #[serde(default)]
    pub project_address: Option<String>,
    #[serde(default)]
    pub project_email: Option<String>,
    #[serde(default)]
    pub project_owner: String,
    #[serde(default)]
    pub project_members: Vec<String>,
    #[serde(default)]
    pub project_start_date: Option<String>,
    #[serde(default)]
    pub project_end_date: Option<String>,
    #[serde(default)]
    pub project_description: Option<String>,
    #[serde(default)]
    pub project_budget: Option<String>,
    #[serde(default)]
    pub project_client_name: Option<String>,
    #[serde(default)]
    pub project_created_at: String,
    #[serde(default)]
    pub project_updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct CreateProjectRequest {
    pub project_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_company: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_description: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct UpdateProjectRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
}

/// GET /projects
pub async fn api_list_projects() -> Result<Vec<Project>, String> {
    client::get::<Vec<Project>>("/projects").await
}

/// POST /projects
pub async fn api_create_project(name: String, company: Option<String>, description: Option<String>) -> Result<Project, String> {
    let body = CreateProjectRequest { project_name: name, project_company: company, project_description: description };
    client::post::<CreateProjectRequest, Project>("/projects", &body).await
}
/// PATCH /projects/{id} (rename)
pub async fn api_rename_project(project_id: &str, project_name: String) -> Result<Project, String> {
    let endpoint = format!("/projects/{}", project_id);
    let body = UpdateProjectRequest { project_name: Some(project_name) };
    client::patch(&endpoint, &body).await
}

/// DELETE /projects/{id}
pub async fn api_delete_project(project_id: &str) -> Result<(), String> {
    let endpoint = format!("/projects/{}", project_id);
    client::delete(&endpoint).await
}
