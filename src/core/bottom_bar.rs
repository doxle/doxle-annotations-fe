use dioxus::prelude::*;
use crate::Route;
use crate::core::{THEME, Theme};

const ANNOTATION_BLOCK_LIGHT: Asset = asset!("/assets/icons/annotation-block-light.svg");
const ANNOTATION_BLOCK_DARK: Asset = asset!("/assets/icons/annotation-block-dark.svg");
const FILE_BLOCK_LIGHT: Asset = asset!("/assets/icons/file-block-light.svg");
const FILE_BLOCK_DARK: Asset = asset!("/assets/icons/file-block-dark.svg");
const BUILD_BLOCK_LIGHT: Asset = asset!("/assets/icons/build-block-light.svg");
const BUILD_BLOCK_DARK: Asset = asset!("/assets/icons/build-block-dark.svg");

/// Which block-type filter is active in the bottom bar.
/// "all" | "file" | "building" | "annotation"
pub static BLOCK_TYPE_FILTER: GlobalSignal<String> = Signal::global(|| "all".to_string());

#[component]
pub fn BottomBar(project_id: String) -> Element {
    let route = use_route::<Route>();
    let nav = use_navigator();
    let is_dark = THEME() == Theme::Dark;
    let current_filter = BLOCK_TYPE_FILTER();

    // Determine active tab from current route context
    let active_tab = match &route {
        Route::BlocksPage { .. } | Route::CreateBlockPage { .. } => current_filter.as_str(),
        Route::TasksListPage { block_type, .. }
        | Route::CreateTaskPage { block_type, .. }
        | Route::AnnotationCanvasPage { block_type, .. } => block_type.as_str(),
        Route::FileBlockPage { .. } | Route::FileItemPage { .. } => "file",
        Route::BuildingBlockPage { .. } => "building",
        Route::ImportBlockPage { block_type, .. } => block_type.as_str(),
        _ => "all",
    };

    let tasks_icon = if is_dark { ANNOTATION_BLOCK_DARK } else { ANNOTATION_BLOCK_LIGHT };
    let files_icon = if is_dark { FILE_BLOCK_DARK } else { FILE_BLOCK_LIGHT };
    let build_icon = if is_dark { BUILD_BLOCK_DARK } else { BUILD_BLOCK_LIGHT };

    // Each tab navigates to BlocksPage with a type filter
    let make_handler = move |filter: &'static str| {
        let pid = project_id.clone();
        let nav = nav.clone();
        move |_| {
            *BLOCK_TYPE_FILTER.write() = filter.to_string();
            nav.push(Route::BlocksPage { project_id: pid.clone() });
        }
    };

    let nav_files = make_handler("file");
    let nav_build = make_handler("building");
    let nav_tasks = make_handler("annotation");

    rsx! {
        div { class: "app-bottom-bar",
            div {
                class: if active_tab == "file" { "bottom-bar-tab active" } else { "bottom-bar-tab" },
                onclick: nav_files,
                img { src: files_icon, class: "bottom-bar-tab-icon", alt: "Files" }
            }
            div {
                class: if active_tab == "building" { "bottom-bar-tab active" } else { "bottom-bar-tab" },
                onclick: nav_build,
                img { src: build_icon, class: "bottom-bar-tab-icon", alt: "Build" }
            }
            div {
                class: if active_tab == "annotation" { "bottom-bar-tab active" } else { "bottom-bar-tab" },
                onclick: nav_tasks,
                img { src: tasks_icon, class: "bottom-bar-tab-icon", alt: "Tasks" }
            }
        }
    }
}
