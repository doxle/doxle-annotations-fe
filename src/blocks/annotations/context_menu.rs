use dioxus::prelude::*;
use crate::blocks::dashboard::api::BlockLabel;
use crate::shell::{THEME, Theme};

const CSS: &str = include_str!("context_menu.css");
const TRASH_ICON_LIGHT: Asset = asset!("/assets/icons/trash-light.svg");
const TRASH_ICON_DARK: Asset = asset!("/assets/icons/trash-dark.svg");

#[component]
pub fn AnnotationContextMenu(
	x:f64,
	y:f64,
	current_label_id:String,
	labels:Vec<BlockLabel>,
	on_change_label:EventHandler<String>,
	on_delete:EventHandler<()>,
	on_close:EventHandler<()>
	)->Element{

let is_dark = THEME() == Theme::Dark;
	let trash_icon = if is_dark { TRASH_ICON_DARK } else { TRASH_ICON_LIGHT };

	rsx!{
		style { {CSS} }
         // Backdrop - covers entire screen, catches outside clicks
        div {
            class: "context-menu-backdrop",
            onclick: move |_| on_close.call(()),
        }
        
		div{
			class: "annotation-context-menu",
            style: "left: {x}px; top: {y}px;",
            onclick: move |evt| evt.stop_propagation(),

             // Label options
            for label in labels {
                div {
                    class: "context-menu-item",
                    onclick: {
                        let lid = label.label_id.clone();
                        move |_| on_change_label.call(lid.clone())
                    },
                    if label.label_id == current_label_id {
                        span { class: "radio-icon", "●" }
                    } else {
                        span { class: "radio-icon", "○" }
                    }
                    span { "{label.label_name}" }
                }
            }

            // Divider
            div { class: "context-menu-divider" }
            
            // Delete
            div {
                class: "context-menu-item delete",
                onclick: move |_| on_delete.call(()),
                img { src: trash_icon, class: "trash-icon", alt: "Delete" }
                span { "Delete" }
            }
		}
	}

}