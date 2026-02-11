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

    // Calculate menu height estimate (items + padding + delete)
    let menu_height = (labels.len() as f64 * 32.0) + 80.0;
    
    // Get viewport height
    let viewport_height = web_sys::window()
        .and_then(|w| Some(w.inner_height().ok()?.as_f64()?))
        .unwrap_or(800.0);
    
    // Anchor at click point, pick direction with more space
    let space_below = viewport_height - y - 20.0;
    let space_above = y - 20.0;
    let grow_up = space_above > space_below;
    let max_h = (if grow_up { space_above } else { space_below }).min(350.0);

    let pos_style = if grow_up {
        let bottom = viewport_height - y;
        format!("left: {x}px; bottom: {bottom}px; max-height: {max_h}px;")
    } else {
        format!("left: {x}px; top: {y}px; max-height: {max_h}px;")
    };

	rsx!{
		style { {CSS} }
         // Backdrop - covers entire screen, catches outside clicks
        div {
            class: "context-menu-backdrop",
            onclick: move |_| on_close.call(()),
        }
        
		div{
			class: "annotation-context-menu",
            style: "{pos_style}",
            onclick: move |evt| evt.stop_propagation(),

            // Scrollable label options
            div {
                class: "context-menu-labels",
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
            }

            // Divider
            div { class: "context-menu-divider" }
            
            // Delete (always visible at bottom)
            div {
                class: "context-menu-item delete",
                onclick: move |_| on_delete.call(()),
                img { src: trash_icon, class: "trash-icon", alt: "Delete" }
                span { "Delete" }
            }
		}
	}

}