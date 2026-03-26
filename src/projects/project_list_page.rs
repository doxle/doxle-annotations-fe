use dioxus::prelude::*;
use crate::Route;
use crate::shell::{progress, AppNavbar, ProtectedRoute, THEME, Theme};
use crate::users::state::USER;
use super::api::api_delete_project;
use super::project_menu::ProjectMenu;
use super::state::{PROJECTS, PROJECTS_LOADING, state_load_projects, state_rename_project};

const PROJECTS_CSS: &str = include_str!("project_list_page.css");
const ADD_ICON_LIGHT: Asset = asset!("/assets/icons/add-light.svg");
const ADD_ICON_DARK: Asset = asset!("/assets/icons/add-dark.svg");

#[component]
pub fn ProjectsPage() -> Element {
    let navigator = use_navigator();
    let is_admin = USER.read().as_ref().map(|u| u.is_admin()).unwrap_or(false);
    let is_dark = THEME() == Theme::Dark;
    let add_icon = if is_dark { ADD_ICON_DARK } else { ADD_ICON_LIGHT };
    let mut open_menu_id: Signal<Option<String>> = use_signal(|| None);
    let mut renaming_project: Signal<Option<(String, String)>> = use_signal(|| None); // (project_id, current_name)
    let mut rename_input = use_signal(String::new);

    // Load projects on mount
    tracing::info!("📋 ProjectsPage render — PROJECTS count: {}, LOADING: {}", PROJECTS.read().len(), PROJECTS_LOADING());
    use_resource(move || async move {
        tracing::info!("📋 use_resource fired — PROJECTS.peek empty: {}", PROJECTS.peek().is_empty());
        if PROJECTS.peek().is_empty() {
            tracing::info!("📋 Calling state_load_projects...");
            state_load_projects().await;
            tracing::info!("📋 state_load_projects done — count: {}", PROJECTS.read().len());
        }
    });

    if PROJECTS_LOADING() {
        return rsx! {
            ProtectedRoute {
                AppNavbar {}
                div { class: "projects-page",
                    div { class: "projects-loading", "Loading projects..." }
                }
            }
        };
    }

    let projects = PROJECTS.read().clone();
    tracing::info!("📋 Projects to render: {}", projects.len());

    rsx! {
        ProtectedRoute {
            AppNavbar {
                if is_admin {
                    button {
                        class: "app-navbar-center-button",
                        onclick: move |_| {
                            navigator.push(Route::CreateProjectPage {});
                        },
                        img { src: add_icon, class: "app-navbar-center-button-icon" }
                        "New Project"
                    }
                }
            }
            style { {PROJECTS_CSS} }
            div {
                class: "projects-page",
                onclick: move |_| {
                    if open_menu_id().is_some() {
                        open_menu_id.set(None);
                    }
                },
                div {
                    class: "projects-heading-row",
                    div { class: "projects-heading", "Projects" }
                }
                if projects.is_empty() {
                    div { class: "projects-empty",
                        if is_admin {
                            button {
                                class: "create-project-btn create-project-btn-primary",
                                onclick: move |_| { navigator.push(Route::CreateProjectPage {}); },
                                "Create your first project"
                            }
                        } else {
                            "No projects available yet"
                        }
                    }
                } else {
                    ul { class: "projects-list",
                        for project in projects.iter() {
                            {
                                let pid = project.project_id.clone();
                                let name = project.project_name.clone();
                                rsx! {
                                    li {
                                        key: "{pid}",
                                        class: "project-card",
                                        onclick: move |_| {
                                            if open_menu_id().is_some() {
                                                open_menu_id.set(None);
                                                return;
                                            }
                                            navigator.push(Route::BlocksPage { project_id: pid.clone() });
                                        },
                                        {
                                            let pid_toggle = project.project_id.clone();
                                            let pid_new_tab = project.project_id.clone();
                                            let pid_copy = project.project_id.clone();
                                            let pid_rename = project.project_id.clone();
                                            let pname_rename = project.project_name.clone();
                                            let pid_delete = project.project_id.clone();
                                            rsx! {
                                                ProjectMenu {
                                                    project_id: pid_toggle.clone(),
                                                    is_open: open_menu_id() == Some(pid_toggle.clone()),
                                                    on_toggle: move |_| {
                                                        let p = pid_toggle.clone();
                                                        if open_menu_id() == Some(p.clone()) {
                                                            open_menu_id.set(None);
                                                        } else {
                                                            open_menu_id.set(Some(p));
                                                        }
                                                    },
                                                    on_rename: move |_| {
                                                        renaming_project.set(Some((pid_rename.clone(), pname_rename.clone())));
                                                        rename_input.set(pname_rename.clone());
                                                        open_menu_id.set(None);
                                                    },
                                                    on_open_new_tab: move |_| {
                                                        if let Some(window) = web_sys::window() {
                                                            let path = format!("/projects/{}/blocks", pid_new_tab);
                                                            let target = match window.location().origin() {
                                                                Ok(origin) => format!("{}{}", origin, path),
                                                                Err(_) => path,
                                                            };
                                                            let _ = window.open_with_url_and_target(&target, "_blank");
                                                        }
                                                        open_menu_id.set(None);
                                                    },
                                                    on_copy_link: move |_| {
                                                        if let Some(window) = web_sys::window() {
                                                            let path = format!("/projects/{}/blocks", pid_copy);
                                                            let full_url = match window.location().origin() {
                                                                Ok(origin) => format!("{}{}", origin, path),
                                                                Err(_) => path,
                                                            };
                                                            let escaped = full_url.replace('\\', "\\\\").replace('\"', "\\\"");
                                                            document::eval(&format!("navigator.clipboard && navigator.clipboard.writeText(\"{}\");", escaped));
                                                            progress::show_info("Project link copied");
                                                        }
                                                        open_menu_id.set(None);
                                                    },
                                                    on_delete: move |_| {
                                                        let id = pid_delete.clone();
                                                        spawn(async move {
                                                            match api_delete_project(&id).await {
                                                                Ok(_) => {
                                                                    PROJECTS.write().retain(|p| p.project_id != id);
                                                                    progress::show_success("Project deleted");
                                                                }
                                                                Err(e) => {
                                                                    progress::show_error_persistent(&format!("Failed to delete project: {}", e));
                                                                }
                                                            }
                                                        });
                                                        open_menu_id.set(None);
                                                    },
                                                }
                                            }
                                        }
                                        div { class: "project-card-name", "{name}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some((project_id, current_name)) = renaming_project() {
                {
                    let project_id_for_enter = project_id.clone();
                    let current_name_for_enter = current_name.clone();
                    rsx! {
                        div {
                            class: "create-project-overlay",
                            onclick: move |_| renaming_project.set(None),
                            div {
                                class: "create-project-modal",
                                onclick: move |e| e.stop_propagation(),
                                input {
                                    class: "create-project-input",
                                    value: "{rename_input}",
                                    oninput: move |e| rename_input.set(e.value()),
                                    onkeypress: move |e| {
                                        if e.key() == Key::Enter {
                                            let id = project_id_for_enter.clone();
                                            let old_name = current_name_for_enter.clone();
                                            let next_name = rename_input().trim().to_string();
                                            if next_name.is_empty() {
                                                progress::show_error("Project name is required");
                                                return;
                                            }
                                            if next_name == old_name {
                                                renaming_project.set(None);
                                                return;
                                            }
                                            spawn(async move {
                                                match state_rename_project(&id, next_name).await {
                                                    Ok(_) => progress::show_success("Project renamed"),
                                                    Err(e) => progress::show_error_persistent(&format!("Failed to rename project: {}", e)),
                                                }
                                            });
                                            renaming_project.set(None);
                                        }
                                    },
                                    placeholder: "{current_name}",
                                    autofocus: true,
                                }
                                div {
                                    class: "create-project-actions",
                                    button {
                                        class: "create-project-btn create-project-btn-secondary",
                                        onclick: move |_| renaming_project.set(None),
                                        "Cancel"
                                    }
                                    button {
                                        class: "create-project-btn create-project-btn-primary",
                                        onclick: move |_| {
                                            let id = project_id.clone();
                                            let old_name = current_name.clone();
                                            let next_name = rename_input().trim().to_string();
                                            if next_name.is_empty() {
                                                progress::show_error("Project name is required");
                                                return;
                                            }
                                            if next_name == old_name {
                                                renaming_project.set(None);
                                                return;
                                            }
                                            spawn(async move {
                                                match state_rename_project(&id, next_name).await {
                                                    Ok(_) => progress::show_success("Project renamed"),
                                                    Err(e) => progress::show_error_persistent(&format!("Failed to rename project: {}", e)),
                                                }
                                            });
                                            renaming_project.set(None);
                                        },
                                        "Save"
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
