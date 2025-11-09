use crate::canvas::sidebar::types::ClassItem;
use crate::state::{get_current_block_id, BLOCKS};
use dioxus::prelude::*;

#[component]
pub fn ClassRow(
    class_item: ClassItem,
    classes: Signal<Vec<ClassItem>>,
    active_class: Signal<Option<String>>,
) -> Element {
    const MORE: Asset = asset!("/assets/icons/more.svg");

    // Get project_id from current block in global state
    let project_id = get_current_block_id()
        .and_then(|bid| {
            BLOCKS
                .read()
                .iter()
                .find(|b| b.block_id == bid)
                .map(|b| b.project_id.clone())
        })
        .unwrap_or_default();

    let id = class_item.id.clone();
    let is_active = active_class().as_ref().is_some_and(|s| s == &id);
    let zero = class_item.count == 0;

    // Local state for dropdown
    let mut menu_open = use_signal(|| false);

    // Prepare clones for multiple closures
    let id_key = id.clone();
    let id_for_select = id.clone();
    let id_for_color = id.clone();
    let id_for_rename = id.clone();
    let id_for_delete = id.clone();
    let pid_for_select = project_id.clone();
    let pid_for_color = project_id.clone();
    let pid_for_rename = project_id.clone();
    let pid_for_delete = project_id.clone();

    let onclick = move |_| {
        let cur = active_class();
        if cur.as_ref().is_some_and(|s| s == &id_for_select) {
            // Keep it selected (no unassigned state)
            return;
        } else {
            active_class.set(Some(id_for_select.clone()));
        }
    };

    rsx! {
        div {
            key: "{id_key}",
            class: if is_active { "class-row active" } else if zero { "class-row zero" } else { "class-row" },
            onclick: onclick,

            // color dot
            div { class: "class-color", style: format_args!("background: {};", class_item.color) }

            // name + count
            div { class: "class-info",
                span { class: "class-name", "{class_item.name}" }
                span { class: "class-count", "{class_item.count}" }
            }

            // actions: only 'more' menu trigger
            button {
                class: "class-more-button",
                onclick: move |e| { e.stop_propagation(); menu_open.set(!menu_open()); },
                img { class: "class-more-icon", src: "{MORE}", alt: "More" }
            }

            // Dropdown menu
            if menu_open() {
                div { class: "avatar-dropdown-menu class-dropdown-menu",
                    onclick: move |e| { e.stop_propagation(); },

                    button { class: "avatar-dropdown-item", onclick: move |_| {
                        if let Some(new_name) = web_sys::window().and_then(|w| w.prompt_with_message("Rename class:").ok().flatten()) {
                            let mut v = classes.write();
                            if let Some(ci) = v.iter_mut().find(|c| c.id == id_for_rename) { ci.name = new_name; }
                            save_json(&storage_key(&pid_for_rename), &*v);
                        }
                        menu_open.set(false);
                    }, "Rename" }

                    // Change Color using color input
                    div { class: "avatar-dropdown-item class-color-item",
                        input {
                            class: "class-color-picker",
                            r#type: "color",
                            value: "{class_item.color}",
                            oninput: move |e| {
                                let mut v = classes.write();
                                if let Some(ci) = v.iter_mut().find(|c| c.id == id_for_color) {
                                    ci.color = e.value().to_string();
                                }
                                save_json(&storage_key(&pid_for_color), &*v);
                            }
                        }
                        span { "Change Color" }
                    }

                    button { class: "avatar-dropdown-item", onclick: move |_| {
                        let cur = classes();
                        if let Some(ci) = cur.iter().find(|c| c.id == id_for_delete) {
                            if ci.count > 0 {
                                let _ = web_sys::window().and_then(|w| w.alert_with_message("Cannot delete class with annotations. Please reassign first.").ok());
                                return;
                            }
                        }
                        let mut v = classes.write();
                        v.retain(|c| c.id != id_for_delete);
                        save_json(&storage_key(&pid_for_delete), &*v);
                        menu_open.set(false);
                    }, "Delete" }
                }
            }
        }
    }
}
