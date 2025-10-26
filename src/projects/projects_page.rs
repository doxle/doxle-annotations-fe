use dioxus::prelude::*;
use crate::Route;
use crate::state::{PROJECTS, PROJECTS_LOADING, PROJECTS_ERROR, load_projects, USER, load_user, get_project_by_id, remove_project, add_project};
use crate::shared::{AppSidebar, AppNavbar, NavbarContext, ProtectedRoute};
use crate::api;
use super::add_project::AddProjectModal;
use super::rename_project::RenameProjectModal;
use super::share_project::ShareProjectModal;
use super::project_dropdown::ProjectDropdown;

#[component]
pub fn ProjectsPage() -> Element {
    let nav = navigator();

    // Load data on mount - runs only once
    use_hook(|| {
        // Initialize WebSocket connection
        // api::init_websocket(); // Disabled for local dev - enable after deployment
        
        spawn(async move {
            // Load user if not already loaded
            if USER.read().is_none() {
                load_user().await;
            }
            
            // Load projects if not already loaded
            if PROJECTS.read().is_empty() && !*PROJECTS_LOADING.read() {
                load_projects().await;
            }
        });
    });

    let mut project_context_menu = use_signal(|| None::<(String, f64, f64)>); // (project_id, x, y)
    let mut show_add_modal = use_signal(|| false);
    let mut show_rename_modal = use_signal(|| false);
    let mut rename_project_id = use_signal(|| String::new());
    let mut rename_project_name = use_signal(|| String::new());
    let mut show_share_modal = use_signal(|| false);
    let mut share_project_id = use_signal(|| String::new());
    let mut sidebar_open = use_signal(|| true); // Sidebar open by default

    let handle_logout = move |_: Event<MouseData>| {
        // Clear the auth token
        api::clear_token();
        // Redirect to signin
        nav.push(Route::LoginPage {});
    };

    let handle_add_project = move |_: Event<MouseData>| {
        *show_add_modal.write() = true;
    };

    let handle_project_click = move |project_id: String| {
        println!("Project {} clicked", project_id);
        nav.push(Route::BlocksPage { project_id });
    };

    rsx! {
    ProtectedRoute {
        div {
            class: if sidebar_open() { "projects-container sidebar-open" } else { "projects-container" },
        onclick: move |_| {
            project_context_menu.set(None);
        },
        
        // Add Project Modal
        AddProjectModal {
            show: show_add_modal
        }
        
        // Rename Project Modal
        RenameProjectModal {
            show: show_rename_modal,
            project_id: rename_project_id.read().clone(),
            current_name: rename_project_name.read().clone(),
        }
        
        // Share Project Modal
        ShareProjectModal {
            show: show_share_modal,
            project_id: share_project_id.read().clone(),
        }

        // App Navbar
        AppNavbar {
            context: NavbarContext::Projects,
            sidebar_open: sidebar_open
        }

        // Shared sidebar
        AppSidebar { 
            open: sidebar_open
        }

        div {
            class: "projects-content",

            // Projects view
            div {
                    // Header with title and add button
                    div {
                        class: "projects-title-bar",

                        h1 {
                            class: "projects-title",
                            if let Some(user) = USER.read().as_ref() {
                                "{user.name}'s Projects"
                            } else {
                                "My Projects"
                            }
                        }

                        button {
                            class: "btn-create-project",
                            onclick: handle_add_project,
                            svg {
                                width: "16",
                                height: "16",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                line { x1: "12", y1: "5", x2: "12", y2: "19" }
                                line { x1: "5", y1: "12", x2: "19", y2: "12" }
                            }
                            span { "Create Project" }
                        }
                    }

                    // Projects grid or empty state
                    if *PROJECTS_LOADING.read() {
                        div {
                            class: "projects-grid-loading",
                            "Loading projects..."
                        }
                    } else if let Some(error) = PROJECTS_ERROR.read().as_ref() {
                        div {
                            class: "projects-grid-error",
                            "Error loading projects: {error}"
                        }
                    } else if PROJECTS.read().is_empty() {
                        div {
                            class: "projects-grid-empty",
                            "No projects available"
                        }
                    } else {
                        div {
                            class: "projects-grid",

                            for (idx, proj) in PROJECTS.read().iter().enumerate() {
                                {   let pid = proj.project_id.clone();
                                    let pname = proj.name.clone();
                                    let ptype = proj.project_type.clone();
                                    let pcreated = proj.created_at.clone();
                                    rsx! {
                                div {
                                    key: "{idx}",
                                    onclick: {
                                        let pid = pid.clone();
                                        move |e| {
                                        e.stop_propagation();
                                        handle_project_click(pid.clone());
                                    }},
                                    oncontextmenu: {
                                        let pid = pid.clone();
                                        move |e| {
                                        e.prevent_default();
                                        e.stop_propagation();
                                        let client_x = e.client_coordinates().x;
                                        let client_y = e.client_coordinates().y;
                                        project_context_menu.set(Some((pid.clone(), client_x, client_y)));
                                    }},
                                    class: "project-card",

                                    // Share button (appears on hover)
                                    button {
                                        class: "btn-share-project",
                                        onclick: {
                                            let pid = pid.clone();
                                            move |e| {
                                                e.stop_propagation();
                                                *share_project_id.write() = pid.clone();
                                                *show_share_modal.write() = true;
                                            }
                                        },
                                        "Share"
                                    }

                                    // Top section
                                    div {
                                        class: "project-card-top",

                                        // Image placeholder
                                        div {
                                            class: "project-card-image",
                                        }
                                    }

                                    // Divider
                                    div {
                                        class: "project-card-divider",
                                    }

                                    // Bottom section
                                    div {
                                        class: "project-card-bottom",

                                        // Title
                                        h3 {
                                            class: "project-card-title",
                                            "{pname}"
                                        }

                                        // Stats
                                        p {
                                            class: "project-card-stats",
                                            "Type: {ptype}"
                                        }

                                        // Created at
                                        p {
                                            class: "project-card-update",
                                            "Created: {pcreated}"
                                        }
                                    }
                                }
                                    }
                                }
                            }
                        }
                    }

                    // Project dropdown menu
                    if let Some((proj_id, x, y)) = project_context_menu.read().clone() {
                        ProjectDropdown {
                            x: x,
                            y: y,
                            project_id: proj_id.clone(),
                            on_close: project_context_menu,
                            show_rename_modal: show_rename_modal,
                            rename_project_id: rename_project_id,
                            rename_project_name: rename_project_name,
                            show_share_modal: show_share_modal,
                            share_project_id: share_project_id
                        }
                    }
                }
            }
        }
    }
    }
}
