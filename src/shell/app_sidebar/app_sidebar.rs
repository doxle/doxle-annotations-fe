use dioxus::prelude::*;
use std::collections::HashSet;
use crate::shell::{THEME, Theme};
use crate::blocks::dashboard::api::{BlockLabel, api_update_label_color};
use crate::blocks::annotations::models::Annotation;



const LABELS_ICON_LIGHT: Asset = asset!("/assets/icons/labels-light.svg");
const LABELS_ICON_DARK: Asset = asset!("/assets/icons/labels-dark.svg");
const COMMENTS_ICON_LIGHT: Asset = asset!("/assets/icons/comment-light.svg");
const COMMENTS_ICON_DARK: Asset = asset!("/assets/icons/comment-dark.svg");

#[derive(Clone, Copy, PartialEq)]
enum SidebarTab {
    Labels,
    Comments,
}

#[component]
pub fn AppSidebar(
    open: Signal<bool>, 
    task_name: String, 
    labels: Vec<BlockLabel>,
    annotations:Vec<Annotation>,
    hidden_label_ids:Signal<HashSet<String>>,
    block_id: String,
    selected_label_id: Signal<String>,
    ) -> Element {
    let is_dark = THEME() == Theme::Dark;
    let labels_icon = if is_dark { LABELS_ICON_DARK } else { LABELS_ICON_LIGHT };
    let comments_icon = if is_dark { COMMENTS_ICON_DARK } else { COMMENTS_ICON_LIGHT };
    let mut active_tab = use_signal(|| SidebarTab::Labels);
    let nav = navigator();
    
    // Load user if not already loaded - runs only once
    // use_hook(|| {
    //     spawn(async move {
    //         if USER.read().is_none() {
    //             load_user().await;
    //         }
    //     });
    // });
   

    rsx! {
        div {
            class: if open() { "app-sidebar open" } else { "app-sidebar" },
            
            // Labels and Comments tabs (at top)
            div {
                class: "app-sidebar-tabs",
                div {
                    class: if active_tab() == SidebarTab::Labels { "app-sidebar-tab active" } else { "app-sidebar-tab" },
                    onclick: move |_| active_tab.set(SidebarTab::Labels),
                    "Labels"
                }
                div {
                    class: if active_tab() == SidebarTab::Comments { "app-sidebar-tab active" } else { "app-sidebar-tab" },
                    onclick: move |_| active_tab.set(SidebarTab::Comments),
                    "Comments"
                }
            }
            
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

                        let row_class = match (is_selected, is_hidden, count == 0) {
                            (true, _, _) => "labels-table-row selected",
                            (false, true, _) => "labels-table-row hidden-label",
                            (false, false, true) => "labels-table-row empty-label",
                            _ => "labels-table-row",
                        };
                        
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
                                },
                                div {
                                    class: "label-color-rect-wrapper",
                                    onclick: move |e| e.stop_propagation(), // Don't select when clicking color
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
                                }
                                span {class:"labels-table-cell label-equals", "=" }
                                span {class:"labels-table-cell label-count", "{count:02}" }
                            }
                        }
                   }}
                   // Total row
                   div {
                       class: "labels-table-row total-row",
                       div { class: "label-color-rect-wrapper" } // Empty placeholder for color column
                       span { class: "labels-table-cell label-name total-label", "# total" }
                       span { class: "labels-table-cell label-equals", "=" }
                       span { class: "labels-table-cell label-count", "{annotations.len():02}" }
                   }
                }
            }
            
            // Bottom section with logout and version
            div {
                class: "app-sidebar-bottom-section",
                
                // Logout button
                div {
                    class: "app-sidebar-logout",
                    onclick: move |_| {
                        // Clear auth token
                       
                        nav.push(crate::Route::HomePage {});
                    },
                    "Logout"
                }
                
                div {
                    class: "app-sidebar-version",
                    "v1.06"
                }
            }
        }
    }
}
