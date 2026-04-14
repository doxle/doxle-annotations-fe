use serde::{Deserialize, Serialize};
use std::collections::HashMap;
fn default_media_type() -> String {
	"image".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MarkupRect {
	pub x: f64,
	pub y: f64,
	pub width: f64,
	pub height: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Attachment {
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
	#[serde(default)]
	pub image_state:String,
	pub uploaded_at:String,
	#[serde(default = "default_media_type")]
	pub media_type: String,
	#[serde(default)]
	pub markup_rects: Vec<MarkupRect>,
	#[serde(default)]
	pub width: Option<u32>,
	#[serde(default)]
	pub height: Option<u32>,
	#[serde(default)]
	pub file_size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FileAttachment {
	pub attachment_id: String,
	pub block_id: String,
	pub attachment_name: String,
	pub url: String,
	pub locked: bool,
	pub order: Option<i32>,
	pub annotation_count: u32,
	pub labels_count: HashMap<String, u32>,
	pub bbox_count: HashMap<String, u32>,
	pub polygon_count: HashMap<String, u32>,
	pub uploaded_at: String,
	#[serde(default = "default_media_type")]
	pub media_type: String,
	#[serde(default)]
	pub markup_rects: Vec<MarkupRect>,
	#[serde(default)]
	pub width: Option<u32>,
	#[serde(default)]
	pub height: Option<u32>,
	#[serde(default)]
	pub attachment_size: u64,
}

impl From<Attachment> for FileAttachment {
	fn from(value: Attachment) -> Self {
		Self {
			attachment_id: value.image_id,
			block_id: value.block_id,
			attachment_name: value.image_name,
			url: value.url,
			locked: value.locked,
			order: value.order,
			annotation_count: value.annotation_count,
			labels_count: value.labels_count,
			bbox_count: value.bbox_count,
			polygon_count: value.polygon_count,
			uploaded_at: value.uploaded_at,
			media_type: value.media_type,
			markup_rects: value.markup_rects,
			width: value.width,
			height: value.height,
			attachment_size: value.file_size,
		}
	}
}
