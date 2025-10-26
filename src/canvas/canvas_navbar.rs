use super::annotations::shapes::{BBox, Polygon};
use super::annotations::Tool;
use super::navbar::{LeftSection, CenterSection, RightSection};
use dioxus::prelude::*;

#[component]
pub fn CanvasNavbar(
    selected_tool: Signal<Tool>,
    avatar_dropdown_open: Signal<bool>,
    show_grid_lines: Signal<bool>,
    sidebar_open: Signal<bool>,
    polygon: Signal<Polygon>,
    bbox: Signal<BBox>,
) -> Element {

    rsx! {
        // Navbar
        div {
            class: "canvas-navbar",
            onclick: move |e| {
                tracing::info!("🎯 NAVBAR CLICKED!");
                e.stop_propagation();
                if avatar_dropdown_open() {
                    avatar_dropdown_open.set(false);
                }
            },

            // Left section
            LeftSection { }

            // Center section
            CenterSection {
                selected_tool: selected_tool,
                polygon: polygon,
                bbox: bbox
            }

            // Right section
            RightSection {
                avatar_dropdown_open: avatar_dropdown_open,
                show_grid_lines: show_grid_lines,
                sidebar_open: sidebar_open
            }
        }

    }
}
