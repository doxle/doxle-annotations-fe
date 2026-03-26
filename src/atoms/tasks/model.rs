use crate::atoms::media::Image;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
	Todo,
	Working,
	Submitted,
	Rejected,
	Approved,
}

impl TaskState {
	pub fn label(&self) -> &'static str {
		match self {
			Self::Todo => "Todo",
			Self::Working => "Working",
			Self::Submitted => "Submitted",
			Self::Rejected => "Rejected",
			Self::Approved => "Approved",
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Task {
	pub task_id:String,
	pub block_id:String,
	pub task_name:String,
	pub task_state:TaskState,
	pub created_at:String,
	pub images:Vec<Image>,
	pub annotation_count:u32,
	pub assignee:String,
	pub reviewer:String,
	pub locked:bool,
	#[serde(default)]
	pub image_count:u32,
	#[serde(default)]
	pub labels_count:std::collections::HashMap<String, u32>,
	#[serde(default)]
	pub bbox_count:std::collections::HashMap<String, u32>,
	#[serde(default)]
	pub polygon_count:std::collections::HashMap<String, u32>,
}
