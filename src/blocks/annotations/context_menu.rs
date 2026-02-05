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
    
    // If menu would overflow bottom, position above click point
    let final_y = if y + menu_height > viewport_height - 20.0 {
        (y - menu_height.min(300.0)).max(10.0)
    } else {
        y
    };
    
    // Cap max height based on available space
    let max_h = if final_y < y {
        // Menu is above click - use space from top
        (y - 20.0).min(350.0)
    } else {
        // Menu is below click - use space to bottom
        (viewport_height - y - 20.0).min(350.0)
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
            style: "left: {x}px; top: {final_y}px; max-height: {max_h}px;",
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