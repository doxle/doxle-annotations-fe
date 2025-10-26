use dioxus::prelude::*;
use crate::api::blocks::Block;
use crate::state::set_current_block;
use crate::Route;

#[component]
pub fn BlockRow(
    index: usize,
    block: Block,
    on_context_menu: EventHandler<(String, f64, f64)>,
) -> Element {
    let nav = navigator();
    
    let block_id_for_context = block.block_id.clone();
    let block_id_for_click = block.block_id.clone();
    let block_name = block.name.clone();
    let block_assigned = block.assigned_to.clone();
    let block_created = block.created_at.clone();
    
    rsx! {
        div {
            key: "{index}",
            class: "block-row",
            oncontextmenu: move |e| {
                e.prevent_default();
                let client_x = e.client_coordinates().x;
                let client_y = e.client_coordinates().y;
                on_context_menu.call((block_id_for_context.clone(), client_x, client_y));
            },
            onclick: move |e| {
                e.stop_propagation();
                set_current_block(block_id_for_click.clone());
                nav.push(Route::CanvasPage { block_id: block_id_for_click.clone() });
            },

            // Name column with image
            div {
                class: "block-name-col",

                // Colored rectangle placeholder for image
                div {
                    class: "block-image-placeholder"
                }

                span {
                    class: "block-name-text",
                    "{block_name}"
                }
            }

            // Progress column (placeholder for now)
            div {
                class: "block-progress",
                "[0/0]"
            }

            // Assigned column
            div {
                class: "block-assigned",
                if let Some(assigned) = &block_assigned {
                    "{assigned}"
                } else {
                    "Unassigned"
                }
            }

            // Last Modified column
            div {
                class: "block-modified",
                "{block_created}"
            }
        }
    }
}