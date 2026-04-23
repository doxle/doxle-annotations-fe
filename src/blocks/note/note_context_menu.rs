use dioxus::prelude::*;
use crate::core::{THEME, Theme};

const NOTE_CONTEXT_MENU_CSS: &str = include_str!("note_context_menu.css");
const DELETE_ICON_LIGHT: Asset = asset!("/assets/icons/delete-light-block-menu.svg");
const DELETE_ICON_DARK: Asset = asset!("/assets/icons/delete-dark-block-menu.svg");
const EDIT_ICON_LIGHT: Asset = asset!("/assets/icons/edit-light-block-menu.svg");
const EDIT_ICON_DARK: Asset = asset!("/assets/icons/edit-dark-block-menu.svg");

#[component]
pub fn FileContextMenu(
    file_name: String,
    media_type: String,
    pos_x: f64,
    pos_y: f64,
    on_rename: EventHandler<()>,
    on_delete: EventHandler<()>,
    on_close: EventHandler<()>,
) -> Element {
    let is_dark = THEME() == Theme::Dark;
    let delete_icon = if is_dark { DELETE_ICON_DARK } else { DELETE_ICON_LIGHT };
    let edit_icon = if is_dark { EDIT_ICON_DARK } else { EDIT_ICON_LIGHT };
    let dropdown_style = format!("left:{}px;top:{}px;", pos_x, pos_y);

    rsx! {
        style { {NOTE_CONTEXT_MENU_CSS} }
        div {
            class: "note-context-menu-overlay",
            onclick: move |e| {
                e.stop_propagation();
                on_close.call(());
            }
        }
        div {
            class: "note-context-menu-dropdown",
            style: "{dropdown_style}",
            onclick: move |e| e.stop_propagation(),
            div {
                class: "note-context-menu-info",
                div { class: "note-context-menu-info-label", "File name" }
                div { "{file_name}" }
            }
            div { class: "note-context-menu-divider" }
            div {
                class: "note-context-menu-item",
                onclick: move |e| {
                    e.stop_propagation();
                    on_rename.call(());
                },
                img { src: edit_icon, class: "note-context-menu-icon", alt: "Rename" }
                "Rename"
            }
            div {
                class: "note-context-menu-item",
                onclick: move |e| {
                    e.stop_propagation();
                    on_delete.call(());
                },
                img { src: delete_icon, class: "note-context-menu-icon", alt: "Delete" }
                "Delete"
            }
        }
    }
}
