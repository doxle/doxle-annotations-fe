use super::add_block::AddBlockModal;
use super::block_dropdown::BlockDropdown;
use crate::shared::{AppSidebar, AppSidebarPage};
use crate::state::{get_project_by_id, BLOCKS, BLOCKS_LOADING, BLOCKS_ERROR, load_project_blocks, create_block, delete_block, set_current_block};
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

    let mut block_dropdown_menu = use_signal(|| None::<(usize, f64, f64)>); // (block_index, x, y)
    let mut show_add_modal = use_signal(|| false);
    
    // Load blocks on mount - runs only once
    use_hook(|| {
        let pid = project_id.clone();
        spawn(async move {
            // Load blocks if not already loaded
            if BLOCKS.read().is_empty() && !*BLOCKS_LOADING.read() {
                load_project_blocks(&pid).await;
            }
        });
    });

    let handle_add_block = move |_| {
        *show_add_modal.write() = true;
    };

    let pid_for_create = project_id.clone();
    let handle_block_added = move |block_name: String| {
        let pid = pid_for_create.clone();
        spawn(async move {
            if let Err(e) = create_block(&pid, block_name).await {
                tracing::error!("Failed to create block: {}", e);
            }
        });
    };

    rsx! {
        div {
            class: "blocks-page-container",
            onclick: move |_| {
                block_dropdown_menu.set(None);
            },

            // Add Block Modal
            AddBlockModal {
                show: show_add_modal,
                project_id: project_id.clone(),
                on_add: handle_block_added
            }

            // Shared sidebar
            AppSidebar {
                current_page: AppSidebarPage::Blocks {
                    project_id: project_id.clone(),
                    project_name: project_name.clone()
                }
            }

            // Main content area
            div {
                class: "blocks-content",

                // Header with title and add button
                div {
                    class: "block-header",

                h1 {
                    class: "block-title",
                    "{project_name}"
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

                // Table header
                div {
                    class: "block-table-header",

                    div { "Name" }
                    div { "Progress" }
                    div { "Assigned" }
                    div { "Last Modified" }
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
                    div {
                        key: "{index}",
                        class: "block-row",
                        oncontextmenu: move |e| {
                            e.prevent_default();
                            let client_x = e.client_coordinates().x;
                            let client_y = e.client_coordinates().y;
                            block_dropdown_menu.set(Some((index, client_x, client_y)));
                        },
                        onclick: move |e| {
                            e.stop_propagation();
                            block_dropdown_menu.set(None);
                            // Navigate to canvas page with actual block ID
                            let block_id = BLOCKS.read().get(index).map(|b| b.block_id.clone()).unwrap_or_default();
                            set_current_block(block_id.clone());
                            nav.push(Route::CanvasPage { task_id: block_id });
                        },

                        // Name column with image
                        div {
                            class: "block-name-col",

                            // Colored rectangle placeholder for image
                            div {
                                class: "block-image-placeholder"
                            }

                            span {
                                class: "block-name-text",
                                "{block.name}"
                            }
                        }

                        // Progress column (placeholder for now)
                        div {
                            class: "block-progress",
                            "[0/0]"
                        }

                        // Assigned column
                        div {
                            class: "block-assigned",
                            if let Some(assigned) = &block.assigned_to {
                                "{assigned}"
                            } else {
                                "Unassigned"
                            }
                        }

                        // Last Modified column
                        div {
                            class: "block-modified",
                            "{block.created_at}"
                        }
                    }
                }
                }
            }

            // Block dropdown menu
            if let Some((block_idx, x, y)) = *block_dropdown_menu.read() {
                BlockDropdown {
                    x: x,
                    y: y,
                    block_index: block_idx,
                    on_close: move |_| block_dropdown_menu.set(None),
                    on_open: move |idx: usize| {
                        let blocks = BLOCKS.read();
                        if let Some(block) = blocks.get(idx) {
                            let block_id = block.block_id.clone();
                            set_current_block(block_id.clone());
                            nav.push(Route::CanvasPage { task_id: block_id });
                        }
                    },
                    on_rename: move |idx| {
                        // TODO: Implement rename functionality
                        println!("Rename block {} clicked", idx);
                    },
                    on_delete: move |idx: usize| {
                        let blocks = BLOCKS.read();
                        if let Some(block) = blocks.get(idx) {
                            let block_id = block.block_id.clone();
                            spawn(async move {
                                if let Err(e) = delete_block(&block_id).await {
                                    tracing::error!("Failed to delete block: {}", e);
                                }
                            });
                        }
                    }
                }
            }
            } // Close blocks-content div
        }
    }
}
