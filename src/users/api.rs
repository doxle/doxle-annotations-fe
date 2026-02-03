use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    #[serde(default)]
    pub user_name: String,
    pub user_email: String,
    pub user_company: Option<String>,
    pub user_role: String,
    pub user_created_at: String,
    pub user_last_login: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateUserRequest {
    pub user_name: String,
    pub user_email: String,
    pub user_company: Option<String>,
    pub user_role: String,
}

// GET /users/me
pub async fn get_current_user() -> Result<User, String> {
    tracing::info!("👤 Getting user profile");
    crate::shell::client::get::<User>("/users/me").await
}

// POST /users
pub async fn create_user_profile(
    user_name: String,
    user_email: String,
    user_company: Option<String>,
    user_role: String,
) -> Result<User, String> {
    tracing::info!("➕ Creating new user profile: {}", user_email);

    let request_body = CreateUserRequest {
        user_name,
        user_email,
        user_company,
        user_role,
    };

    crate::shell::client::post::<CreateUserRequest, User>("/users", &request_body).await
}
