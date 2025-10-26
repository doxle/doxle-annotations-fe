pub mod projects;
pub mod blocks;
pub mod images;
pub mod user;

// Re-export commonly used items
pub use projects::{PROJECTS, PROJECTS_LOADING, PROJECTS_ERROR, load_projects, get_project_by_id, remove_project, add_project};
pub use blocks::{BLOCKS, BLOCKS_LOADING, BLOCKS_ERROR, CURRENT_BLOCK_ID, load_project_blocks, create_block, delete_block, set_current_block, get_current_block_id, get_current_block_name};
pub use images::{IMAGES, IMAGES_LOADING, IMAGES_ERROR, CURRENT_IMAGE_INDEX, load_block_images, prev_image, next_image, get_current_image};
pub use user::{USER, USER_LOADING, load_user, clear_user, create_user_profile};
