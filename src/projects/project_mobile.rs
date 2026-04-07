use dioxus::prelude::*;
use crate::Route;
use super::project_page::{
    close_project_menu, ProjectListItem, ProjectsController, ADD_ICON_DARK, ADD_ICON_LIGHT,
    CLOSE_ICON_DARK, CLOSE_ICON_LIGHT, SEARCH_ICON_DARK, SEARCH_ICON_LIGHT,
};

#[component]
pub fn ProjectsMobile(controller: ProjectsController) -> Element {
    let navigator = use_navigator();
    let add_icon = if controller.is_dark { ADD_ICON_DARK } else { ADD_ICON_LIGHT };
    let search_icon = if controller.is_dark { SEARCH_ICON_DARK } else { SEARCH_ICON_LIGHT };
    let close_icon = if controller.is_dark { CLOSE_ICON_DARK } else { CLOSE_ICON_LIGHT };
    let search_active = controller.search_active;
    let search_query = controller.search_query;
    let is_search_active = search_active();
    let search_query_value = search_query();
    let controller_for_page = controller.clone();
    let mut search_query_for_input = search_query;
    let mut search_query_for_escape = search_query;
    let mut search_query_for_close = search_query;
    let mut search_active_for_escape = search_active;
    let mut search_active_for_close = search_active;
    let mut search_active_for_open = search_active;

    rsx! {
        div {
            class: "project-page",
            onclick: move |_| close_project_menu(&controller_for_page),
            if controller.projects.is_empty() {
                div {
                    class: "project-empty-page",
                    if controller.is_admin {
                        button {
                            class: "create-project-btn",
                            onclick: move |_| {
                                navigator.push(Route::CreateProjectPage {});
                            },
                            "Create Project"
                        }
                    } else {
                        "No projects available"
                    }
                }
            } else {
                div {
                    class: "project-list-container list-mode",
                    div {
                        class: if is_search_active { "new-page-btn search-mode" } else { "new-page-btn" },
                        if is_search_active {
                            button {
                                class: "search-icon-btn",
                                img { src: search_icon, class: "action-icon search-icon" }
                            }
                            input {
                                class: "search-input",
                                r#type: "text",
                                placeholder: "Search projects...",
                                value: "{search_query_value}",
                                oninput: move |e| search_query_for_input.set(e.value()),
                                onkeydown: move |e| {
                                    if e.key() == Key::Escape {
                                        search_active_for_escape.set(false);
                                        search_query_for_escape.set(String::new());
                                    }
                                },
                                onmounted: move |e| {
                                    let _ = e.set_focus(true);
                                },
                            }
                            button {
                                class: "close-search-btn",
                                onclick: move |_| {
                                    search_active_for_close.set(false);
                                    search_query_for_close.set(String::new());
                                },
                                img { src: close_icon, class: "action-icon" }
                            }
                        } else {
                            button {
                                class: "new-project-action",
                                onclick: move |_| {
                                    navigator.push(Route::CreateProjectPage {});
                                },
                                img { src: add_icon, class: "action-icon" }
                                "New Project"
                            }
                            div { class: "action-divider" }
                            button {
                                class: "action-btn",
                                onclick: move |_| search_active_for_open.set(true),
                                img { src: search_icon, class: "action-icon search-icon" }
                            }
                        }
                    }
                    if controller.has_no_search_results() {
                        div { class: "no-results", "No projects matched" }
                    } else {
                        ul {
                            class: "project-list project-list-view",
                            for project in controller.filtered_projects.iter() {
                                ProjectListItem {
                                    controller: controller.clone(),
                                    project: project.clone(),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
