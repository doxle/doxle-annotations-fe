use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub user_id: String,
    pub user_name: String,
    pub user_email: String,
    pub user_company: Option<String>,
    pub user_created_at: String,
    pub user_last_login: Option<String>,
    #[serde(default)]
    pub is_admin: bool,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.is_admin
    }
}

// GET /users
pub async fn list_users() -> Result<Vec<User>, String> {
    crate::core::client::get::<Vec<User>>("/users").await
}

// GET /users/me
pub async fn get_current_user() -> Result<User, String> {
    tracing::info!("👤 Getting user profile");
    crate::core::client::get::<User>("/users/me").await
}
