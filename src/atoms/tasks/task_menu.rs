use dioxus::prelude::*;
use crate::shell::{THEME, Theme};

const TASK_MENU_CSS: &str = include_str!("task_menu.css");
const DELETE_ICON_LIGHT: Asset = asset!("/assets/icons/delete-light.svg");
const DELETE_ICON_DARK: Asset = asset!("/assets/icons/delete-dark.svg");
const EDIT_ICON_LIGHT: Asset = asset!("/assets/icons/edit-light.svg");
const EDIT_ICON_DARK: Asset = asset!("/assets/icons/edit-dark.svg");
const ARCHIVE_ICON_LIGHT: Asset = asset!("/assets/icons/archive-light.svg");
const ARCHIVE_ICON_DARK: Asset = asset!("/assets/icons/archive-dark.svg");

#[component]
pub fn TaskMenu(
    task_id: String,
    is_open: bool,
    on_toggle: EventHandler<()>,
    on_edit: EventHandler<()>,
    on_archive: EventHandler<()>,
    on_delete: EventHandler<()>,
) -> Element {
    let is_dark = THEME() == Theme::Dark;
    let delete_icon = if is_dark { DELETE_ICON_DARK } else { DELETE_ICON_LIGHT };
    let edit_icon = if is_dark { EDIT_ICON_DARK } else { EDIT_ICON_LIGHT };
    let archive_icon = if is_dark { ARCHIVE_ICON_DARK } else { ARCHIVE_ICON_LIGHT };

    rsx! {
        style { {TASK_MENU_CSS} }
        div {
            class: "task-menu-container",
            span {
                class: "task-menu-trigger",
                onclick: move |e| {
                    e.stop_propagation();
                    on_toggle.call(());
                },
                "⋮"
            }
            if is_open {
                div {
                    class: "task-menu-overlay",
                    onclick: move |e| {
                        e.stop_propagation();
                        on_toggle.call(());
                    }
                }
                div {
                    class: "task-menu-dropdown",
                    onclick: move |e| e.stop_propagation(),
                    div {
                        class: "task-menu-item task-menu-item-edit",
                        onclick: move |e| {
                            e.stop_propagation();
                            on_edit.call(());
                        },
                        img {
                            src: edit_icon,
                            class: "task-menu-icon",
                            alt: "Edit"
                        }
                        "Edit task name"
                    }
                    div {
                        class: "task-menu-item task-menu-item-archive",
                        onclick: move |e| {
                            e.stop_propagation();
                            on_archive.call(());
                        },
                        img {
                            src: archive_icon,
                            class: "task-menu-icon",
                            alt: "Archive"
                        }
                        "Archive task"
                    }
                    div { class: "task-menu-divider" }
                    div {
                        class: "task-menu-item task-menu-item-delete",
                        onclick: move |e| {
                            e.stop_propagation();
                            on_delete.call(());
                        },
                        img {
                            src: delete_icon,
                            class: "task-menu-icon",
                            alt: "Delete"
                        }
                        "Delete task"
                    }
                }
            }
        }
    }
}
