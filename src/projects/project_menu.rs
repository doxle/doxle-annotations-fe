use dioxus::prelude::*;
use crate::core::{THEME, Theme};

const PROJECT_MENU_CSS: &str = include_str!("project_menu.css");
const DELETE_ICON_LIGHT: Asset = asset!("/assets/icons/delete-light-project-menu.svg");
const DELETE_ICON_DARK: Asset = asset!("/assets/icons/delete-dark-project-menu.svg");
const COPY_ICON_LIGHT: Asset = asset!("/assets/icons/copy-light.svg");
const COPY_ICON_DARK: Asset = asset!("/assets/icons/copy-dark.svg");
const OPEN_ICON_LIGHT: Asset = asset!("/assets/icons/open-light.svg");
const OPEN_ICON_DARK: Asset = asset!("/assets/icons/open-dark.svg");
const RENAME_ICON_LIGHT: Asset = asset!("/assets/icons/rename-light.svg");
const RENAME_ICON_DARK: Asset = asset!("/assets/icons/rename-dark.svg");

#[component]
pub fn ProjectMenu(
    project_id: String,
    is_open: bool,
    on_toggle: EventHandler<()>,
    on_rename: EventHandler<()>,
    on_open_new_tab: EventHandler<()>,
    on_copy_link: EventHandler<()>,
    on_delete: EventHandler<()>,
) -> Element {
    let is_dark = THEME() == Theme::Dark;
    let rename_icon = if is_dark { RENAME_ICON_DARK } else { RENAME_ICON_LIGHT };
    let copy_icon = if is_dark { COPY_ICON_DARK } else { COPY_ICON_LIGHT };
    let open_icon = if is_dark { OPEN_ICON_DARK } else { OPEN_ICON_LIGHT };
    let delete_icon = if is_dark { DELETE_ICON_DARK } else { DELETE_ICON_LIGHT };
    let mut search_query = use_signal(String::new);
    let query = search_query().trim().to_lowercase();
    let matches = |label: &str| query.is_empty() || label.to_lowercase().contains(&query);
    let show_rename = matches("Rename");
    let show_open_new_tab = matches("Open in new tab");
    let show_copy_link = matches("Copy link");
    let show_delete = matches("Delete");
    let first_group_has_items = show_rename;
    let second_group_has_items = show_open_new_tab || show_copy_link;
    let show_first_divider = first_group_has_items && (second_group_has_items || show_delete);
    let show_second_divider = second_group_has_items && show_delete;
    let show_any_items = first_group_has_items || second_group_has_items || show_delete;
    rsx! {
        style { {PROJECT_MENU_CSS} }
        div {
            class: "project-menu-container",
            "data-project-id": "{project_id}",
            span {
                class: "project-menu-trigger",
                onclick: move |e| {
                    e.stop_propagation();
                    on_toggle.call(());
                },
                "⋮"
            }
            if is_open {
                div {
                    class: "project-menu-overlay",
                    onclick: move |e| {
                        e.stop_propagation();
                        on_toggle.call(());
                    }
                }
                div {
                    class: "project-menu-dropdown",
                    onclick: move |e| e.stop_propagation(),
                    input {
                        class: "project-menu-search",
                        r#type: "text",
                        placeholder: "Search actions...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value()),
                        onclick: move |e| e.stop_propagation(),
                    }
                    // div { class: "project-menu-title", "Project Menu" }
                    if show_rename {
                        div {
                            class: "project-menu-item",
                            onclick: move |e| {
                                e.stop_propagation();
                                on_rename.call(());
                            },
                            img {
                                src: rename_icon,
                                class: "project-menu-icon",
                                alt: "Rename"
                            }
                            "Rename"
                        }
                    }
                    if show_first_divider {
                        div { class: "project-menu-divider" }
                    }
                    if show_open_new_tab {
                        div {
                            class: "project-menu-item",
                            onclick: move |e| {
                                e.stop_propagation();
                                on_open_new_tab.call(());
                            },
                            img {
                                src: open_icon,
                                class: "project-menu-icon",
                                alt: "Open in new tab"
                            }
                            "Open in new tab"
                        }
                    }
                    if show_copy_link {
                        div {
                            class: "project-menu-item",
                            onclick: move |e| {
                                e.stop_propagation();
                                on_copy_link.call(());
                            },
                            img {
                                src: copy_icon,
                                class: "project-menu-icon",
                                alt: "Copy link"
                            }
                            "Copy link"
                        }
                    }
                    if show_second_divider {
                        div { class: "project-menu-divider" }
                    }
                    if show_delete {
                        div {
                            class: "project-menu-item project-menu-item-delete",
                            onclick: move |e| {
                                e.stop_propagation();
                                on_delete.call(());
                            },
                            img {
                                src: delete_icon,
                                class: "project-menu-icon project-menu-icon-delete",
                                alt: "Delete"
                            }
                            "Delete"
                        }
                    }
                    if !show_any_items {
                        div { class: "project-menu-empty", "No actions found" }
                    }
                }
            }
        }
    }
}
