use dioxus::prelude::*;
use crate::state::{self, get_project_by_id};
use crate::Route;

#[component]
pub fn ProjectDropdown(
    x: f64,
    y: f64,
    project_id: String,
    on_close: Signal<Option<(String, f64, f64)>>,
    show_rename_modal: Signal<bool>,
    rename_project_id: Signal<String>,
    rename_project_name: Signal<String>,
    show_share_modal: Signal<bool>,
    share_project_id: Signal<String>,
) -> Element {
    let nav = navigator();
    
    rsx! {
        div {
            class: "project-dropdown",
            style: "left: {x}px; top: {y}px;",
            onclick: move |e| e.stop_propagation(),

            div {
                class: "project-dropdown-item",
                onclick: {
                    let pid = project_id.clone();
                    move |_| {
                        nav.push(Route::BlocksPage { project_id: pid.clone() });
                        on_close.set(None);
                    }
                },
                "Open"
            }

            div {
                class: "project-dropdown-item",
                onclick: {
                    let pid = project_id.clone();
                    move |_| {
                        if let Some(project) = get_project_by_id(&pid) {
                            *rename_project_id.write() = pid.clone();
                            *rename_project_name.write() = project.name;
                            *show_rename_modal.write() = true;
                        }
                        on_close.set(None);
                    }
                },
                "Rename"
            }

            div {
                class: "project-dropdown-item",
                onclick: {
                    let pid = project_id.clone();
                    move |_| {
                        *share_project_id.write() = pid.clone();
                        *show_share_modal.write() = true;
                        on_close.set(None);
                    }
                },
                "Share"
            }

            div {
                class: "project-dropdown-item project-dropdown-item-danger",
                onclick: {
                    let pid = project_id.clone();
                    move |_| {
                        let id = pid.clone();
                        on_close.set(None);
                        
                        // Use wasm_bindgen_futures to spawn a persistent task
                        wasm_bindgen_futures::spawn_local(async move {
                            state::delete_project(id).await;
                        });
                    }
                },
                "Move to trash"
            }
        }
    }
}
