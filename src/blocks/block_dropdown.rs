use crate::state::{delete_block, set_current_block, BLOCKS};
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn BlockDropdown(
    x: f64,
    y: f64,
    block_id: String,
    on_close: Signal<Option<(String, f64, f64)>>,
    on_rename: EventHandler<(String, String)>,
) -> Element {
    let nav = navigator();

    // Get the block from BLOCKS using block_id
    let block = BLOCKS
        .read()
        .iter()
        .find(|b| b.block_id == block_id)
        .cloned();

    rsx! {
        div {
            class: "block-dropdown",
            style: "left: {x}px; top: {y}px;",
            onclick: move |e| e.stop_propagation(),

            div {
                class: "block-dropdown-item",
                onclick: {
                    let bid = block_id.clone();
                    move |_| {
                        set_current_block(bid.clone());
                        nav.push(Route::CanvasPage { block_id: bid.clone() });
                        on_close.set(None);
                    }
                },
                "Open"
            }

            div {
                class: "block-dropdown-item",
                onclick: {
                    let bid = block_id.clone();
                    let b = block.clone();
                    move |_| {
                        if let Some(blk) = &b {
                            on_rename.call((bid.clone(), blk.name.clone()));
                        }
                        on_close.set(None);
                    }
                },
                "Rename"
            }

            div {
                class: "block-dropdown-item block-dropdown-item-danger",
                onclick: {
                    let bid = block_id.clone();
                    let b = block.clone();
                    move |_| {
                        if let Some(blk) = &b {
                            let project_id = blk.project_id.clone();
                            let bid = bid.clone();
                            
                            // Call state layer function
                            spawn(async move {
                                delete_block(&project_id, &bid).await;
                            });
                        }
                        on_close.set(None);
                    }
                },
                "Move to trash"
            }
        }
    }
}
