pub mod annotations_api;
pub mod auth_api;
pub mod blocks_api;
pub mod classes_api;
pub mod client_api;
pub mod cloudfront_api;
pub mod images_api;
pub mod projects_api;
pub mod uploads_api;
pub mod websocket_api;

// Re-exports for convenience
pub use auth_api::{
    authenticate, clear_token, get_token, refresh_access_token, signup, store_refresh_token,
    store_token,
};
pub use client_api::{get_user, User};
pub use projects_api::{list_projects, Project};
pub use websocket_api::{close_websocket, init_websocket, is_websocket_connected};

/// Get the API URL from environment or default to localhost
pub fn get_api_url() -> String {
    // TODO: Make this configurable via environment
    "https://api.doxle.ai".to_string()
}
