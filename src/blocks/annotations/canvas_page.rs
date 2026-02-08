/**

┌────────────────────────────────────────────────────────────────────────┐
│  canvas_page.rs - THE SINGLE SOURCE OF TRUTH                          │
│                                                                        │
│  Owns ALL state:                                                       │
│  • selected_tool: Signal<Tool>     (Pan, Polygon, BBox, etc.)         │
│  • active_drawing: Signal<Vec<Point>>                                  │
│  • pan/zoom transform                                                  │
│  • grid_visible, sidebar_open                                          │
│                                                                        │
│  Calls: setup_keyboard_shortcuts(all signals)                          │
│  Passes: all signals down to SvgCanvas                                 │
└────────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────────────┐
│  keyboard_shortcuts.rs - THE ONLY PLACE FOR KEYBOARD HANDLING         │
│                                                                        │
│  Receives signals, modifies them on key press                          │
│  • p → selected_tool.set(Tool::Polygon)                               │
│  • Escape → selected_tool.set(Tool::Pan), clear drawing                │
│  • g → toggle grid                                                     │
└────────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────────┐
│  svg_canvas.rs - DUMB RENDERER                                         │
│                                                                        │
│  Receives props, renders based on them                                 │
│  NO internal state, NO keyboard handlers                               │
└────────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────────┐
│  event_layer.rs - MOUSE ONLY                                           │
│                                                                        │
│  Receives props (selected_tool, active_drawing, etc.)                  │
│  Routes mouse events based on selected_tool prop                       │
│  NO keyboard handlers, NO internal tool state                          │
└────────────────────────────────────────────────────────────────────────┘



When SvgCanvas modifies signals that canvas_page reads:
•  cursor_world_pos() - every mouse move → canvas_page re-renders
•  active_drawing() - every point added → re-renders
•  zoom() - every zoom change → re-renders
•  image_size() - when image loads → re-renders
**/


use std::collections::HashSet;
use dioxus::prelude::*;
use dioxus::logger::tracing::info;
use crate::atoms::svg_canvas::{SvgCanvas, SvgCanvasV2};
use crate::atoms::svg_canvas::state::Tool;
use crate::atoms::tasks::state::{TASKS, TASKS_LOADING, state_load_tasks};
use crate::atoms::tasks::model::Task;
use super::image_layer::ImageLayer;
use super::annotations_layer::AnnotationsLayer;
use super::models::Annotation;
use super::state::{state_load_annotations, state_update_annotation_label, state_delete_annotation};
use crate::shell::{AppNavbar, app_sidebar::{AppSidebar, SidebarTab}};
use super::keyboard_shortcuts::setup_keyboard_shortcuts;
use crate::blocks::dashboard::state::{LABELS, LABELS_LOADING, state_load_labels};
use super::context_menu::AnnotationContextMenu;


const CSS: &str = include_str!("canvas_page.css");

#[component]
pub fn AnnotationCanvasPage(
    block_id: String,
    block_name: String,
    task_id: String,
    task_name: String,
    image_id: String,
    image_name: String,
) -> Element {

    info!("Canvas Page:: mounted ::+ image_id={}", image_id);
    info!("task-name: {:?}, image_name: {:?}", task_name, image_name);

    
    // LOAD TASKS
    let block_id_tasks = block_id.clone();
    use_resource(move || {
        let bid = block_id_tasks.clone();
        async move { state_load_tasks(&bid).await }
    });

    // LOAD BLOCK LABELS
    let block_id_labels = block_id.clone();
    use_resource(move|| {
        let bid = block_id_labels.clone();
        async move {state_load_labels(&bid).await}
    });

    // Annotation-specific state
    let mut annotations: Signal<Vec<Annotation>> = use_signal(|| Vec::new());
    let mut active_drawing: Signal<Vec<(f64, f64)>> = use_signal(|| Vec::new());
    let mut image_size:Signal<(f64,f64)> = use_signal(|| (0.0,0.0));
    let cursor_world_pos:Signal<(f64,f64)> = use_signal(|| (0.0,0.0));
    let zoom: Signal<f64> = use_signal(|| 1.0);
    let sidebar_open: Signal<bool> = use_signal(|| true);
    let grid_visible: Signal<bool> = use_signal(|| false);
    let selected_tool: Signal<Tool> = use_signal(|| Tool::Pan);
    let mut context_menu:Signal<Option<(String, String, f64, f64)>> = use_signal(|| None);
    let mut selected_label_id:Signal<String> = use_signal(|| String::new());
    let mut hidden_label_ids:Signal<HashSet<String>> = use_signal(HashSet::new);
    let mut hovered_label_id:Signal<Option<String>> = use_signal(|| None);
    let mut show_shortcuts:Signal<bool> = use_signal(|| false);
    let sidebar_tab: Signal<SidebarTab> = use_signal(|| SidebarTab::Labels);
    setup_keyboard_shortcuts(sidebar_open, grid_visible, selected_tool, active_drawing, hidden_label_ids, hovered_label_id, show_shortcuts);



    // Select the first visible label, or switch if current is hidden
    use_effect(move || {
        let labels = LABELS();
        let hidden = hidden_label_ids();
        let current = selected_label_id();
        
        if labels.is_empty() { return; }
        
        // Check if current selection is empty, invalid, or hidden
        let needs_reselect = current.is_empty() 
            || !labels.iter().any(|l| l.label_id == current)
            || hidden.contains(&current);
            
        if needs_reselect {
            // Find first visible label
            if let Some(visible) = labels.iter().find(|l| !hidden.contains(&l.label_id)) {
                selected_label_id.set(visible.label_id.clone());
            }
        }
    });

    //*** - COMPONENT IS NOT REMOUNTED BUT ONLY RERENDERED ****//
    let mount_id = use_hook(move || uuid::Uuid::new_v4().to_string());
    info!("Component mount_id: {}", mount_id);

    // Load annotations whenever img id changes
    let mut last_loaded_image_id: Signal<String> = use_signal(|| String::new());
    
    if last_loaded_image_id() != image_id {
        let iid = image_id.clone();
        info!("Loading annotations for image_id: {}", iid);
        last_loaded_image_id.set(iid.clone());
        annotations.write().clear(); // good to clear but not necessary as we load annotations for the new img anyways
        spawn(async move {
            state_load_annotations(&iid, annotations).await;
        });
    }

    // Get task to find image URL
    let task: Option<Task> = TASKS.read()
        .iter()
        .find(|t| t.block_id == block_id && t.task_id == task_id)
        .cloned();

    let image_url = task.as_ref().and_then(|t| {
        t.images.iter()
            .find(|img| img.image_id == image_id)
            .map(|img| crate::shell::client::to_cloudfront_url(&img.url))
    });
    tracing::info!("image_url: {:?}", image_url);



    rsx! {
        style { {CSS} }
        AppNavbar {
            sidebar_tab: Some(sidebar_tab),
            sidebar_open: Some(sidebar_open),
        }
        div { class: "annotation-canvas-page",
            div { class: "canvas-area",
                if TASKS_LOADING() {
                    div { class: "no-image-text", "Loading..." }
                } else if task.is_none() {
                    div { class: "no-image-text", "Task not found" }
                } else if let Some(url) = image_url {
                    // SvgCanvas {
                    //     active_drawing:active_drawing,
                    //     cursor_world_pos:cursor_world_pos,
                    //     image_size:image_size,
                    //     zoom:zoom,
                    //     grid_visible:grid_visible(),
                    //     selected_tool:selected_tool(),
                    //     ImageLayer { src: url, image_size:image_size }
                    //     AnnotationsLayer {
                    //         annotations: annotations.read().clone(),
                    //         active_drawing: active_drawing(),
                    //         width:image_size().0,
                    //         height:image_size().1,
                    //         cursor_world_pos:cursor_world_pos(),
                    //         zoom:zoom(),
                    //         on_right_click: move |(ann_id, label_id, x, y)| {
                    //             context_menu.set(Some((ann_id, label_id, x, y)))
                    //         },
                    //     }
                    // }
                    SvgCanvasV2 { 
                        block_id:block_id.clone(),
                        image_id: image_id.clone(),
                        image_url: url , 
                        selected_tool:selected_tool(), 
                        active_drawing:active_drawing, 
                        annotations:annotations,
                        selected_label_id: selected_label_id(),
                        hidden_label_ids: hidden_label_ids(),
                        hovered_label_id: hovered_label_id,
                        on_annotation_context_menu:move|(ann_id, label_id, x, y)| {
                            context_menu.set(Some((ann_id, label_id, x, y)));
                        },
                    }
                } else {
                    div { class: "no-image-text", "No image" }
                }
            }
            AppSidebar{
                open:sidebar_open, 
                task_name: task_name.clone(), 
                labels:LABELS(), 
                annotations:annotations(),
                hidden_label_ids:hidden_label_ids,
                block_id: block_id.clone(),
                selected_label_id: selected_label_id,
                active_tab: sidebar_tab,
            }
        }
        // Context menu (outside canvas)
        if let Some((ann_id, label_id, x, y)) = context_menu() {
            {
                let block_id_for_delete = block_id.clone();
                let block_id_for_update = block_id.clone();
                let image_id_for_delete = image_id.clone();
                let image_id_for_update = image_id.clone();

                let ann_id_for_delete = ann_id.clone();
                let ann_id_for_update = ann_id.clone();

                rsx!{
                     AnnotationContextMenu {
                        x:x,
                        y:y,
                        current_label_id:label_id,
                        labels:LABELS(),
                        on_delete:move |_| {
                            let image_id_for_delete1 = image_id_for_delete.clone();
                            let ann_id_for_delete1 = ann_id_for_delete.clone();
                            let block_id_del = block_id_for_delete.clone();
                            // annotations.write().retain(|a| a.id != ann_id_for_delete);
                            spawn(async move {
                                state_delete_annotation(&block_id_del, &image_id_for_delete1, &ann_id_for_delete1, annotations).await;
                            });
                            context_menu.set(None);
                        },

                        on_change_label:move |new_label_id: String| {
                            let block_id_upd = block_id_for_update.clone();
                            let image_id_for_update1 = image_id_for_update.clone();
                            let ann_id_for_update1 = ann_id_for_update.clone();
                            
                            spawn(async move{
                                state_update_annotation_label(&block_id_upd, &image_id_for_update1, &ann_id_for_update1, &new_label_id, annotations).await;
                            });

                            context_menu.set(None);
                        },
                       
                       on_close:move|_| context_menu.set(None)
                    }
                }
               
            }
        }
        // Shortcuts help modal
        if show_shortcuts() {
            div {
                class: "shortcuts-modal-backdrop",
                onclick: move |_| show_shortcuts.set(false),
            }
            div {
                class: "shortcuts-modal",
                h2 { class: "shortcuts-title", "Keyboard Shortcuts" }
                div { class: "shortcuts-list",
                    div { class: "shortcut-item",
                        span { class: "shortcut-key", "P" }
                        span { class: "shortcut-desc", "Polygon draw mode" }
                    }
                    div { class: "shortcut-item",
                        span { class: "shortcut-key", "Esc" }
                        span { class: "shortcut-desc", "Cancel / Pan mode" }
                    }
                    div { class: "shortcut-item",
                        span { class: "shortcut-key", "G" }
                        span { class: "shortcut-desc", "Toggle grid" }
                    }
                    div { class: "shortcut-item",
                        span { class: "shortcut-key", "H" }
                        span { class: "shortcut-desc", "Hide hovered label" }
                    }
                    div { class: "shortcut-item",
                        span { class: "shortcut-key", "ha" }
                        span { class: "shortcut-desc", "Toggle hide all labels" }
                    }
                    div { class: "shortcut-item",
                        span { class: "shortcut-key", "⌘\\" }
                        span { class: "shortcut-desc", "Toggle sidebar" }
                    }
                    div { class: "shortcut-item",
                        span { class: "shortcut-key", "⌘+click" }
                        span { class: "shortcut-desc", "Add/remove node" }
                    }
                    div { class: "shortcut-item",
                        span { class: "shortcut-key", "Space" }
                        span { class: "shortcut-desc", "Show this help" }
                    }
                }
                div { class: "shortcuts-footer", "Press Space or click outside to close" }
            }
        }
    }
}
