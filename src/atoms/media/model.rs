use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Image {
	pub image_id:String,
	pub block_id:String,
	pub task_id:Option<String>,
	pub image_name:String,
	pub url: String,
	pub locked:bool,
	pub order:Option<i32>,
	pub annotation_count:u32,
	pub labels_count:HashMap<String, u32>,
	pub bbox_count:HashMap<String, u32>,
	pub polygon_count:HashMap<String, u32>,
	pub uploaded_at:String
}
