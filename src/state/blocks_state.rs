use crate::api;
use dioxus::prelude::*;

// Global signals for blocks state
pub static BLOCKS: GlobalSignal<Vec<api::blocks_api::Block>> = Signal::global(|| Vec::new());
pub static BLOCKS_LOADING: GlobalSignal<bool> = Signal::global(|| false);
pub static BLOCKS_ERROR: GlobalSignal<Option<String>> = Signal::global(|| None);
pub static CURRENT_BLOCK_ID: GlobalSignal<Option<String>> = Signal::global(|| None);

/// Load blocks for a project from API and update global state
pub async fn load_project_blocks(project_id: &str) {
    *BLOCKS_LOADING.write() = true;
    *BLOCKS_ERROR.write() = None;

    match api::blocks_api::list_project_blocks(project_id).await {
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
pub async fn create_block(project_id: &str, name: String) -> Result<api::blocks_api::Block, String> {
    match api::blocks_api::create_block(project_id, name).await {
        Ok(block) => {
            BLOCKS.write().push(block.clone());
            Ok(block)
        }
        Err(e) => Err(e),
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
        BLOCKS
            .read()
            .iter()
            .find(|b| &b.block_id == id)
            .map(|b| b.name.clone())
    } else {
        None
    }
}

/// Delete a block with optimistic update and rollback
pub async fn delete_block(project_id: &str, block_id: &str) {
    tracing::info!("🗑️ Optimistic delete block: {}", block_id);
    
    // Step 1: Save backup for rollback
    let block_backup = BLOCKS
        .read()
        .iter()
        .find(|b| b.block_id == block_id)
        .cloned();
    
    // Step 2: Optimistic UI update - remove immediately
    BLOCKS.write().retain(|b| b.block_id != block_id);
    tracing::info!("⚡ Block removed from UI immediately");
    
    // Step 3: API call in background
    match api::blocks_api::delete_block(project_id, block_id).await {
        Ok(_) => {
            tracing::info!("✅ Block delete confirmed by server: {}", block_id);
        }
        Err(e) => {
            tracing::error!("❌ Block delete failed: {}", e);
            
            // Step 4: Rollback - restore block
            if let Some(block) = block_backup {
                BLOCKS.write().push(block.clone());
                tracing::info!("🔄 Block restored to UI: {}", block.name);
            }
            
            // Step 5: Show error to user
            if let Some(window) = web_sys::window() {
                let _ = window.alert_with_message(&format!("Failed to delete block: {}", e));
            }
        }
    }
}

/// Rename a block with optimistic update and rollback
pub async fn rename_block(project_id: &str, block_id: &str, new_name: String) {
    tracing::info!("✏️ Renaming block {} to {}", block_id, new_name);
    
    // Step 1: Save old name for rollback
    let old_name = BLOCKS
        .read()
        .iter()
        .find(|b| b.block_id == block_id)
        .map(|b| b.name.clone());
    
    // Step 2: Optimistic UI update
    BLOCKS.write().iter_mut().for_each(|b| {
        if b.block_id == block_id {
            b.name = new_name.clone();
        }
    });
    tracing::info!("⚡ Block renamed in UI immediately");
    
    // Step 3: API call in background
    match api::blocks_api::rename_block(project_id, block_id, new_name.clone()).await {
        Ok(_) => {
            tracing::info!("✅ Block rename confirmed by server");
        }
        Err(e) => {
            tracing::error!("❌ Block rename failed: {}", e);
            
            // Step 4: Rollback - restore old name
            if let Some(old) = old_name {
                BLOCKS.write().iter_mut().for_each(|b| {
                    if b.block_id == block_id {
                        b.name = old.clone();
                    }
                });
                tracing::info!("🔄 Block name restored to: {}", old);
            }
            
            // Step 5: Show error to user
            if let Some(window) = web_sys::window() {
                let _ = window.alert_with_message(&format!("Failed to rename block: {}", e));
            }
        }
    }
}
