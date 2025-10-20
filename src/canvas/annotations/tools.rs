/// Canvas Tools - Always exactly ONE tool is selected at any time
///
/// Logic:
/// - Default tool on page load: Select (Arrow)
/// - Clicking a tool button always sets that tool (no toggle behavior)
/// - Only one tool can be active at any time (mutually exclusive)
/// - When switching tools, previous tool's annotation is cleared/saved to DB

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tool {
    Select,      // Arrow cursor - select and move annotations
    Pan,         // Hand cursor - pan the canvas (also works with middle mouse)
    Polygon,     // Draw polygon annotations
    BoundingBox, // Draw bounding box annotations
    Comment,     // Add comment annotations
}

impl Default for Tool {
    fn default() -> Self {
        Tool::Select
    }
}

impl Tool {
    /// Returns true if this tool allows drawing annotations
    pub fn is_drawing_tool(&self) -> bool {
        matches!(self, Tool::Polygon | Tool::BoundingBox | Tool::Comment)
    }

    /// Returns true if this tool is an annotation tool (not Select or Pan)
    pub fn is_annotation_tool(&self) -> bool {
        matches!(self, Tool::Polygon | Tool::BoundingBox | Tool::Comment)
    }

    /// Returns the cursor class for this tool
    pub fn cursor_class(&self) -> &'static str {
        match self {
            Tool::Select => "cursor-default",
            Tool::Pan => "cursor-grab",
            Tool::Polygon => "cursor-crosshair",
            Tool::BoundingBox => "cursor-crosshair",
            Tool::Comment => "cursor-text",
        }
    }
}
