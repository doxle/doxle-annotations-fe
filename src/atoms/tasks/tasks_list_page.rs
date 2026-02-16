use dioxus::prelude::*;
use crate::Route;
use crate::blocks::dashboard::state::state_set_current_block;
use crate::atoms::tasks::state::{state_load_tasks, state_load_tasks_silent, state_delete_task, state_rename_task, TASKS_LOADING, TASKS_ERROR, TASKS, DELETE_CURRENT, DELETE_TOTAL};
use crate::atoms::tasks::api::{api_assign_task, api_set_reviewer};
use crate::users::state::USER;
use crate::users::api::{User, UserRole, list_users};
use super::task_menu::TaskMenu;
use super::edit_task_modal::EditTaskModal;

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
pub fn TasksListPage(block_id: String, block_name: String, block_type: String) -> Element {

    let block_id_for_hook = block_id.clone();
    let block_id_for_resource = block_id.clone();
    let block_id_for_navbar = block_id.clone();
    let block_id_for_create_task = block_id.clone();
    let block_name_for_create_task = block_name.clone();
    let block_type_for_create_task = block_type.clone();
    let block_name_for_nav = block_name.clone();
    let block_type_for_nav = block_type.clone();

    let nav = use_navigator();


    
    // Set current block on mount
    use_hook(||{
        state_set_current_block(&block_id_for_hook.clone());
    });
    
    // Load tasks on for a block mount
    use_resource(move || {
        let block_id = block_id_for_resource.clone();
        let is_empty = TASKS.peek().is_empty();
        async move {
            if is_empty {
                state_load_tasks(&block_id).await;
            } else {
                state_load_tasks_silent(&block_id).await;
            }
        }
    });



    

    let _creating = use_signal(|| false);
    let mut deleting_task_id: Signal<Option<String>> = use_signal(|| None);
    let is_admin = USER.read().as_ref().map(|u| u.is_admin()).unwrap_or(false);
    let is_dark = THEME() == Theme::Dark;

    // Load users for assignee/reviewer dropdowns
    let mut users: Signal<Vec<User>> = use_signal(Vec::new);
    use_resource(move || async move {
        match list_users().await {
            Ok(u) => users.set(u),
            Err(e) => tracing::error!("Failed to load users: {}", e),
        }
    });

    // Filter users by block type: annotation -> annotators+admins, file/building -> builders+admins
    let filtered_users: Vec<User> = {
        let bt = block_type.to_lowercase();
        users.read().iter().filter(|u| {
            u.user_role == UserRole::Admin || match bt.as_str() {
                "annotation" => u.user_role == UserRole::Annotator,
                _ => u.user_role == UserRole::Builder,
            }
        }).cloned().collect()
    };
    let tasks_icon = if is_dark { BLOCKS_ICON_DARK } else { BLOCKS_ICON_LIGHT };
    let add_icon = if is_dark { ADD_ICON_DARK } else { ADD_ICON_LIGHT };
    let mut open_menu_id: Signal<Option<String>> = use_signal(|| None);
    let mut editing_task: Signal<Option<(String, String)>> = use_signal(|| None); // (task_id, current_name)
    

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
    
    // If no tasks, show centered "+ NEW TASK" button (admin) or empty message (annotator)
    if TASKS.read().is_empty() {
        return rsx! {
            style { {TASKS_LIST_CSS} }
            AppNavbar {}
            div { class: "tasks-empty-page",
                if is_admin {
                    button {
                        class: "tasks-create-button-centered",
                        onclick: move |_| {
                        nav.push(Route::CreateTaskPage { block_id: block_id_for_create_task.clone(), block_name: block_name_for_create_task.clone(), block_type: block_type_for_create_task.clone() });
                        },
                        img { src: add_icon, class: "tasks-create-icon" }
                        "New Task"
                    }
                } else {
                    div { class: "tasks-empty-message", "No tasks assigned" }
                }
            }
        };
    }
    
    // If tasks exist, show list
    rsx! {
        style { {TASKS_LIST_CSS} }
        AppNavbar {
            if is_admin {
                button {
                    class: "app-navbar-center-button",
                    onclick: move |_| {
                        nav.push(Route::CreateTaskPage { block_id: block_id_for_navbar.clone(), block_name: block_name.clone(), block_type: block_type_for_nav.clone() });
                    },
                    img { src: add_icon, class: "app-navbar-center-button-icon" }
                    "New Task"
                }
            }
        }
        div { 
            class: "tasks-page",
            // Close dropdown menu when clicked outside
            onclick: move |_| {
                if open_menu_id().is_some() {
                    open_menu_id.set(None);
                }
            },
            div { class: "tasks-list-container",
                ul { class: "tasks-list",
                    for task in &*TASKS.read() {
                        {
                            let task_id = task.task_id.clone();
                            let task_name = task.task_name.clone();
                            let task_name_for_nav = task.task_name.clone();
                            let task_state = task.task_state.clone();
                            let block_id_for_nav = block_id.clone();
                            let block_type_for_task_nav = block_type.clone();
                            let block_name = block_name_for_nav.clone();
                            // Get first image info (or defaults if no images)
                            let (image_id, image_name) = task.images.first()
                                .map(|img| (img.image_id.clone(), extract_filename(&img.url)))
                                .unwrap_or_else(|| ("no-image".to_string(), "No Image".to_string()));

                            let is_deleting = deleting_task_id() == Some(task.task_id.clone());
                            let del_current = DELETE_CURRENT();
                            let del_total = DELETE_TOTAL();
                            rsx! {
                                li {
                                    key: "{task.task_id}",
                                    class: if is_deleting { "tasks-list-item deleting" } else { "tasks-list-item" },
                                    onclick: move |_| {
                                        // Don't navigate if menu is open
                                        if open_menu_id().is_some() {
                                            open_menu_id.set(None);
                                            return;
                                        }
                                        nav.push(Route::AnnotationCanvasPage { 
                                            block_id: block_id_for_nav.clone(),
                                            block_name: block_name.clone(),
                                            block_type: block_type_for_task_nav.clone(),
                                            task_id: task_id.clone(),
                                            task_name: task_name_for_nav.clone(),
                                            image_id: image_id.clone(),
                                            image_name: image_name.clone(),
                                        });
                                    },
                                    
                                    // Task card content
                                    div { class: "task-card",
                                        // Row 1: Task name + block name + three dots menu
                                        div {
                                            class: "task-row-1",
                                            div {
                                                class: "task-name-group",
                                                div { 
                                                    class: "task-name",
                                                    "{task_name}" 
                                                }
                                                div {
                                                    class: "task-block-name",
                                                    "{block_name}"
                                                }
                                            }
                                            // Task dropdown menu (admin only)
                                            if is_admin {
                                                div {
                                                    class: "task-actions",
                                                    {
                                                        let tid = task.task_id.clone();
                                                        let tid_toggle = task.task_id.clone();
                                                        let tid_delete = task.task_id.clone();
                                                        let bid_delete = block_id.clone();
                                                        rsx!{
                                                            TaskMenu {
                                                                task_id: tid.clone(),
                                                                is_open: open_menu_id() == Some(task.task_id.clone()),
                                                                on_toggle: move |_| {
                                                                    let t = tid_toggle.clone();
                                                                    if open_menu_id() == Some(t.clone()) {
                                                                        open_menu_id.set(None);
                                                                    } else {
                                                                        open_menu_id.set(Some(t));
                                                                    }
                                                                },
                                                                on_edit: {
                                                                    let tid_edit = task.task_id.clone();
                                                                    let tname_edit = task.task_name.clone();
                                                                    move |_| {
                                                                        editing_task.set(Some((tid_edit.clone(), tname_edit.clone())));
                                                                        open_menu_id.set(None);
                                                                    }
                                                                },
                                                                on_archive: move |_| {
                                                                    // TODO: archive task
                                                                    open_menu_id.set(None);
                                                                },
                                                                on_delete: move |_| {
                                                                    let id = tid_delete.clone();
                                                                    let bid = bid_delete.clone();
                                                                    deleting_task_id.set(Some(id.clone()));
                                                                    spawn(async move {
                                                                        state_delete_task(&bid, &id).await;
                                                                        deleting_task_id.set(None);
                                                                    });
                                                                    open_menu_id.set(None);
                                                                },
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        // Assignee and Reviewer row
                                        div { class: "task-meta-row",
                                            // Assignee dropdown
                                            {
                                                let bid_assign = block_id.clone();
                                                let tid_assign = task.task_id.clone();
                                                let assignee_val = task.assignee.clone();
                                                rsx! {
                                                    div { class: "task-dropdown-group",
                                                        label { class: "task-dropdown-label", "Assignee" }
                                                        select {
                                                            class: "task-dropdown",
                                                            value: "{assignee_val}",
                                                            onclick: move |e| e.stop_propagation(),
                                                            onchange: move |e| {
                                                                let val = e.value();
                                                                let bid = bid_assign.clone();
                                                                let tid = tid_assign.clone();
                                                                spawn(async move {
                                                                    let _ = api_assign_task(&bid, &tid, &val).await;
                                                                });
                                                            },
                                                            option { value: "", "Unassigned" }
                                                            for user in filtered_users.iter() {
                                                                {
                                                                    let uid = user.user_id.clone();
                                                                    let uname = user.user_name.clone();
                                                                    let is_sel = assignee_val == uid;
                                                                    rsx! { option { value: "{uid}", selected: is_sel, "{uname}" } }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            
                                            // Reviewer dropdown
                                            {
                                                let bid_review = block_id.clone();
                                                let tid_review = task.task_id.clone();
                                                let reviewer_val = task.reviewer.clone();
                                                rsx! {
                                                    div { class: "task-dropdown-group",
                                                        label { class: "task-dropdown-label", "Reviewer" }
                                                        select {
                                                            class: "task-dropdown",
                                                            value: "{reviewer_val}",
                                                            onclick: move |e| e.stop_propagation(),
                                                            onchange: move |e| {
                                                                let val = e.value();
                                                                let bid = bid_review.clone();
                                                                let tid = tid_review.clone();
                                                                spawn(async move {
                                                                    let _ = api_set_reviewer(&bid, &tid, &val).await;
                                                                });
                                                            },
                                                            option { value: "", "Unassigned" }
                                                            for user in filtered_users.iter() {
                                                                {
                                                                    let uid = user.user_id.clone();
                                                                    let uname = user.user_name.clone();
                                                                    let is_sel = reviewer_val == uid;
                                                                    rsx! { option { value: "{uid}", selected: is_sel, "{uname}" } }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        // Delete progress overlay
                                        if is_deleting && del_total > 0 {
                                            div {
                                                class: "task-delete-progress",
                                                div {
                                                    class: "task-delete-progress-bar",
                                                    div {
                                                        class: "task-delete-progress-fill",
                                                        style: "width: {(del_current as f64 / del_total as f64 * 100.0) as u32}%",
                                                    }
                                                }
                                                span {
                                                    class: "task-delete-progress-text",
                                                    "Deleting {del_current}/{del_total}"
                                                }
                                            }
                                        }

                                        // Divider
                                        div { class: "task-divider" }
                                        
                                        // Image rectangles
                                        div { class: "task-image-rects",
                                            for i in 0..task.images.len() {
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
                                                span { class: "task-count-item", "{task.annotation_count} annotations" }
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
        // Edit modal
        if let Some((task_id, current_name)) = editing_task() {
            {
                let bid = block_id.clone();
                rsx! {
                    EditTaskModal {
                        task_id: task_id.clone(),
                        current_name: current_name.clone(),
                        on_save: move |new_name: String| {
                            let tid = task_id.clone();
                            let bid = bid.clone();
                            spawn(async move {
                                state_rename_task(&bid, &tid, new_name).await;
                            });
                            editing_task.set(None);
                        },
                        on_cancel: move |_| {
                            editing_task.set(None);
                        }
                    }
                }
            }
        }
    }
   
}

