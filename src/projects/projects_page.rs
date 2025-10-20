use dioxus::prelude::*;
use crate::Route;
use crate::home::api::cognito;

#[component]
pub fn ProjectsPage() -> Element {
    let nav = navigator();
    let mut projects = use_signal(|| vec![
        ("Project Alpha", 13, 5075, 10000, 20, "Edited 2 mins ago"),
        ("Project Beta", 8, 3200, 5000, 15, "Edited 1 hour ago"),
        ("Project Gamma", 5, 1500, 8000, 12, "Edited 3 hours ago"),
    ]);
    
    
    let mut project_context_menu = use_signal(|| None::<(usize, f64, f64)>); // (project_index, x, y)

    let handle_logout = move |_| {
        // Clear the auth token
        cognito::clear_token();
        // Redirect to signin
        nav.push(Route::LoginPage {});
    };

    let handle_add_project = move |_| {
        // TODO: Implement add project logic
        println!("Add project clicked");
    };

    let handle_project_click = move |index: usize| {
        println!("Project {} clicked", index);
        nav.push(Route::BlocksPage { project_id: index.to_string() });
    };
    

    rsx! {
        div {
            class: "projects-container",
            onclick: move |_| {
                project_context_menu.set(None);
            },
            
            // Header with user avatar
            div {
                class: "projects-header-wrapper",
                
                // User avatar button
                div {
                    class: "user-avatar",
                    onclick: handle_logout,
                    title: "Click to logout",
                    "S"
                }
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
                                "Projects"
                            }
                            
                            button {
                                class: "btn-add-project",
                                onclick: handle_add_project,
                                "+ Add Project"
                            }
                        }
                        
                        // Projects grid
                        div {
                            class: "projects-grid",
                            
                            for (index, (name, users, images, annotations, classes, last_update)) in projects.read().iter().enumerate() {
                                div {
                                    key: "{index}",
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        handle_project_click(index);
                                    },
                                    oncontextmenu: move |e| {
                                        e.prevent_default();
                                        e.stop_propagation();
                                        let client_x = e.client_coordinates().x;
                                        let client_y = e.client_coordinates().y;
                                        project_context_menu.set(Some((index, client_x, client_y)));
                                    },
                                    class: "project-card",
                                    
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
                                            "{name}"
                                        }
                                        
                                        // Stats with monospace font
                                        p {
                                            class: "project-card-stats",
                                            "{images} images / {annotations} annotated"
                                        }
                                        
                                        // Last update
                                        p {
                                            class: "project-card-update",
                                            "{last_update}"
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Project context menu
                        if let Some((proj_idx, x, y)) = *project_context_menu.read() {
                            div {
                                class: "context-menu",
                                style: "left: {x}px; top: {y}px;",
                                onclick: move |e| {
                                    e.stop_propagation();
                                },
                                
                                div {
                                    class: "context-menu-item",
                                    onclick: move |_| {
                                        println!("Delete project {} clicked", proj_idx);
                                        projects.write().remove(proj_idx);
                                        project_context_menu.set(None);
                                    },
                                    "Delete"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
