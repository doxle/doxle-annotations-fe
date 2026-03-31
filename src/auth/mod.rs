pub mod api;
// client_api moved to users module

// Re-exports so callers can just `use crate::auth::*;`
pub use api::{
    SessionResponse,
    SignupResponse,
    InviteResponse,
    authenticate,
    signup,
    send_contact,
    logout,
    create_invite,
    list_invites,
    delete_invite,
};

pub use crate::users::api::{
    User,
    get_current_user,
    create_user_profile,
};
