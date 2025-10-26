use dioxus::prelude::*;

#[component]
pub fn ProjectDropdown(
    x: f64,
    y: f64,
    project_id: String,
    on_close: EventHandler<()>,
    on_open: EventHandler<String>,
    on_rename: EventHandler<String>,
    on_delete: EventHandler<String>,
) -> Element {
    let pid_open = project_id.clone();
    let pid_rename = project_id.clone();
    let pid_delete = project_id.clone();
    
    rsx! {
        div {
            class: "project-dropdown",
            style: "left: {x}px; top: {y}px;",
            onclick: move |e| e.stop_propagation(),

            div {
                class: "project-dropdown-item",
                onclick: move |_| {
                    on_open.call(pid_open.clone());
                    on_close.call(());
                },
                "Open"
            }

            div {
                class: "project-dropdown-item",
                onclick: move |_| {
                    on_rename.call(pid_rename.clone());
                    on_close.call(());
                },
                "Rename"
            }

            div {
                class: "project-dropdown-item project-dropdown-item-danger",
                onclick: move |_| {
                    on_delete.call(pid_delete.clone());
                    on_close.call(());
                },
                "Move to trash"
            }
        }
    }
}
