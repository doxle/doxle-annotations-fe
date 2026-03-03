use serde::{Deserialize, Serialize};
use crate::atoms::svg_canvas::Geometry;

// ============================================
// Label (color comes from here)
// ============================================

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct Label {
    pub label_id: String,
    pub name: String,
    pub color: String,
}

// ============================================
// Annotation
// ============================================

#[derive(Clone, PartialEq, Debug)]
pub struct Annotation {
    pub id: String,
    pub label_id: String,
    pub geometry: Geometry,
}

// ============================================
// Comment (pinned to a canvas location)
// ============================================

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub user_id: String,
    pub user_name: String,
    pub text: String,
    pub created_at: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CommentThread {
    pub id: String,
    pub world_x: f64,
    pub world_y: f64,
    pub resolved: bool,
    pub comments: Vec<Comment>,
    /// Whether this thread has been persisted to the server
    #[serde(default = "default_persisted")]
    pub persisted: bool,
}

fn default_persisted() -> bool { true }

// ============================================
// API Response Types (match BE models)
// ============================================

#[derive(Debug, Clone, Deserialize)]
pub struct ApiComment {
    pub comment_id: String,
    pub thread_id: String,
    pub user_id: String,
    pub user_name: String,
    pub text: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiCommentThread {
    pub thread_id: String,
    pub parent_id: String,
    pub metadata: Option<String>,
    pub resolved: bool,
    pub created_by: String,
    pub created_at: String,
    #[serde(default)]
    pub comments: Vec<ApiComment>,
}

/// Thread metadata for canvas annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMetadata {
    pub world_x: f64,
    pub world_y: f64,
}
