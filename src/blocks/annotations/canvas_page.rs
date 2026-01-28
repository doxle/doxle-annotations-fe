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
use crate::shell::{AppNavbar, app_sidebar::AppSidebar};
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

    info!("Canvas Page:: mounted ::+");
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
    setup_keyboard_shortcuts(sidebar_open, grid_visible, selected_tool, active_drawing);
    let mut context_menu:Signal<Option<(String, String, f64, f64)>> = use_signal(|| None);
    let mut selected_label_id:Signal<String> = use_signal(|| String::new());
    let mut hidden_label_ids:Signal<HashSet<String>> = use_signal(HashSet::new);





    // Select the first label id
    use_effect(move || {
        let labels = LABELS();
        if selected_label_id().is_empty() && !labels.is_empty() {
            selected_label_id.set(labels[0].label_id.clone());
        }      
    });


    // Load annotations on mount
    let iid = image_id.clone();

    use_hook(move || {
        let id = iid.clone();
        let out = annotations;
        spawn(async move {
            state_load_annotations(&id, out).await;
        });
    });

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
        AppNavbar {}
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
            }
        }
        // Context menu (outside canvas)
        if let Some((ann_id, label_id, x, y)) = context_menu() {
            {
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
                            let block_id_del = block_id.clone();
                            // annotations.write().retain(|a| a.id != ann_id_for_delete);
                            spawn(async move {
                                state_delete_annotation(&block_id_del, &image_id_for_delete1, &ann_id_for_delete1, annotations).await;
                            });
                            context_menu.set(None);
                        },

                        on_change_label:move |new_label_id: String| {
                            let image_id_for_update1 = image_id_for_update.clone();
                            let ann_id_for_update1 = ann_id_for_update.clone();
                            
                            // let mut anns = annotations.write();
                            // if let Some(ann) = anns.iter_mut().find(|a| a.id == ann_id_for_change) {
                            //     ann.label_id = new_label_id.clone();
                            // }
                            spawn(async move{
                                state_update_annotation_label(&image_id_for_update1, &ann_id_for_update1, &new_label_id, annotations).await;
                            });

                            context_menu.set(None);
                        },
                       
                       on_close:move|_| context_menu.set(None)
                    }
                }
               
            }
        }
    }
}
