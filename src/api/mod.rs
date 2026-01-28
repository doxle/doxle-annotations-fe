pub mod auth_api;
// client_api moved to users module

// Re-exports so callers can just `use crate::api::*;`
pub use auth_api::{
    AuthResult,
    authenticate,
    signup,
    get_access_token,
    store_access_token,
    store_refresh_token,
    clear_access_token,
    refresh_access_token,
    is_access_token_expired,
};

pub use crate::users::api::{
    User,
    get_current_user,
    create_user_profile,
};
