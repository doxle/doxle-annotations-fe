use dioxus::prelude::*;
use crate::users::api::{User, get_current_user};


// Global signal for user state
pub static USER:GlobalSignal<Option<User>> = Signal::global(||None);
pub static USER_LOADING:GlobalSignal<bool> = Signal::global(||false);

/// Load user from API and update global state
pub async fn load_user(){
	*USER_LOADING.write() = true;

	match get_current_user().await {
		Ok(user)=>{
			tracing::info!("✅ User loaded: {}", user.user_name);
			*USER.write() = Some(user);
		}
		Err(e) => {
            tracing::error!("❌ Error loading user: {:?}", e);
            *USER.write() = None;
        }
	}
	*USER_LOADING.write() = false;
}
