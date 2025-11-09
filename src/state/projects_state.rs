use dioxus::prelude::*;
use crate::api;

// Global signals for projects state
pub static PROJECTS: GlobalSignal<Vec<api::Project>> = Signal::global(|| Vec::new());
pub static PROJECTS_LOADING: GlobalSignal<bool> = Signal::global(|| false);
pub static PROJECTS_ERROR: GlobalSignal<Option<String>> = Signal::global(|| None);

/// Load projects from API and update global state
pub async fn load_projects() {
    let start = js_sys::Date::now();
    web_sys::console::log_1(&"🚀 [STATE] Starting load_projects()".into());
    
    *PROJECTS_LOADING.write() = true;
    *PROJECTS_ERROR.write() = None;
    
    let before_api = js_sys::Date::now();
    match api::list_projects().await {
        Ok(proj_list) => {
            let after_api = js_sys::Date::now();
            web_sys::console::log_1(&format!("⏱️ [STATE] API call completed in: {}ms", after_api - before_api).into());
            
            tracing::info!("✅ Projects loaded: {} items", proj_list.len());
            let before_write = js_sys::Date::now();
            *PROJECTS.write() = proj_list;
            let after_write = js_sys::Date::now();
            
            web_sys::console::log_1(&format!("⏱️ [STATE] Writing to global state took: {}ms", after_write - before_write).into());
        }
        Err(e) => {
            tracing::error!("❌ Error loading projects: {}", e);
            *PROJECTS_ERROR.write() = Some(e);
        }
    }
    
    *PROJECTS_LOADING.write() = false;
    let end = js_sys::Date::now();
    web_sys::console::log_1(&format!("✅ [STATE] load_projects() completed in: {}ms", end - start).into());
}

/// Get a project by ID from the global state
pub fn get_project_by_id(project_id: &str) -> Option<api::Project> {
    PROJECTS
        .read()
        .iter()
        .find(|p| p.project_id == project_id)
        .cloned()
}

/// Remove a project from the global state by ID
pub fn remove_project(project_id: &str) {
    PROJECTS.write().retain(|p| p.project_id != project_id);
}

/// Add a project to the global state
pub fn add_project(project: api::Project) {
    PROJECTS.write().push(project);
}

/// Delete a project (optimistic with rollback)
pub async fn delete_project(project_id: String) {
    tracing::info!("🗑️ Starting delete for project: {}", project_id);
    
    // Save project data for potential rollback
    let project_backup = get_project_by_id(&project_id);
    
    // Step 1: IMMEDIATELY remove from UI (optimistic)
    remove_project(&project_id);
    tracing::info!("⚡ UI updated immediately (optimistic)");
    
    // Step 2: Send HTTP DELETE to backend
    tracing::info!("📡 Sending HTTP DELETE /projects/{}", project_id);
    
    match api::client_api::delete(&format!("/projects/{}", project_id)).await {
        Ok(_) => {
            tracing::info!("✅ Delete confirmed by server for project: {}", project_id);
        }
        Err(e) => {
            tracing::error!("❌ Delete failed for project {}: {}", project_id, e);
            
            // Show user-friendly error
            if let Some(window) = web_sys::window() {
                let _ = window.alert_with_message(&format!("Failed to delete project: {}", e));
            }
            
            // Rollback: add project back to UI
            if let Some(project) = project_backup {
                let project_name = project.name.clone();
                add_project(project);
                tracing::info!("🔄 Project restored to UI: {}", project_name);
            }
        }
    }
}
