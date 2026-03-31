use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Annotator,
    Builder,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::Annotator
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub user_id: String,
    pub user_name: String,
    pub user_email: String,
    pub user_company: Option<String>,
    pub user_role: UserRole,
    pub user_created_at: String,
    pub user_last_login: Option<String>,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.user_role == UserRole::Admin
    }

    pub fn is_annotator(&self) -> bool {
        self.user_role == UserRole::Annotator
    }
}

#[derive(Debug, Serialize)]
pub struct CreateUserRequest {
    pub user_name: String,
    pub user_email: String,
    pub user_company: Option<String>,
    pub user_role: UserRole,
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

// POST /users
pub async fn create_user_profile(
    user_name: String,
    user_email: String,
    user_company: Option<String>,
    user_role: UserRole,
) -> Result<User, String> {
    tracing::info!("➕ Creating new user profile: {}", user_email);

    let request_body = CreateUserRequest {
        user_name,
        user_email,
        user_company,
        user_role,
    };

    crate::core::client::post::<CreateUserRequest, User>("/users", &request_body).await
}
