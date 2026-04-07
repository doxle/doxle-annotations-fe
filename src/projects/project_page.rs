use dioxus::prelude::*;
use crate::Route;
use crate::core::{is_mobile, AppNavbar, LoadingScreen, ProtectedRoute, THEME, Theme};
use crate::users::state::USER;
use super::api::{api_delete_project, Project};
use super::project_desktop::ProjectsDesktop;
use super::project_mobile::ProjectsMobile;
use super::project_menu::ProjectMenu;
use super::project_state::{PROJECTS, PROJECTS_LOADING, state_load_projects, state_rename_project};

pub(crate) const PROJECTS_CSS: &str = include_str!("project.css");
pub(crate) const ADD_ICON_LIGHT: Asset = asset!("/assets/icons/add-light.svg");
pub(crate) const ADD_ICON_DARK: Asset = asset!("/assets/icons/add-dark.svg");
pub(crate) const LIST_ICON_LIGHT: Asset = asset!("/assets/icons/list-light.svg");
pub(crate) const LIST_ICON_DARK: Asset = asset!("/assets/icons/list-dark.svg");
pub(crate) const GRID_ICON_LIGHT: Asset = asset!("/assets/icons/grid-light.svg");
pub(crate) const GRID_ICON_DARK: Asset = asset!("/assets/icons/grid-dark.svg");
pub(crate) const SEARCH_ICON_LIGHT: Asset = asset!("/assets/icons/search-light.svg");
pub(crate) const SEARCH_ICON_DARK: Asset = asset!("/assets/icons/search-dark.svg");
pub(crate) const CLOSE_ICON_LIGHT: Asset = asset!("/assets/icons/close-light.svg");
pub(crate) const CLOSE_ICON_DARK: Asset = asset!("/assets/icons/close-dark.svg");

pub(crate) fn format_date(rfc3339: &str) -> String {
    if rfc3339.is_empty() { return "—".to_string(); }
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(rfc3339));
    let months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    let month = months[date.get_month() as usize];
    format!("{} {}, {}", month, date.get_date(), date.get_full_year())
}

pub(crate) fn format_relative_time(rfc3339: &str) -> String {
    if rfc3339.is_empty() { return "—".to_string(); }
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(rfc3339));
    let now = js_sys::Date::new_0();
    let diff_secs = ((now.get_time() - date.get_time()) / 1000.0) as u64;
    match diff_secs {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{} min ago", diff_secs / 60),
        3600..=86399 => format!("{} hours ago", diff_secs / 3600),
        86400..=2591999 => format!("{} days ago", diff_secs / 86400),
        _ => format_date(rfc3339),
    }
}

#[derive(Clone, PartialEq)]
pub struct ProjectsController {
    pub is_admin: bool,
    pub is_dark: bool,
    pub projects: Vec<Project>,
    pub filtered_projects: Vec<Project>,
    pub open_menu_id: Signal<Option<String>>,
    pub renaming_project: Signal<Option<(String, String)>>,
    pub rename_input: Signal<String>,
    pub view_mode: Signal<String>,
    pub search_active: Signal<bool>,
    pub search_query: Signal<String>,
}

impl ProjectsController {
    pub fn is_list_mode(&self) -> bool {
        let view_mode = self.view_mode;
        view_mode() == "list"
    }

    pub fn has_no_search_results(&self) -> bool {
        let search_active = self.search_active;
        self.filtered_projects.is_empty() && search_active()
    }
}

pub(crate) fn use_projects_controller() -> ProjectsController {
    let is_admin = USER.read().as_ref().map(|u| u.is_admin()).unwrap_or(false);
    let is_dark = THEME() == Theme::Dark;
    let open_menu_id: Signal<Option<String>> = use_signal(|| None);
    let renaming_project: Signal<Option<(String, String)>> = use_signal(|| None);
    let rename_input = use_signal(String::new);
    let view_mode: Signal<String> = use_signal(|| "list".to_string());
    let search_active: Signal<bool> = use_signal(|| false);
    let search_query = use_signal(String::new);

    use_resource(move || async move {
        if PROJECTS.peek().is_empty() {
            state_load_projects().await;
        }
    });

    let projects = PROJECTS.read().clone();
    let filtered_projects = if search_query().is_empty() {
        projects.clone()
    } else {
        let query = search_query().to_lowercase();
        projects
            .iter()
            .filter(|project| project.project_name.to_lowercase().contains(&query))
            .cloned()
            .collect()
    };

    ProjectsController {
        is_admin,
        is_dark,
        projects,
        filtered_projects,
        open_menu_id,
        renaming_project,
        rename_input,
        view_mode,
        search_active,
        search_query,
    }
}

pub(crate) fn close_project_menu(controller: &ProjectsController) {
    let mut open_menu_id = controller.open_menu_id;
    if open_menu_id().is_some() {
        open_menu_id.set(None);
    }
}

pub(crate) fn toggle_project_menu(controller: &ProjectsController, project_id: String) {
    let mut open_menu_id = controller.open_menu_id;
    if open_menu_id() == Some(project_id.clone()) {
        open_menu_id.set(None);
    } else {
        open_menu_id.set(Some(project_id));
    }
}

pub(crate) fn begin_project_rename(controller: &ProjectsController, project_id: String, current_name: String) {
    let mut renaming_project = controller.renaming_project;
    let mut rename_input = controller.rename_input;
    let mut open_menu_id = controller.open_menu_id;
    renaming_project.set(Some((project_id, current_name.clone())));
    rename_input.set(current_name);
    open_menu_id.set(None);
}

pub(crate) fn cancel_project_rename(controller: &ProjectsController) {
    let mut renaming_project = controller.renaming_project;
    renaming_project.set(None);
}

pub(crate) fn submit_project_rename(controller: ProjectsController, project_id: String, current_name: String) {
    let rename_input = controller.rename_input;
    let next_name = rename_input().trim().to_string();
    if next_name.is_empty() {
        crate::core::progress::show_error("Project name is required");
        return;
    }
    if next_name == current_name {
        cancel_project_rename(&controller);
        return;
    }
    spawn(async move {
        match state_rename_project(&project_id, next_name).await {
            Ok(_) => crate::core::progress::show_success("Project renamed"),
            Err(e) => crate::core::progress::show_error_persistent(&format!("Failed to rename project: {}", e)),
        }
    });
    cancel_project_rename(&controller);
}

pub(crate) fn open_project_in_new_tab(project_id: &str) {
    if let Some(window) = web_sys::window() {
        let path = format!("/projects/{}/blocks", project_id);
        let target = match window.location().origin() {
            Ok(origin) => format!("{}{}", origin, path),
            Err(_) => path,
        };
        let _ = window.open_with_url_and_target(&target, "_blank");
    }
}

pub(crate) fn copy_project_link(project_id: &str) {
    if let Some(window) = web_sys::window() {
        let path = format!("/projects/{}/blocks", project_id);
        let full_url = match window.location().origin() {
            Ok(origin) => format!("{}{}", origin, path),
            Err(_) => path,
        };
        let escaped = full_url.replace('\\', "\\\\").replace('\"', "\\\"");
        document::eval(&format!("navigator.clipboard && navigator.clipboard.writeText(\"{}\");", escaped));
        crate::core::progress::show_info("Project link copied");
    }
}

pub(crate) fn delete_project(controller: &ProjectsController, project_id: String) {
    let project_id_for_delete = project_id.clone();
    spawn(async move {
        match api_delete_project(&project_id_for_delete).await {
            Ok(_) => {
                PROJECTS.write().retain(|project| project.project_id != project_id_for_delete);
                crate::core::progress::show_success("Project deleted");
            }
            Err(e) => crate::core::progress::show_error_persistent(&format!("Failed to delete project: {}", e)),
        }
    });
    let mut open_menu_id = controller.open_menu_id;
    open_menu_id.set(None);
}

#[component]
pub(crate) fn ProjectListItem(controller: ProjectsController, project: Project) -> Element {
    let navigator = use_navigator();
    let project_id = project.project_id.clone();
    let project_name = project.project_name.clone();
    let project_id_for_open = project_id.clone();
    let mut open_menu_id_for_open = controller.open_menu_id;
    let open_menu_id_for_menu = controller.open_menu_id;

    let project_id_for_ctx = project_id.clone();
    rsx! {
        li {
            key: "{project_id}",
            class: "project-card",
            oncontextmenu: move |e| {
                e.prevent_default();
                e.stop_propagation();
                open_menu_id_for_open.set(Some(project_id_for_ctx.clone()));
            },
            onclick: move |_| {
                if open_menu_id_for_open().is_some() {
                    open_menu_id_for_open.set(None);
                    return;
                }
                navigator.push(Route::BlocksPage { project_id: project_id_for_open.clone() });
            },
            div { class: "project-card-meta", "#{project.block_count} BLOCKS" }
            div { class: "project-card-name", "{project_name}" }
            div {
                class: "project-card-actions",
                {
                    let controller_for_toggle = controller.clone();
                    let controller_for_rename = controller.clone();
                    let controller_for_open_new_tab = controller.clone();
                    let controller_for_copy_link = controller.clone();
                    let controller_for_delete = controller.clone();
                    let project_id_for_toggle = project.project_id.clone();
                    let project_id_for_open_new_tab = project.project_id.clone();
                    let project_id_for_copy_link = project.project_id.clone();
                    let project_id_for_rename = project.project_id.clone();
                    let project_name_for_rename = project.project_name.clone();
                    let project_id_for_delete = project.project_id.clone();
                    rsx! {
                        ProjectMenu {
                            project_id: project_id_for_toggle.clone(),
                            is_open: open_menu_id_for_menu() == Some(project_id_for_toggle.clone()),
                            on_toggle: move |_| toggle_project_menu(&controller_for_toggle, project_id_for_toggle.clone()),
                            on_rename: move |_| begin_project_rename(&controller_for_rename, project_id_for_rename.clone(), project_name_for_rename.clone()),
                            on_open_new_tab: move |_| {
                                open_project_in_new_tab(&project_id_for_open_new_tab);
                                close_project_menu(&controller_for_open_new_tab);
                            },
                            on_copy_link: move |_| {
                                copy_project_link(&project_id_for_copy_link);
                                close_project_menu(&controller_for_copy_link);
                            },
                            on_delete: move |_| delete_project(&controller_for_delete, project_id_for_delete.clone()),
                        }
                    }
                }
            }
            div {
                class: "project-card-dates",
                div {
                    class: "project-dates",
                    div { class: "project-date", "{format_date(&project.project_created_at)}" }
                    div { class: "project-date", "{format_relative_time(&project.project_updated_at)}" }
                }
            }
        }
    }
}

#[component]
fn ProjectsRenameModal(controller: ProjectsController) -> Element {
    let renaming_project = controller.renaming_project;
    let rename_input = controller.rename_input;

    if let Some((project_id, current_name)) = renaming_project() {
        let project_id_for_enter = project_id.clone();
        let current_name_for_enter = current_name.clone();
        let controller_for_overlay = controller.clone();
        let controller_for_enter = controller.clone();
        let controller_for_cancel = controller.clone();
        let controller_for_save = controller.clone();
        let mut rename_input_for_change = rename_input;

        rsx! {
            div {
                class: "create-project-overlay",
                onclick: move |_| cancel_project_rename(&controller_for_overlay),
                div {
                    class: "create-project-modal",
                    onclick: move |e| e.stop_propagation(),
                    input {
                        class: "create-project-input",
                        value: "{rename_input}",
                        oninput: move |e| rename_input_for_change.set(e.value()),
                        onkeypress: move |e| {
                            if e.key() == Key::Enter {
                                submit_project_rename(controller_for_enter.clone(), project_id_for_enter.clone(), current_name_for_enter.clone());
                            }
                        },
                        placeholder: "{current_name}",
                        autofocus: true,
                    }
                    div {
                        class: "create-project-actions",
                        button {
                            class: "create-project-btn create-project-btn-secondary",
                            onclick: move |_| cancel_project_rename(&controller_for_cancel),
                            "Cancel"
                        }
                        button {
                            class: "create-project-btn create-project-btn-primary",
                            onclick: move |_| submit_project_rename(controller_for_save.clone(), project_id.clone(), current_name.clone()),
                            "Save"
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

#[component]
pub fn ProjectsPage() -> Element {
    let controller = use_projects_controller();

    rsx! {
        ProtectedRoute {
            AppNavbar {}
            if PROJECTS_LOADING() {
                LoadingScreen { text: "Loading projects".to_string() }
            } else {
                style { {PROJECTS_CSS} }
                if is_mobile() {
                    ProjectsMobile { controller: controller.clone() }
                } else {
                    ProjectsDesktop { controller: controller.clone() }
                }
                ProjectsRenameModal { controller: controller.clone() }
            }
        }
    }
}
