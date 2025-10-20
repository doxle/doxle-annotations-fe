use dioxus::prelude::*;
use crate::Route;

#[component]
pub fn BlocksPage(project_id: String) -> Element {
    let nav = navigator();
    
    // Mock blocks data - in real app, fetch based on project_id
    let mut blocks = use_signal(|| vec![
        ("Annotate Street Signs", 30, 50, "Kim Beale", "2 mins ago"),
        ("Label Vehicles", 45, 100, "John Smith", "15 mins ago"),
        ("Classify Objects", 20, 60, "Sarah Jones", "1 hour ago"),
    ]);
    let mut context_menu = use_signal(|| None::<(usize, f64, f64)>); // (block_index, x, y)
    
    let handle_add_block = move |_| {
        println!("Add block clicked");
    };

    rsx! {
        div {
            class: "block-container",
            onclick: move |_| {
                context_menu.set(None);
            },
            
            // Header with title and add button
            div {
                class: "block-header",
                
                h1 {
                    class: "block-title",
                    "Project {project_id} - Blocks"
                }
                
                button {
                    class: "btn-add-block",
                    onclick: handle_add_block,
                    "+ Add Block"
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
                
                // Table rows
                for (index, (block_name, annotated, total, assigned, last_modified)) in blocks.read().iter().enumerate() {
                    div {
                        key: "{index}",
                        class: "block-row",
                        oncontextmenu: move |e| {
                            e.prevent_default();
                            let client_x = e.client_coordinates().x;
                            let client_y = e.client_coordinates().y;
                            context_menu.set(Some((index, client_x, client_y)));
                        },
                        onclick: move |e| {
                            e.stop_propagation();
                            context_menu.set(None);
                            // Navigate to canvas page with block ID
                            nav.push(Route::CanvasPage { task_id: format!("block-{}", index) });
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
                                "{block_name}"
                            }
                        }
                        
                        // Progress column
                        div {
                            class: "block-progress",
                            "[{annotated}/{total}]"
                        }
                        
                        // Assigned column
                        div {
                            class: "block-assigned",
                            "{assigned}"
                        }
                        
                        // Last Modified column
                        div {
                            class: "block-modified",
                            "{last_modified}"
                        }
                    }
                }
            }
            
            // Context menu
            if let Some((block_idx, x, y)) = *context_menu.read() {
                div {
                    class: "context-menu",
                    style: "left: {x}px; top: {y}px;",
                    onclick: move |e| {
                        e.stop_propagation();
                    },
                    
                    div {
                        class: "context-menu-item delete",
                        onclick: move |_| {
                            println!("Delete block {} clicked", block_idx);
                            blocks.write().remove(block_idx);
                            context_menu.set(None);
                        },
                        "Delete"
                    }
                }
            }
        }
    }
}
