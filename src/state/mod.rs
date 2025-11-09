pub mod projects_state;
pub mod blocks_state;
pub mod images_state;
pub mod user_state;

// Re-export commonly used items
pub use projects_state::{PROJECTS, PROJECTS_LOADING, PROJECTS_ERROR, load_projects, get_project_by_id, remove_project, add_project, delete_project};
pub use blocks_state::{BLOCKS, BLOCKS_LOADING, BLOCKS_ERROR, CURRENT_BLOCK_ID, load_project_blocks, create_block, delete_block, rename_block, set_current_block, get_current_block_id, get_current_block_name};
pub use images_state::{IMAGES, IMAGES_LOADING, IMAGES_ERROR, CURRENT_IMAGE_INDEX, load_block_images, prev_image, next_image, get_current_image};
pub use user_state::{USER, USER_LOADING, load_user, clear_user, create_user_profile};
