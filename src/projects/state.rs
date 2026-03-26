use super::api::{Project, api_list_projects, api_create_project, api_rename_project};
use dioxus::prelude::*;

pub static PROJECTS: GlobalSignal<Vec<Project>> = Signal::global(|| Vec::new());
pub static PROJECTS_LOADING: GlobalSignal<bool> = Signal::global(|| false);

/// Load all projects from API
pub async fn state_load_projects() {
    *PROJECTS_LOADING.write() = true;

    match api_list_projects().await {
        Ok(list) => {
            tracing::info!("✅ Projects loaded: {} items", list.len());
            *PROJECTS.write() = list;
        }
        Err(e) => {
            tracing::error!("❌ Error loading projects: {}", e);
        }
    }

    *PROJECTS_LOADING.write() = false;
}

/// Create a new project and add to local state
pub async fn state_create_project(name: String, company: Option<String>, description: Option<String>) -> Result<Project, String> {
    let project = api_create_project(name, company, description).await?;
    PROJECTS.write().push(project.clone());
    Ok(project)
}

/// Rename project and keep local state in sync
pub async fn state_rename_project(project_id: &str, new_name: String) -> Result<Project, String> {
    let old_name = PROJECTS
        .read()
        .iter()
        .find(|p| p.project_id == project_id)
        .map(|p| p.project_name.clone());

    // Optimistic local update
    PROJECTS.write().iter_mut().for_each(|p| {
        if p.project_id == project_id {
            p.project_name = new_name.clone();
        }
    });

    match api_rename_project(project_id, new_name).await {
        Ok(updated) => {
            PROJECTS.write().iter_mut().for_each(|p| {
                if p.project_id == project_id {
                    *p = updated.clone();
                }
            });
            Ok(updated)
        }
        Err(e) => {
            // Rollback optimistic update on failure
            if let Some(old) = old_name {
                PROJECTS.write().iter_mut().for_each(|p| {
                    if p.project_id == project_id {
                        p.project_name = old.clone();
                    }
                });
            }
            Err(e)
        }
    }
}
