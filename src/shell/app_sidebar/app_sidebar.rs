use dioxus::prelude::*;
use std::collections::HashSet;
use crate::shell::{THEME, Theme};
use crate::blocks::dashboard::api::{BlockLabel, api_update_label_color};
use crate::blocks::annotations::models::{Annotation, CommentThread};
use crate::atoms::svg_canvas::state::Tool;
use crate::users::state::USER;

const BBOX_ICON_LIGHT: Asset = asset!("/assets/icons/bbox-light.svg");
const BBOX_ICON_DARK: Asset = asset!("/assets/icons/bbox-dark.svg");
const POLYGON_ICON_LIGHT: Asset = asset!("/assets/icons/polygon-light.svg");
const POLYGON_ICON_DARK: Asset = asset!("/assets/icons/polygon-dark.svg");



const LABELS_ICON_LIGHT: Asset = asset!("/assets/icons/labels-light.svg");
const LABELS_ICON_DARK: Asset = asset!("/assets/icons/labels-dark.svg");
const COMMENTS_ICON_LIGHT: Asset = asset!("/assets/icons/comment-light.svg");
const COMMENTS_ICON_DARK: Asset = asset!("/assets/icons/comment-dark.svg");

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SidebarTab {
    Labels,
    Comments,
}

#[component]
pub fn AppSidebar(
    open: Signal<bool>, 
    task_name: String, 
    labels: Vec<BlockLabel>,
    annotations:Vec<Annotation>,
    comment_threads: Vec<CommentThread>,
    hidden_label_ids:Signal<HashSet<String>>,
    block_id: String,
    selected_label_id: Signal<String>,
    active_tab: Signal<SidebarTab>,
    scroll_to_thread: Signal<Option<String>>,
    #[props(default)] selected_tool: Option<Signal<Tool>>,
    ) -> Element {
    let is_dark = THEME() == Theme::Dark;
    let labels_icon = if is_dark { LABELS_ICON_DARK } else { LABELS_ICON_LIGHT };
    let comments_icon = if is_dark { COMMENTS_ICON_DARK } else { COMMENTS_ICON_LIGHT };
    let is_admin = USER.read().as_ref().map(|u| u.is_admin()).unwrap_or(false);
    
    // Load user if not already loaded - runs only once
    // use_hook(|| {
    //     spawn(async move {
    //         if USER.read().is_none() {
    //             load_user().await;
    //         }
    //     });
    // });
   

    let mut comment_threads = comment_threads;
    comment_threads.sort_by_key(|thread| thread.resolved);

    rsx! {
        div {
            class: if open() { "app-sidebar open" } else { "app-sidebar" },
            
            // Labels table
            if active_tab() == SidebarTab::Labels {
                div {
                    class: "sidebar-labels-table",
                   for label in labels.iter(){{
                        let count = annotations.iter().filter(|a| a.label_id == label.label_id).count();
                        let is_hidden = hidden_label_ids().contains(&label.label_id);
                        let is_selected = selected_label_id() == label.label_id;
                        let lid = label.label_id.clone();
                        let label_color = label.label_color.clone();
                        let block_id_for_update = block_id.clone();

                        let mut row_class = String::from("labels-table-row");
                        if is_selected { row_class.push_str(" selected"); }
                        if is_hidden { row_class.push_str(" hidden-label"); }
                        if count == 0 { row_class.push_str(" empty-label"); }
                        
                        let lid_select = lid.clone();
                        let lid_toggle = lid.clone();
                        let lid_color = lid.clone();
                        
                        rsx!{
                            div {
                                class: row_class,
                                key:"{lid}",
                                style:"--label-color: {label_color};",
                                // Click anywhere except label name = select
                                onclick: move |_| {
                                    selected_label_id.set(lid_select.clone());
                                    // Auto-switch tool based on label_properties
                                    if let Some(mut tool_sig) = selected_tool {
                                        if let Some(label) = crate::blocks::dashboard::state::LABELS.read().iter().find(|l| l.label_id == lid_select) {
                                            let tool_str = label.label_properties.as_ref()
                                                .and_then(|p| p.get("tool"))
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("polygon");
                                            match tool_str {
                                                "bbox" => tool_sig.set(Tool::BBox),
                                                _ => tool_sig.set(Tool::Polygon),
                                            }
                                        }
                                    }
                                },
                                div {
                                    class: "label-color-rect-wrapper",
                                    onclick: move |e| e.stop_propagation(), // Don't select when clicking color
                                    if is_admin {
                                        input {
                                            r#type: "color",
                                            class: "label-color-input",
                                            value: "{label_color}",
                                            onchange: move |e| {
                                                let new_color = e.value();
                                                let lid = lid_color.clone();
                                                let bid = block_id_for_update.clone();
                                                spawn(async move {
                                                    match api_update_label_color(&bid, &lid, new_color.clone()).await {
                                                        Ok(updated) => {
                                                            crate::blocks::dashboard::state::LABELS.write().iter_mut().for_each(|l| {
                                                                if l.label_id == lid {
                                                                    l.label_color = updated.label_color.clone();
                                                                }
                                                            });
                                                        }
                                                        Err(e) => tracing::error!("Failed to update color: {}", e),
                                                    }
                                                });
                                            }
                                        }
                                    }
                                }
                                span {
                                    class:"labels-table-cell label-name",
                                    onclick: move |e| {
                                        e.stop_propagation(); // Don't trigger row select
                                        // Toggle visibility
                                        let mut h = hidden_label_ids.write();
                                        if h.contains(&lid_toggle) {
                                            h.remove(&lid_toggle);
                                        } else {
                                            h.insert(lid_toggle.clone());
                                        }
                                    },
                                    "{label.label_name}"
                                    if is_selected {
                                        span { class: "label-active-dot" }
                                    }
                                }
                                span {class:"labels-table-cell label-count", "{count:02}" }
                                // Geometry type indicators (only show > 0)
                                {{
                                    let bbox_count = annotations.iter().filter(|a| a.label_id == lid && matches!(a.geometry, crate::atoms::svg_canvas::Geometry::BBox { .. })).count();
                                    let poly_count = annotations.iter().filter(|a| a.label_id == lid && matches!(a.geometry, crate::atoms::svg_canvas::Geometry::Polygon { .. })).count();
                                    let bbox_icon = if is_dark { BBOX_ICON_DARK } else { BBOX_ICON_LIGHT };
                                    let poly_icon = if is_dark { POLYGON_ICON_DARK } else { POLYGON_ICON_LIGHT };
                                    rsx! {
                                        div {
                                            class: "label-geom-types",
                                            if bbox_count > 0 {
                                                div { class: "label-geom-group",
                                                    img { src: bbox_icon, class: "label-geom-icon" }
                                                    span { class: "label-geom-tag", "{bbox_count}" }
                                                }
                                            }
                                            if poly_count > 0 {
                                                div { class: "label-geom-group",
                                                    img { src: poly_icon, class: "label-geom-icon-poly" }
                                                    span { class: "label-geom-tag", "{poly_count}" }
                                                }
                                            }
                                        }
                                    }
                                }}
                            }
                        }
                   }}
                   // Total row
                   {{
                       let total_bbox = annotations.iter().filter(|a| matches!(a.geometry, crate::atoms::svg_canvas::Geometry::BBox { .. })).count();
                       let total_poly = annotations.iter().filter(|a| matches!(a.geometry, crate::atoms::svg_canvas::Geometry::Polygon { .. })).count();
                       rsx! {
                           div {
                               class: "labels-table-row total-row",
                               div { class: "label-color-rect-wrapper" }
                               span { class: "labels-table-cell label-name total-label", "# total" }
                               span { class: "labels-table-cell label-count", "{annotations.len():02}" }
                               {{
                                   let bbox_icon = if is_dark { BBOX_ICON_DARK } else { BBOX_ICON_LIGHT };
                                   let poly_icon = if is_dark { POLYGON_ICON_DARK } else { POLYGON_ICON_LIGHT };
                                   rsx! {
                                       div {
                                           class: "label-geom-types",
                                           if total_bbox > 0 {
                                               div { class: "label-geom-group",
                                                   img { src: bbox_icon, class: "label-geom-icon" }
                                                   span { class: "label-geom-tag", "{total_bbox}" }
                                               }
                                           }
                                           if total_poly > 0 {
                                               div { class: "label-geom-group",
                                                   img { src: poly_icon, class: "label-geom-icon-poly" }
                                                   span { class: "label-geom-tag", "{total_poly}" }
                                               }
                                           }
                                       }
                                   }
                               }}
                           }
                       }
                   }}
                }
            }

            if active_tab() == SidebarTab::Comments {
                div {
                    class: "sidebar-comments-list",
                    if comment_threads.is_empty() {
                        div { class: "sidebar-comments-empty", "No comments yet" }
                    }
                    for thread in comment_threads.iter() {
                        {{
                            let tid = thread.id.clone();
                            let first_comment = thread.comments.first();
                            let comment_count = thread.comments.len();
                            let is_resolved = thread.resolved;
                            rsx! {
                                div {
                                    key: "{tid}",
                                    class: if is_resolved { "sidebar-comment-item resolved" } else { "sidebar-comment-item" },
                                    style: "cursor: pointer;",
                                    onclick: {
                                        let tid = tid.clone();
                                        move |_| {
                                            scroll_to_thread.set(Some(tid.clone()));
                                        }
                                    },
                                    if let Some(comment) = first_comment {
                                        div {
                                            class: "sidebar-comment-meta",
                                            span { class: "sidebar-comment-author", "{comment.user_name}" }
                                            span { class: "sidebar-comment-time", "{comment.created_at}" }
                                        }
                                        div { class: "sidebar-comment-text", "{comment.text}" }
                                    }
                                    if comment_count > 1 {
                                        div { class: "sidebar-comment-replies", "{comment_count - 1} replies" }
                                    }
                                }
                            }
                        }}
                    }
                }
            }
            
            // Bottom section with version
            div {
                class: "app-sidebar-bottom-section",
                div {
                    class: "app-sidebar-version",
                    "v1.06"
                }
            }
        }
    }
}
