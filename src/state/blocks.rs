use dioxus::prelude::*;
use crate::api;

// Global signals for blocks state
pub static BLOCKS: GlobalSignal<Vec<api::blocks::Block>> = Signal::global(|| Vec::new());
pub static BLOCKS_LOADING: GlobalSignal<bool> = Signal::global(|| false);
pub static BLOCKS_ERROR: GlobalSignal<Option<String>> = Signal::global(|| None);
pub static CURRENT_BLOCK_ID: GlobalSignal<Option<String>> = Signal::global(|| None);

/// Load blocks for a project from API and update global state
pub async fn load_project_blocks(project_id: &str) {
    *BLOCKS_LOADING.write() = true;
    *BLOCKS_ERROR.write() = None;
    
    match api::blocks::list_project_blocks(project_id).await {
        Ok(blocks_list) => {
            tracing::info!("✅ Blocks loaded: {} items", blocks_list.len());
            *BLOCKS.write() = blocks_list;
        }
        Err(e) => {
            tracing::error!("❌ Error loading blocks: {}", e);
            *BLOCKS_ERROR.write() = Some(e);
        }
    }
    
    *BLOCKS_LOADING.write() = false;
}

/// Create a new block and add to global state
pub async fn create_block(project_id: &str, name: String) -> Result<api::blocks::Block, String> {
    match api::blocks::create_block(project_id, name).await {
        Ok(block) => {
            BLOCKS.write().push(block.clone());
            Ok(block)
        }
        Err(e) => Err(e)
    }
}

/// Remove a block from the global state by ID and delete from API
pub async fn delete_block(block_id: &str) -> Result<(), String> {
    match api::blocks::delete_block(block_id).await {
        Ok(()) => {
            BLOCKS.write().retain(|b| b.block_id != block_id);
            Ok(())
        }
        Err(e) => Err(e)
    }
}

/// Set the current block being worked on
pub fn set_current_block(block_id: String) {
    *CURRENT_BLOCK_ID.write() = Some(block_id.clone());
    tracing::info!("📦 Current block set to: {}", block_id);
}

/// Get the current block ID
pub fn get_current_block_id() -> Option<String> {
    CURRENT_BLOCK_ID.read().clone()
}

/// Get the current block name
pub fn get_current_block_name() -> Option<String> {
    let current_id = CURRENT_BLOCK_ID.read();
    if let Some(id) = current_id.as_ref() {
        BLOCKS.read().iter()
            .find(|b| &b.block_id == id)
            .map(|b| b.name.clone())
    } else {
        None
    }
}
