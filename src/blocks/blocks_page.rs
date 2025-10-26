use super::add_block::AddBlockModal;
use super::block_dropdown::BlockDropdown;
use super::block_row::BlockRow;
use crate::shared::{AppSidebar, AppNavbar, NavbarContext};
use crate::state::{
    create_block, delete_block, get_project_by_id, load_project_blocks, load_projects,
    BLOCKS, BLOCKS_ERROR, BLOCKS_LOADING, PROJECTS,
};
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn BlocksPage(project_id: String) -> Element {
    let nav = navigator();

    // Get the actual project name
    let project = get_project_by_id(&project_id);
    let project_name = project
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| project_id.clone());

    let mut block_dropdown_menu = use_signal(|| None::<(String, f64, f64)>); // (block_id, x, y)
    let mut show_add_modal = use_signal(|| false);
    let mut sidebar_open = use_signal(|| true); // Sidebar open by default

    // Load projects and blocks on mount - runs only once
    use_hook(|| {
        let pid = project_id.clone();
        spawn(async move {
            // Load projects first if not already loaded
            if PROJECTS.read().is_empty() {
                load_projects().await;
            }
            // Load blocks if not already loaded
            if BLOCKS.read().is_empty() && !*BLOCKS_LOADING.read() {
                load_project_blocks(&pid).await;
            }
        });
    });

    let handle_add_block = move |_| {
        *show_add_modal.write() = true;
    };

    rsx! {
        div {
            class: if sidebar_open() { "blocks-page-container sidebar-open" } else { "blocks-page-container" },
            onclick: move |_| {
                block_dropdown_menu.set(None);
            },

            // Add Block Modal
            AddBlockModal {
                show: show_add_modal,
                project_id: project_id.clone()
            }

            // App Navbar
            AppNavbar {
                context: NavbarContext::Blocks { project_name: Box::leak(project_name.clone().into_boxed_str()) },
                sidebar_open: sidebar_open
            }

            // Shared sidebar
            AppSidebar { 
                open: sidebar_open
            }

            // Main content area
            div {
                class: "blocks-content",

                // Header with add button
                div {
                    class: "block-header",

                    h1 {
                        class: "block-title",
                        "Blocks"
                    }

                    button {
                        class: "btn-add-block",
                        onclick: handle_add_block,
                        svg {
                            width: "16",
                            height: "16",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            line { x1: "12", y1: "5", x2: "12", y2: "19" }
                            line { x1: "5", y1: "12", x2: "19", y2: "12" }
                        }
                        span { "Create Block" }
                    }
                }

                // Blocks table
                div {
                    class: "block-table",

                    // Table header - only show when there are blocks
                    if !BLOCKS.read().is_empty() {
                        div {
                            class: "block-table-header",

                            div { "Name" }
                            div { "Progress" }
                            div { "Assigned" }
                            div { "Last Modified" }
                        }
                    }

                    // Table rows or loading/error states
                    if *BLOCKS_LOADING.read() {
                        div {
                            class: "block-table-loading",
                            "Loading blocks..."
                        }
                    } else if let Some(error) = BLOCKS_ERROR.read().as_ref() {
                        div {
                            class: "block-table-error",
                            "Error loading blocks: {error}"
                        }
                    } else if BLOCKS.read().is_empty() {
                        div {
                            class: "block-table-empty",
                            "No blocks available. Create your first block!"
                        }
                    } else {
                        for (index, block) in BLOCKS.read().iter().enumerate() {
                            BlockRow {
                                index: index,
                                block: block.clone(),
                                on_context_menu: move |data: (String, f64, f64)| {
                                    block_dropdown_menu.set(Some(data));
                                }
                            }
                        }
                    }
                }

                // Block dropdown menu
                if let Some((block_id, x, y)) = block_dropdown_menu.read().clone() {
                    BlockDropdown {
                        x: x,
                        y: y,
                        block_id: block_id,
                        on_close: block_dropdown_menu
                    }
                }
            } // Close blocks-content div
        }
    }
}
