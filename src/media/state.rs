use super::api;
use super::model::Image;
use dioxus::prelude::*;

// Global signals for images state
pub static IMAGES: GlobalSignal<Vec<Image>> = Signal::global(|| Vec::new());
pub static IMAGES_LOADING: GlobalSignal<bool> = Signal::global(|| false);
pub static IMAGES_ERROR: GlobalSignal<Option<String>> = Signal::global(|| None);
pub static CURRENT_IMAGE_INDEX: GlobalSignal<usize> = Signal::global(|| 0);

/// Load images for a block from API and update global state
pub async fn load_block_images(project_id: &str, block_id: &str) {
    tracing::info!("🔍 Fetching images from API for block: {}", block_id);
    *IMAGES_LOADING.write() = true;
    *IMAGES_ERROR.write() = None;

    match api::list_block_media(project_id, block_id).await {
        Ok(images_list) => {
            tracing::info!("✅ API returned {} images", images_list.len());
            *IMAGES.write() = images_list;
            // Reset to first image when loading new images
            *CURRENT_IMAGE_INDEX.write() = 0;
        }
        Err(e) => {
            tracing::error!("❌ Error loading images: {}", e);
            *IMAGES_ERROR.write() = Some(e);
        }
    }

    *IMAGES_LOADING.write() = false;
}

/// Navigate to previous image
pub fn prev_image() {
    let current = *CURRENT_IMAGE_INDEX.read();
    if current > 0 {
        *CURRENT_IMAGE_INDEX.write() = current - 1;
        tracing::info!("📷 Navigate to image {}", current - 1);
    }
}

/// Navigate to next image
pub fn next_image() {
    let current = *CURRENT_IMAGE_INDEX.read();
    let total = IMAGES.read().len();
    if current < total.saturating_sub(1) {
        *CURRENT_IMAGE_INDEX.write() = current + 1;
        tracing::info!("📷 Navigate to image {}", current + 1);
    }
}

/// Get the current image
pub fn get_current_image() -> Option<Image> {
    let index = *CURRENT_IMAGE_INDEX.read();
    IMAGES.read().get(index).cloned()
}
