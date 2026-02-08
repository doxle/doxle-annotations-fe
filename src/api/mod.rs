pub mod auth_api;
// client_api moved to users module

// Re-exports so callers can just `use crate::api::*;`
pub use auth_api::{
    SessionResponse,
    InviteResponse,
    authenticate,
    signup,
    send_contact,
    logout,
    create_invite,
};

pub use crate::users::api::{
    User,
    get_current_user,
    create_user_profile,
};
