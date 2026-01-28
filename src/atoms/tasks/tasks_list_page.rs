use dioxus::prelude::*;
use crate::Route;
use crate::blocks::dashboard::state::{state_set_current_block, state_get_current_block};
use crate::atoms::tasks::state::{state_load_tasks, TASKS_LOADING, TASKS_ERROR, TASKS};

use crate::shell::{THEME, Theme, AppNavbar};
use dioxus::logger::tracing::info;

/// Extract filename from image URL (last segment after /)
fn extract_filename(url: &str) -> String {
    url.split('/').last().unwrap_or("image").to_string()
}

const TASKS_LIST_CSS: &str = include_str!("tasks_list_page.css");
const BLOCKS_ICON_LIGHT: Asset = asset!("/assets/icons/blocks-light.svg");
const BLOCKS_ICON_DARK: Asset = asset!("/assets/icons/blocks-dark.svg");
const ADD_ICON_LIGHT: Asset = asset!("/assets/icons/add-light.svg");
const ADD_ICON_DARK: Asset = asset!("/assets/icons/add-dark.svg");

/// Top-level page for an Annotation Block: shows tasks for a given block,
/// with a simple "create + list" flow similar to blocks.
#[component]
pub fn TasksListPage(block_id: String) -> Element {

    let block_id_for_hook = block_id.clone();
    let block_id_for_resource = block_id.clone();
    let block_id_for_navbar = block_id.clone();
     let block_id_for_create_task = block_id.clone();

    let nav = use_navigator();


    
    // Set current block on mount
    use_hook(||{
        state_set_current_block(&block_id_for_hook.clone());
    });
    
    // Load tasks on for a block mount
    use_resource(move || {
        let block_id = block_id_for_resource.clone();
        async move {
            state_load_tasks(&block_id).await;
        }
    });



    

    let _creating = use_signal(|| false);
    let is_dark = THEME() == Theme::Dark;
    let tasks_icon = if is_dark { BLOCKS_ICON_DARK } else { BLOCKS_ICON_LIGHT };
    let add_icon = if is_dark { ADD_ICON_DARK } else { ADD_ICON_LIGHT };
    

    // Show loading 
    if TASKS_LOADING() {
        return rsx! {
            style { {TASKS_LIST_CSS} }
            AppNavbar {}
            div { class: "tasks-page",
                div { "Loading tasks..." }
            }
        };
    }

    // Show error 
    if let Some(err) = TASKS_ERROR.read().as_ref() {
        return rsx! {
            style { {TASKS_LIST_CSS} }
            AppNavbar {}
            div { class: "tasks-page",
                div { style: "color: red;", "Error loading tasks: {err}" }
            }
        };
    }

    info!("TASKS-LIST page:");
    
    // If no tasks, show centered "+ NEW TASK" button
    if TASKS.read().is_empty() {
        return rsx! {
            style { {TASKS_LIST_CSS} }
            AppNavbar {}
            div { class: "tasks-empty-page",
                button {
                    class: "tasks-create-button-centered",
                    onclick: move |_| {
                        nav.push(Route::CreateTaskPage { block_id: block_id_for_create_task.clone() });
                    },
                    img { src: add_icon, class: "tasks-create-icon" }
                    "New Task"
                }
            }
        };
    }
    
    // If tasks exist, show list
    rsx! {
        style { {TASKS_LIST_CSS} }
        AppNavbar {
            button {
                class: "app-navbar-center-button",
                onclick: move |_| {
                    nav.push(Route::CreateTaskPage { block_id: block_id_for_navbar.clone() });
                },
                img { src: add_icon, class: "app-navbar-center-button-icon" }
                "New Task"
            }
        }
        div { class: "tasks-page",
            div { class: "tasks-list-container",
                ul { class: "tasks-list",
                    for task in &*TASKS.read() {
                        {
                            let task_id = task.task_id.clone();
                            let task_name = task.task_name.clone();
                            let task_name_for_nav = task.task_name.clone();
                            let task_state = task.task_state.clone();
                            let block_id_for_nav = block_id.clone();
                            // Get block name from current block
                            let block_name = state_get_current_block()
                                .map(|b| b.block_name)
                                .unwrap_or_else(|| "Block".to_string());
                            // Get first image info (or defaults if no images)
                            let (image_id, image_name) = task.images.first()
                                .map(|img| (img.image_id.clone(), extract_filename(&img.url)))
                                .unwrap_or_else(|| ("no-image".to_string(), "No Image".to_string()));

                            rsx! {
                                li {
                                    key: "{task.task_id}",
                                    class: "tasks-list-item",
                                    onclick: move |_| {
                                        nav.push(Route::AnnotationCanvasPage { 
                                            block_id: block_id_for_nav.clone(),
                                            block_name: block_name.clone(),
                                            task_id: task_id.clone(),
                                            task_name: task_name_for_nav.clone(),
                                            image_id: image_id.clone(),
                                            image_name: image_name.clone(),
                                        });
                                    },
                                    
                                    // Task card content
                                    div { class: "task-card",
                                        // Task name
                                        div { 
                                            class: "task-name",
                                            "{task_name}" 
                                        }
                                        
                                        // Assignee and Reviewer row
                                        div { class: "task-meta-row",
                                            // Assignee dropdown
                                            div { class: "task-dropdown-group",
                                                label { class: "task-dropdown-label", "Assignee" }
                                                select { 
                                                    class: "task-dropdown",
                                                    value: "{task.assignee}",
                                                    onclick: move |e| e.stop_propagation(),
                                                    onchange: move |_e| {
                                                        // TODO: Update assignee via API
                                                    },
                                                    option { value: "", "Unassigned" }
                                                    option { value: "user1", "User 1" }
                                                    option { value: "user2", "User 2" }
                                                    option { value: "user3", "User 3" }
                                                }
                                            }
                                            
                                            // Reviewer dropdown
                                            div { class: "task-dropdown-group",
                                                label { class: "task-dropdown-label", "Reviewer" }
                                                select { 
                                                    class: "task-dropdown",
                                                    value: "{task.reviewer}",
                                                    onclick: move |e| e.stop_propagation(),
                                                    onchange: move |_e| {
                                                        // TODO: Update reviewer via API
                                                    },
                                                    option { value: "", "Unassigned" }
                                                    option { value: "user1", "User 1" }
                                                    option { value: "user2", "User 2" }
                                                    option { value: "user3", "User 3" }
                                                }
                                            }
                                        }
                                        
                                        // Divider
                                        div { class: "task-divider" }
                                        
                                        // Image rectangles
                                        div { class: "task-image-rects",
                                            // TODO: Backend should provide total image count
                                            // For now showing 30 rectangles as placeholder
                                            for i in 0..30 {
                                                div { 
                                                    key: "{i}",
                                                    class: if (i + 1) % 10 == 0 { "task-image-rect-wrapper with-marker" } else { "task-image-rect-wrapper" },
                                                    if (i + 1) % 10 == 0 {
                                                        div { class: "task-image-marker" }
                                                    }
                                                    div { class: "task-image-rect" }
                                                }
                                            }
                                        }
                                        
                                        // Last updated and counts row
                                        div { class: "task-info-row",
                                            // Last updated info
                                            div { class: "task-updated-info",
                                                "Last updated by User on Dec.5.2025"
                                            }
                                            
                                            // Image and annotation counts
                                            div { class: "task-counts",
                                                span { class: "task-count-item", "{task.images.len()} images" }
                                                span { class: "task-count-divider", "/" }
                                                span { class: "task-count-item", "45,000 annotations" }
                                            }
                                        }
                                        
                                        // Task state (right side)
                                        span { class: "task-state", "{task_state}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
   
}

