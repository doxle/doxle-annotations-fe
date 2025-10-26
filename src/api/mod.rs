pub mod auth;
pub mod client;
pub mod projects;
pub mod blocks;
pub mod images;
pub mod annotations;
pub mod websocket;
pub mod classes;
pub mod uploads;

// Re-exports for convenience
pub use auth::{authenticate, signup, store_token, get_token, clear_token};
pub use client::{User, get_user};
pub use projects::{Project, list_projects};
pub use websocket::{init_websocket, is_websocket_connected, close_websocket};

/// Get the API URL from environment or default to localhost
pub fn get_api_url() -> String {
    // TODO: Make this configurable via environment
    "http://localhost:9000".to_string()
}
