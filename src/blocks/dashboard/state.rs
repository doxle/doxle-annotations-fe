use super::api::{Block, BlockType, BlockLabel, api_list_blocks, api_create_block, api_create_label, api_delete_block, api_rename_block, api_get_labels};
use dioxus::prelude::*;

// Global signals for blocks state
pub static BLOCKS: GlobalSignal<Vec<Block>> = Signal::global(|| Vec::new());
pub static BLOCKS_LOADING: GlobalSignal<bool> = Signal::global(|| false);
pub static BLOCKS_ERROR: GlobalSignal<Option<String>> = Signal::global(|| None);
pub static CURRENT_BLOCK: GlobalSignal<Option<Block>> = Signal::global(|| None);

pub static LABELS: GlobalSignal<Vec<BlockLabel>> = Signal::global(|| Vec::new());
pub static LABELS_LOADING:GlobalSignal<bool> = Signal::global(||false);


/// Reset all block-specific state (call when leaving block or deleting)
pub fn state_reset_block_context() {
    *CURRENT_BLOCK.write() = None;
    *LABELS.write() = Vec::new();
    *LABELS_LOADING.write() = false;
    tracing::info!("🧹 Block context reset");
}


/// Load all blocks from API and update global state
pub async fn state_load_blocks() {
    *BLOCKS_LOADING.write() = true;
    *BLOCKS_ERROR.write() = None;

    match api_list_blocks().await {
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


/// Load blocks silently (no loading spinner) - for background refresh
pub async fn state_load_blocks_silent() {
    match api_list_blocks().await {
        Ok(blocks_list) => {
            tracing::info!("✅ Blocks refreshed silently: {} items", blocks_list.len());
            *BLOCKS.write() = blocks_list;
        }
        Err(e) => {
            tracing::error!("❌ Error refreshing blocks silently: {}", e);
        }
    }
}

/// Create a new block (BE auto-creates default labels)
pub async fn state_create_block(name: String, block_type: BlockType, company: Option<String>) -> Result<Block, String> {
    let block = match api_create_block(name, block_type, company).await {
        Ok(block) => block,
        Err(e) => return Err(e)
    };
    
    tracing::info!("📦 Block created with {} labels", block.labels.len());
    BLOCKS.write().push(block.clone());
    Ok(block)
}


/// Set the current block being worked on
pub fn state_set_current_block(block_id: &str) {

    if let Some(block) = BLOCKS.read().iter().find(|b| b.block_id == block_id).cloned(){
        *CURRENT_BLOCK.write() = Some(block);
        tracing::info!("📦 Current block set to: ({:?})", CURRENT_BLOCK.read().as_ref().map(|b| b.block_name.clone()));
        return;
    }
}

pub fn state_get_current_block()->Option<Block>{
    CURRENT_BLOCK.read().clone()
}

/// Delete a block with optimistic update and rollback
pub async fn state_delete_block(block_id: &str) {
    tracing::info!("🗑️ Delete block requested: {}", block_id);

    // Always set loading during delete so dashboard doesn't auto-redirect while we work
    *BLOCKS_LOADING.write() = true;

    // Step 1: Save backup for rollback (if we do optimistic removal)
    let block_backup = BLOCKS
        .read()
        .iter()
        .find(|b| b.block_id == block_id)
        .cloned();

    // Step 2: Decide optimistic vs conservative
    let total = BLOCKS.read().len();
    let optimistic = total > 1; // If it's the last block, DON'T remove yet

    if optimistic {
        // Optimistic UI update - remove immediately
        BLOCKS.write().retain(|b| b.block_id != block_id);
        tracing::info!("⚡ Block removed from UI immediately (optimistic)");
    } else {
        tracing::info!("⌛ Last block - delaying UI removal until server confirms");
    }

    // Step 3: API call
    match api_delete_block(block_id).await {
        Ok(_) => {
            tracing::info!("✅ Block delete confirmed by server: {}", block_id);
            state_reset_block_context();
            
            if !optimistic {
                // Now remove it for real (this triggers redirect effect after we drop loading)
                BLOCKS.write().retain(|b| b.block_id != block_id);
            }
        }
        Err(e) => {
            tracing::error!("❌ Block delete failed: {}", e);
            // Rollback optimistic removal if needed
            if optimistic {
                if let Some(block) = block_backup {
                    BLOCKS.write().push(block.clone());
                    tracing::info!("🔄 Block restored to UI: {}", block.block_name);
                }
            }
            if let Some(window) = web_sys::window() {
                let _ = window.alert_with_message(&format!("Failed to delete block: {}", e));
            }
        }
    }

    // Step 4: Done
    *BLOCKS_LOADING.write() = false;
}

/// Rename a block with optimistic update and rollback
pub async fn state_rename_block(block_id: &str, new_name: String) {
    tracing::info!("✏️ Renaming block {} to {}", block_id, new_name);
    
    // Step 1: Save old name for rollback
    let old_name = BLOCKS
        .read()
        .iter()
        .find(|b| b.block_id == block_id)
        .map(|b| b.block_name.clone());
    
    // Step 2: Optimistic UI update
    BLOCKS.write().iter_mut().for_each(|b| {
        if b.block_id == block_id {
            b.block_name = new_name.clone();
        }
    });
    tracing::info!("⚡ Block renamed in UI immediately");
    
    // Step 3: API call in background
    match api_rename_block(block_id, new_name.clone()).await {
        Ok(_) => {
            tracing::info!("✅ Block rename confirmed by server");
        }
        Err(e) => {
            tracing::error!("❌ Block rename failed: {}", e);
            
            // Step 4: Rollback - restore old name
            if let Some(old) = old_name {
                BLOCKS.write().iter_mut().for_each(|b| {
                    if b.block_id == block_id {
                        b.block_name = old.clone();
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


/// Load labels for a block
pub async fn state_load_labels(block_id:&str){
    *LABELS_LOADING.write() = true;
    let order = ["fp-outside", "fp-inside", "ewalls", "windows", "iwalls", "doors"];

    match api_get_labels(block_id).await {
        Ok(labels_list)=> {
            tracing::info!("✅ Labels loaded: {} items", &labels_list.len());
            for label in &labels_list {
                info!("{}", "-".repeat(45));
                info!("Label:= {:?}", label);
                info!("{}", "-".repeat(45));
            }

            let mut sorted:Vec<BlockLabel> = Vec::new();

            // First sort them in order
            for name in order {
                if let Some(label) = labels_list.iter().find(|l| l.label_name == name) {
                    sorted.push(label.clone());
                }
            }

             // Then: add any remaining labels not in the order list
            for label in labels_list {
                if !order.contains(&label.label_name.as_str()) {
                    sorted.push(label);
                }
            }

            
            *LABELS.write() = sorted;
        }
        Err(e) =>{
            tracing::error!("❌ Error loading labels: {}", e);
        }
    }
    *LABELS_LOADING.write() = false;
}

/// Refresh a single block's labels from BE
pub async fn state_refresh_block_labels(block_id:&str){
    tracing::info!("🔄 Fetching labels for block {}", block_id);
    match api_get_labels(block_id).await {
        Ok(labels) => {
            tracing::info!("📥 Got {} labels from BE:", labels.len());
            for l in &labels {
                tracing::info!("   - {} ({}): count={}", l.label_name, l.label_id, l.label_count);
            }
            let total:u32 = labels.iter().map(|l| l.label_count).sum();
            BLOCKS.write().iter_mut().for_each(|b| {
                if b.block_id == block_id {
                    tracing::info!("✏️ Updating block {} in BLOCKS signal", block_id);
                    b.labels = labels.clone();
                    b.annotation_count = total;
                }
            });
            tracing::info!("🔄 Refreshed labels for block {}", block_id);
        }
        Err(e) => tracing::error!("❌ Failed refreshing block labels: {}", e),
    
    }

}





