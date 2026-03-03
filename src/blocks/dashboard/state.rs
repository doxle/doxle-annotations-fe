use super::api::{Block, BlockType, BlockLabel, api_list_blocks, api_create_block, api_create_label, api_delete_block, api_rename_block, api_get_labels};
use dioxus::prelude::*;

// Global signals for blocks state
pub static BLOCKS: GlobalSignal<Vec<Block>> = Signal::global(|| Vec::new());
pub static BLOCKS_LOADING: GlobalSignal<bool> = Signal::global(|| false);
pub static BLOCKS_ERROR: GlobalSignal<Option<String>> = Signal::global(|| None);
pub static CURRENT_BLOCK: GlobalSignal<Option<Block>> = Signal::global(|| None);

/// Track which block_id the current TASKS data belongs to
pub static TASKS_BLOCK_ID:GlobalSignal<String> = Signal::global(|| String::new());


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

/// Delete a block with per-image, per-task progress reporting
pub async fn state_delete_block(block_id: &str) {
    use crate::shell::progress::{show_progress_danger, push_log_danger, show_success, show_error};
    use crate::atoms::tasks::api::api_list_tasks;
    use crate::atoms::tasks::api::api_delete_task;
    use crate::atoms::media::api::api_delete_image;

    tracing::info!("🗑️ Delete block requested: {}", block_id);
    let start = web_time::Instant::now();

    // Save backup for rollback
    let block_backup = BLOCKS
        .read()
        .iter()
        .find(|b| b.block_id == block_id)
        .cloned();

    let block_name = block_backup.as_ref().map(|b| b.block_name.clone()).unwrap_or_default();

    // Optimistic UI removal
    let optimistic = BLOCKS.read().len() > 1;
    if optimistic {
        BLOCKS.write().retain(|b| b.block_id != block_id);
    }

    // Fetch tasks for this block
    show_progress_danger(&format!("Loading tasks for '{}'", block_name), 0, 1, 0);
    let tasks = match api_list_tasks(block_id).await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("❌ Failed to fetch tasks: {}", e);
            if optimistic {
                if let Some(block) = block_backup {
                    BLOCKS.write().push(block);
                }
            }
            show_error(&format!("Delete failed: {}", e));
            return;
        }
    };

    let task_count = tasks.len();
    // Total steps = all images across all tasks + task records + block cleanup
    let total_images: usize = tasks.iter().map(|t| t.images.len()).sum();
    let total = total_images + task_count + 1;
    let mut done: usize = 0;

    // Delete each task: images first, then task record
    for (ti, task) in tasks.iter().enumerate() {
        let img_count = task.images.len();
        let ann_count = task.annotation_count;

        // Delete each image in this task
        for (ii, image) in task.images.iter().enumerate() {
            show_progress_danger(
                &format!("Task {}/{} '{}' — img {}/{}", ti + 1, task_count, task.task_name, ii + 1, img_count),
                done, total, start.elapsed().as_secs(),
            );
            if let Err(e) = api_delete_image(block_id, &image.image_id).await {
                tracing::error!("❌ Failed to delete image {}: {}", image.image_id, e);
            }
            done += 1;
        }

        // Delete the task record
        show_progress_danger(
            &format!("Removing task {}/{} '{}'", ti + 1, task_count, task.task_name),
            done, total, start.elapsed().as_secs(),
        );
        match api_delete_task(block_id, &task.task_id).await {
            Ok(_) => {
                done += 1;
                let log_line = format!(
                    "✓ Task {}/{} '{}' ({} imgs, {} ann)",
                    ti + 1, task_count, task.task_name, img_count, ann_count
                );
                push_log_danger(
                    &log_line,
                    &format!("Deleted task {}/{}", ti + 1, task_count),
                    done, total, start.elapsed().as_secs(),
                );
            }
            Err(e) => {
                done += 1;
                tracing::error!("❌ Failed to delete task {}: {}", task.task_name, e);
            }
        }
    }

    // Delete the block record (labels, remaining block images, block itself)
    show_progress_danger(
        &format!("Cleaning up block '{}'", block_name),
        done, total, start.elapsed().as_secs(),
    );
    match api_delete_block(block_id).await {
        Ok(_) => {
            state_reset_block_context();
            if !optimistic {
                BLOCKS.write().retain(|b| b.block_id != block_id);
            }
            let elapsed = start.elapsed().as_secs();
            show_success(&format!("Block '{}' deleted in {}s", block_name, elapsed));
        }
        Err(e) => {
            tracing::error!("❌ Block delete failed: {}", e);
            if optimistic {
                if let Some(block) = block_backup {
                    BLOCKS.write().push(block);
                }
            }
            show_error(&format!("Delete failed: {}", e));
        }
    }
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





