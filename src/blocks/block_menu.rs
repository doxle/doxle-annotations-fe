use dioxus::prelude::*;
use crate::core::{THEME, Theme};

const BLOCK_MENU_CSS: &str = include_str!("block_menu.css");
const DELETE_ICON_LIGHT: Asset = asset!("/assets/icons/delete-light-block-menu.svg");
const DELETE_ICON_DARK: Asset = asset!("/assets/icons/delete-dark-block-menu.svg");
const EDIT_ICON_LIGHT: Asset = asset!("/assets/icons/edit-light-block-menu.svg");
const EDIT_ICON_DARK: Asset = asset!("/assets/icons/edit-dark-block-menu.svg");
const ARCHIVE_ICON_LIGHT: Asset = asset!("/assets/icons/archive-light-block-menu.svg");
const ARCHIVE_ICON_DARK: Asset = asset!("/assets/icons/archive-dark-block-menu.svg");

#[derive(Clone, PartialEq)]
pub struct MenuItem {
    pub label: String,
    pub is_danger: bool,
}

#[component]
pub fn BlockMenu(
    block_id: String,
    block_type: String,
    is_open: bool,
    on_toggle: EventHandler<()>,
    on_edit: EventHandler<()>,
    on_archive: EventHandler<()>,
    on_import: EventHandler<()>,
    on_export: EventHandler<()>,
    on_reconcile: EventHandler<()>,
    on_delete: EventHandler<()>,
) -> Element {
    let is_dark = THEME() == Theme::Dark;
    let delete_icon = if is_dark { DELETE_ICON_DARK } else { DELETE_ICON_LIGHT };
    let edit_icon = if is_dark { EDIT_ICON_DARK } else { EDIT_ICON_LIGHT };
    let archive_icon = if is_dark { ARCHIVE_ICON_DARK } else { ARCHIVE_ICON_LIGHT };
    


    rsx! {
        style { {BLOCK_MENU_CSS} }
        div {
            class: "block-menu-container",
            span {
                class: "block-menu-trigger",
                onclick: move |e| {
                    e.stop_propagation();
                    on_toggle.call(());
                },
                "⋮"
            }
            if is_open {
                div {
                    class: "block-menu-overlay",
                    onclick: move |e| {
                        e.stop_propagation();
                        on_toggle.call(());
                    }
                }
                div {
                    class: "block-menu-dropdown",
                    onclick: move |e| e.stop_propagation(),
                    div {
                        class: "block-menu-item block-menu-item-edit",
                        onclick: move |e| {
                            e.stop_propagation();
                            on_edit.call(());
                        },
                        img {
                            src: edit_icon,
                            class: "block-menu-icon",
                            alt: "Edit"
                        }
                        "Edit block name"
                    }
                    div {
                        class: "block-menu-item block-menu-item-archive",
                        onclick: move |e| {
                            e.stop_propagation();
                            on_archive.call(());
                        },
                        img {
                            src: archive_icon,
                            class: "block-menu-icon",
                            alt: "Archive"
                        }
                        "Archive block"
                    }
                    if block_type == "annotation" {
                        div {
                            class: "block-menu-item block-menu-item-edit",
                            onclick: move |e| {
                                e.stop_propagation();
                                on_import.call(());
                            },
                            img {
                                src: edit_icon,
                                class: "block-menu-icon",
                                alt: "Import"
                            }
                            "Import block"
                        }
                        div {
                            class: "block-menu-item block-menu-item-edit",
                            onclick: move |e| {
                                e.stop_propagation();
                                on_export.call(());
                            },
                            img {
                                src: archive_icon,
                                class: "block-menu-icon",
                                alt: "Export"
                            }
                            "Export block"
                        }
                        div {
                            class: "block-menu-item block-menu-item-edit",
                            onclick: move |e| {
                                e.stop_propagation();
                                on_reconcile.call(());
                            },
                            img {
                                src: edit_icon,
                                class: "block-menu-icon",
                                alt: "Reconcile"
                            }
                            "Reconcile counts"
                        }
                    }
                    div { class: "block-menu-divider" }
                    div {
                        class: "block-menu-item block-menu-item-delete",
                        onclick: move |e| {
                            e.stop_propagation();
                            on_delete.call(());
                        },
                        img {
                            src: delete_icon,
                            class: "block-menu-icon",
                            alt: "Delete"
                        }
                        "Delete block"
                    }
                }
            }
        }
    }
}