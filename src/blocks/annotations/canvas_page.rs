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


use std::collections::{HashSet, HashMap};
use dioxus::prelude::*;
use dioxus::logger::tracing::info;
use crate::atoms::svg_canvas::{SvgCanvas, SvgCanvasV2, Geometry};
use crate::atoms::svg_canvas::state::Tool;
use crate::atoms::tasks::state::{TASKS, TASKS_LOADING, state_load_tasks};
use crate::atoms::tasks::model::Task;
use super::image_layer::ImageLayer;
use super::annotations_layer::AnnotationsLayer;
use super::models::Annotation;
use super::state::{state_load_annotations, state_update_annotation_label, state_delete_annotation, state_load_threads, state_create_thread, state_add_comment, state_delete_thread, state_resolve_thread};
use crate::shell::{AppNavbar, app_sidebar::{AppSidebar, SidebarTab}};
use crate::shell::loading::LoadingPage;
use super::keyboard_shortcuts::setup_keyboard_shortcuts;
use crate::blocks::dashboard::state::{LABELS, LABELS_LOADING, state_load_labels};
use crate::blocks::dashboard::state::CURRENT_BLOCK;
use super::context_menu::AnnotationContextMenu;
use super::comment_dialog::CommentDialog;
use super::models::CommentThread;
use crate::Route;

const CSS: &str = include_str!("canvas_page.css");

#[component]
pub fn AnnotationCanvasPage(
    block_id: String,
    block_name: String,
    block_type: String,
    task_id: String,
    task_name: String,
    image_id: String,
    image_name: String,
) -> Element {

    info!("Canvas Page:: mounted ::+ image_id={}", image_id);
    info!("task-name: {:?}, image_name: {:?}", task_name, image_name);
    info!("🔍 PROP block_id = {}", block_id);
    

    
    // Use cached tasks — only fetch if not loaded for this block
    let block_id_tasks = block_id.clone();
    use_effect(move || {
        let current_bid = crate::blocks::dashboard::state::CURRENT_BLOCK().map(|b| b.block_id.clone()).unwrap_or_default();
        if current_bid != block_id_tasks {
            let bid = block_id_tasks.clone();
            spawn(async move {
                state_load_tasks(&bid).await;
            });
        }
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
    let mut selected_tool: Signal<Tool> = use_signal(|| Tool::Select);
    let mut context_menu:Signal<Option<(String, String, f64, f64)>> = use_signal(|| None);
    let mut selected_label_id:Signal<String> = use_signal(|| String::new());
    let mut hidden_label_ids:Signal<HashSet<String>> = use_signal(HashSet::new);
    let mut hovered_label_id:Signal<Option<String>> = use_signal(|| None);
    let mut show_shortcuts:Signal<bool> = use_signal(|| false);
    let sidebar_tab: Signal<SidebarTab> = use_signal(|| SidebarTab::Labels);
    let annotation_opacity: Signal<f64> = use_signal(|| 0.2);
    let mut comment_threads: Signal<Vec<CommentThread>> = use_signal(|| Vec::new());
    let mut comment_dialog: Signal<Option<(f64, f64, f64, f64, String)>> = use_signal(|| None); // (world_x, world_y, screen_x, screen_y, thread_id)
    let mut scroll_to_thread: Signal<Option<String>> = use_signal(|| None);
    let mut nav_direction: Signal<Option<i32>> = use_signal(|| None);
    let mut clipboard: Signal<Option<(Geometry, String)>> = use_signal(|| None);
    let mut paste_mode: Signal<bool> = use_signal(|| false);
    let mut copy_requested: Signal<u32> = use_signal(|| 0);
    setup_keyboard_shortcuts(sidebar_open, grid_visible, selected_tool, active_drawing, hidden_label_ids, hovered_label_id, show_shortcuts, sidebar_tab, nav_direction, copy_requested, paste_mode);



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

    // Navigate to next/prev image on f/b key press
    let nav = use_navigator();
    let nav_block_id = block_id.clone();
    let nav_block_name = block_name.clone();
    let nav_block_type = block_type.clone();
    let nav_task_id = task_id.clone();
    let nav_task_name = task_name.clone();
    let nav_image_id = image_id.clone();
    use_effect(move || {
        if let Some(dir) = nav_direction() {
            nav_direction.set(None);
            let tasks = TASKS.read();
            if let Some(task) = tasks.iter().find(|t| t.task_id == nav_task_id) {
                if let Some(idx) = task.images.iter().position(|img| img.image_id == nav_image_id) {
                    let new_idx = idx as i32 + dir;
                    if new_idx >= 0 && (new_idx as usize) < task.images.len() {
                        let new_img = &task.images[new_idx as usize];
                        nav.replace(Route::AnnotationCanvasPage {
                            block_id: nav_block_id.clone(),
                            block_name: nav_block_name.clone(),
                            block_type: nav_block_type.clone(),
                            task_id: nav_task_id.clone(),
                            task_name: nav_task_name.clone(),
                            image_id: new_img.image_id.clone(),
                            image_name: format!("{}.png", new_img.image_id),
                        });
                    }
                }
            }
        }
    });

    //*** - COMPONENT IS NOT REMOUNTED BUT ONLY RERENDERED ****//
    let mount_id = use_hook(move || uuid::Uuid::new_v4().to_string());
    info!("Component mount_id: {}", mount_id);

    // Load annotations whenever img id changes — cache per image for instant back/forward
    let mut last_loaded_image_id: Signal<String> = use_signal(|| String::new());
    let mut ann_cache: Signal<HashMap<String, Vec<Annotation>>> = use_signal(HashMap::new);
    let mut thread_cache: Signal<HashMap<String, Vec<CommentThread>>> = use_signal(HashMap::new);
    
    if last_loaded_image_id() != image_id {
        let old_iid = last_loaded_image_id();
        // Save current data to cache before switching
        if !old_iid.is_empty() {
            ann_cache.write().insert(old_iid.clone(), annotations().clone());
            thread_cache.write().insert(old_iid.clone(), comment_threads().clone());
        }

        let iid = image_id.clone();
        info!("Loading annotations for image_id: {}", iid);
        last_loaded_image_id.set(iid.clone());

        // Check cache first — instant if previously visited
        if let Some(cached_anns) = ann_cache.read().get(&iid).cloned() {
            *annotations.write() = cached_anns;
            info!("✅ Annotations loaded from cache");
        } else {
            annotations.write().clear();
            let iid_clone = iid.clone();
            spawn(async move {
                state_load_annotations(&iid_clone, annotations).await;
            });
        }

        if let Some(cached_threads) = thread_cache.read().get(&iid).cloned() {
            *comment_threads.write() = cached_threads;
            info!("✅ Threads loaded from cache");
        } else {
            comment_threads.write().clear();
            let iid_clone = iid.clone();
            spawn(async move {
                state_load_threads(&iid_clone, comment_threads).await;
            });
        }
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

    info!("Canvas render: LABELS count = {}", LABELS().len());

    rsx! {
        style { {CSS} }
        AppNavbar {
            sidebar_tab: Some(sidebar_tab),
            sidebar_open: Some(sidebar_open),
            selected_tool: Some(selected_tool),
            annotation_opacity: Some(annotation_opacity),
        }
        div { class: "annotation-canvas-page",
            div { class: "canvas-area",
                if TASKS_LOADING() {
                    LoadingPage {}
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
                        comment_threads: comment_threads,
                        selected_label_id: selected_label_id(),
                        hidden_label_ids: hidden_label_ids(),
                        hovered_label_id: hovered_label_id,
                        active_thread_id: comment_dialog().map(|(_, _, _, _, ref tid)| tid.clone()),
                        scroll_to_thread: scroll_to_thread,
                        show_comments: sidebar_tab() == SidebarTab::Comments,
                        annotation_opacity: annotation_opacity(),
                        clipboard: clipboard,
                        paste_mode: paste_mode,
                        copy_requested: copy_requested,
                        on_annotation_context_menu:move|(ann_id, label_id, x, y)| {
                            context_menu.set(Some((ann_id, label_id, x, y)));
                        },
                        on_comment_click: {
                            let image_id = image_id.clone();
                            move |(wx, wy, sx, sy)| {
                            // Generate UUID locally, add placeholder thread, open dialog instantly
                            let tid = uuid::Uuid::new_v4().to_string();
                            comment_threads.write().push(CommentThread {
                                id: tid.clone(),
                                world_x: wx,
                                world_y: wy,
                                resolved: false,
                                comments: Vec::new(),
                                persisted: false,
                            });
                            comment_dialog.set(Some((wx, wy, sx, sy, tid)));
                        }},
                        on_marker_click: move |(tid, sx, sy): (String, f64, f64)| {
                            // Clean up existing empty thread if dialog is open
                            if let Some((_, _, _, _, ref old_tid)) = comment_dialog() {
                                let old_tid = old_tid.clone();
                                comment_threads.write().retain(|t| t.id != old_tid || !t.comments.is_empty());
                            }
                            if let Some(t) = comment_threads.read().iter().find(|t| t.id == tid).cloned() {
                                comment_dialog.set(Some((t.world_x, t.world_y, sx, sy, tid)));
                                selected_tool.set(Tool::Comment);
                            }
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
                comment_threads: comment_threads(),
                hidden_label_ids:hidden_label_ids,
                block_id: block_id.clone(),
                selected_label_id: selected_label_id,
                active_tab: sidebar_tab,
                scroll_to_thread: scroll_to_thread,
                selected_tool: Some(selected_tool),
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
        // Comment dialog (floating on canvas)
        if let Some((world_x, world_y, screen_x, screen_y, thread_id)) = comment_dialog() {
            {
                let thread = comment_threads.read().iter().find(|t| t.id == thread_id).cloned();
                let image_id_for_post = image_id.clone();
                let image_id_for_resolve = image_id.clone();
                let image_id_for_del = image_id.clone();
                let image_id_for_close = image_id.clone();
                rsx! {
                    CommentDialog {
                        screen_x: screen_x,
                        screen_y: screen_y,
                        thread: thread,
                        on_post: move |text: String| {
                            let iid = image_id_for_post.clone();
                            let tid = thread_id.clone();
                            // Check if thread has comments — if not, this is the first post (create on server)
                            let has_comments = comment_threads.read().iter()
                                .find(|t| t.id == tid)
                                .map_or(false, |t| !t.comments.is_empty());
                            if has_comments {
                                // Existing thread — just add comment
                                spawn(async move {
                                    state_add_comment(&iid, &tid, &text, comment_threads).await;
                                });
                            } else {
                                // First comment — create thread + comment on server
                                let (wx, wy) = (world_x, world_y);
                                spawn(async move {
                                    let ok = state_create_thread(&iid, &tid, wx, wy, &text, comment_threads).await;
                                    if !ok {
                                        crate::shell::progress::show_error("Failed to save comment");
                                    }
                                });
                            }
                            // Close dialog immediately — server persists in the background
                            comment_dialog.set(None);
                        },
                        on_resolve: move |_| {
                            if let Some((_, _, _, _, ref tid)) = comment_dialog() {
                                let tid = tid.clone();
                                let iid = image_id_for_resolve.clone();
                                spawn(async move {
                                    state_resolve_thread(&iid, &tid, comment_threads).await;
                                });
                            }
                        },
                        on_delete: move |_| {
                            if let Some((_, _, _, _, ref tid)) = comment_dialog() {
                                let tid = tid.clone();
                                let iid = image_id_for_del.clone();
                                comment_dialog.set(None);
                                spawn(async move {
                                    state_delete_thread(&iid, &tid, comment_threads).await;
                                });
                            }
                        },
                        on_close: move |_| {
                            // Remove local-only thread if user closed without posting
                            if let Some((_, _, _, _, ref tid)) = comment_dialog() {
                                let tid = tid.clone();
                                let has_comments = comment_threads.read().iter()
                                    .find(|t| t.id == tid)
                                    .map_or(false, |t| !t.comments.is_empty());
                                if !has_comments {
                                    // Thread was never sent to server — just remove locally
                                    comment_threads.write().retain(|t| t.id != tid);
                                }
                            }
                            comment_dialog.set(None);
                            selected_tool.set(Tool::Select);
                        },
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
                        span { class: "shortcut-key", "F" }
                        span { class: "shortcut-desc", "Next image (forward)" }
                    }
                    div { class: "shortcut-item",
                        span { class: "shortcut-key", "D" }
                        span { class: "shortcut-desc", "Previous image (backward)" }
                    }
                    div { class: "shortcut-item",
                        span { class: "shortcut-key", "⌘C" }
                        span { class: "shortcut-desc", "Copy selected annotation" }
                    }
                    div { class: "shortcut-item",
                        span { class: "shortcut-key", "⌘V" }
                        span { class: "shortcut-desc", "Paste annotation (follows cursor)" }
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
