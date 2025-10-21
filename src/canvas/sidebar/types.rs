use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq)]
pub enum SidebarTab {
    Classes,
    Comments,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClassItem {
    pub id: String,
    pub name: String,
    pub color: String, // hex "#RRGGBB"
    pub count: u32,
}
