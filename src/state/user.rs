use crate::api;
use dioxus::prelude::*;

// Global signal for user state
pub static USER: GlobalSignal<Option<api::User>> = Signal::global(|| None);
pub static USER_LOADING: GlobalSignal<bool> = Signal::global(|| false);

/// Load user from API and update global state
pub async fn load_user() {
    if let Some(_token) = api::get_token() {
        *USER_LOADING.write() = true;
        
        match api::client::get_current_user().await {
            Ok(user) => {
                tracing::info!("✅ User loaded: {}", user.email);
                *USER.write() = Some(user);
            }
            Err(e) => {
                tracing::error!("❌ Error loading user: {:?}", e);
                *USER.write() = None;
            }
        }
        
        *USER_LOADING.write() = false;
    } else {
        *USER.write() = None;
    }
}

/// Create user profile
pub async fn create_user_profile(name: String, email: String, company: Option<String>, role: String) -> Result<(), String> {
    match api::client::create_user_profile(name, email, company, role).await {
        Ok(user) => {
            tracing::info!("✅ User created: {}", user.email);
            *USER.write() = Some(user);
            Ok(())
        }
        Err(e) => {
            tracing::error!("❌ Error creating user: {:?}", e);
            Err(e)
        }
    }
}

/// Clear user state (for logout)
pub fn clear_user() {
    *USER.write() = None;
}
