use dioxus::prelude::*;
use crate::state::{BLOCKS, delete_block, set_current_block};
use crate::Route;

#[component]
pub fn BlockDropdown(
    x: f64,
    y: f64,
    block_id: String,
    on_close: Signal<Option<(String, f64, f64)>>,
) -> Element {
    let nav = navigator();
    
    // Get the block from BLOCKS using block_id
    let block = BLOCKS.read().iter()
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
                    let b = block.clone();
                    move |_| {
                        if let Some(blk) = &b {
                            // Use browser prompt to get new name
                            if let Some(window) = web_sys::window() {
                                if let Ok(Some(new_name)) = window.prompt_with_message_and_default("Rename block:", &blk.name) {
                                    if !new_name.is_empty() && new_name != blk.name {
                                        tracing::info!("Rename '{}' to '{}' - Not implemented yet", blk.name, new_name);
                                        // TODO: Call API to rename block
                                    }
                                }
                            }
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
                            let id = bid.clone();
                            let name = blk.name.clone();
                            let block_backup = blk.clone();
                            
                            tracing::info!("🗑️ Optimistic delete block: {} ({})", name, id);
                            
                            // Step 1: IMMEDIATELY remove from UI (optimistic)
                            BLOCKS.write().retain(|b| b.block_id != id);
                            tracing::info!("⚡ Block removed from UI immediately");
                            
                            // Step 2: Send API DELETE in background
                            spawn(async move {
                                tracing::info!("📡 Sending DELETE /blocks/{}", id);
                                
                                match crate::api::blocks::delete_block(&id).await {
                                    Ok(_) => {
                                        tracing::info!("✅ Block delete confirmed by server for block: {}", id);
                                    }
                                    Err(e) => {
                                        tracing::error!("❌ Block delete failed for {}: {}", id, e);
                                        tracing::error!("❌ Full error details: {:?}", e);
                                        
                                        // Show user-friendly error
                                        if let Some(window) = web_sys::window() {
                                            let _ = window.alert_with_message(&format!("Failed to delete block: {}", e));
                                        }
                                        
                                        // Rollback: add block back to UI
                                        BLOCKS.write().push(block_backup.clone());
                                        tracing::info!("🔄 Block restored to UI: {}", block_backup.name);
                                    }
                                }
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
