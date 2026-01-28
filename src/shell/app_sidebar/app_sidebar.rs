use dioxus::prelude::*;
use std::collections::HashSet;
use crate::shell::{THEME, Theme};
use crate::blocks::dashboard::api::BlockLabel;
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
    labels:Vec<BlockLabel>,
    annotations:Vec<Annotation>,
    hidden_label_ids:Signal<HashSet<String>>,

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
                    div { class: "sidebar-icon-wrapper",
                        img { src: labels_icon, class: "sidebar-label-icon", alt: "Labels" }
                    }
                }
                div {
                    class: if active_tab() == SidebarTab::Comments { "app-sidebar-tab active" } else { "app-sidebar-tab" },
                    onclick: move |_| active_tab.set(SidebarTab::Comments),
                    div { class: "sidebar-icon-wrapper",
                        img { src: comments_icon, class: "sidebar-comment-icon", alt: "Comments" }
                    }
                }
            }
            
            // Labels table
            if active_tab() == SidebarTab::Labels {
                div {
                    class: "sidebar-labels-table",
                   for (idx, label) in labels.iter().enumerate(){{
                        let count = annotations.iter().filter(|a| a.label_id == label.label_id).count();
                        let is_hidden = hidden_label_ids().contains(&label.label_id);
                        let lid = label.label_id.clone();
                        let num = idx + 1;

                        rsx!{
                            div {
                                class:"labels-table-row",
                                key:"{label.label_id}",
                                style:"--label-color: {label.label_color};",
                                span{
                                    class: "labels-table-cell visibility-toggle",
                                    onclick: move |_| {
                                        let mut h = hidden_label_ids.write();
                                        if h.contains(&lid) { h.remove(&lid); } 
                                        else { h.insert(lid.clone()); }
                                    },
                                    if is_hidden { "⊘" } else { "👁️" }

                                }
                                span {class:"label-color-rect"}
                                span {class:"labels-table-cell label-name", "{label.label_name}" }
                                span {class:"labels-table-cell label-equals", "=" }
                                span {class:"labels-table-cell label-count", "{count:02}" }
                            }
                        }
                   }}
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
