use crate::atoms::media::Image;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Task {
	pub task_id:String,
	pub block_id:String,
	pub task_name:String,
	pub task_state:String, // "todo" | "in_progress" | "done"
	pub created_at:String,
	pub images:Vec<Image>,
	pub assignee:String,
	pub reviewer:String,
	pub locked:bool,

}
