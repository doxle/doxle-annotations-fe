use dioxus::prelude::*;
use crate::blocks::block_list::api::BlockLabel;
use crate::shell::{THEME, Theme};

const CSS: &str = include_str!("context_menu.css");
const TRASH_ICON_LIGHT: Asset = asset!("/assets/icons/delete-light.svg");
const TRASH_ICON_DARK: Asset = asset!("/assets/icons/delete-dark.svg");

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
	let trash_icon = TRASH_ICON_DARK;

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

    let mut search_query = use_signal(|| String::new());

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

            // Search input
            input {
                class: "context-menu-search",
                r#type: "text",
                placeholder: "Search labels...",
                autofocus: true,
                value: "{search_query}",
                oninput: move |evt| search_query.set(evt.value().clone()),
            }

            // Scrollable label options
            div {
                class: "context-menu-labels",
                {
                    let q = search_query().to_lowercase();
                    let filtered: Vec<_> = labels.iter()
                        .filter(|l| q.is_empty() || l.label_name.to_lowercase().contains(&q))
                        .collect();
                    rsx! {
                        for label in filtered {
                            {
                                let is_current = label.label_id == current_label_id;
                                rsx! {
                                    div {
                                        class: if is_current { "context-menu-item current" } else { "context-menu-item" },
                                        onclick: {
                                            let lid = label.label_id.clone();
                                            move |_| on_change_label.call(lid.clone())
                                        },
                                        span {
                                            class: "label-color-square",
                                            style: "background: {label.label_color};",
                                        }
                                        span { "{label.label_name}" }
                                    }
                                }
                            }
                        }
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