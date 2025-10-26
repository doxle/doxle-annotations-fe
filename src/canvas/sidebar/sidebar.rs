use dioxus::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{window, MouseEvent};

use super::header::SidebarHeader;
use super::tabs::Tabs;
use super::types::SidebarTab;
use super::classes_panel::ClassesPanel;
use super::comments_panel::CommentsPanel;
use super::storage::{load_f32, save_f32};

const _SIDEBAR_CSS: Asset = asset!("/src/canvas/sidebar/sidebar.css");

#[component]
pub fn Sidebar(sidebar_open: Signal<bool>, class_counter: Signal<u64>) -> Element {
    let mut active_tab = use_signal(|| SidebarTab::Classes);

    // Sidebar width with persistence
    let mut width_px = use_signal(|| load_f32("sidebar_width_px").unwrap_or(259.0));
    let mut is_resizing = use_signal(|| false);

    let on_resize_mousedown = move |_e: Event<MouseData>| {
        is_resizing.set(true);
        if let Some(win) = window() {
let move_cb = {
                let mut width_px = width_px.clone();
                let mut is_resizing = is_resizing.clone();
                Closure::wrap(Box::new(move |ev: MouseEvent| {
                    if !is_resizing() { return; }
                    let vw = web_sys::window()
                        .and_then(|w| w.inner_width().ok())
                        .and_then(|w| w.as_f64())
                        .unwrap_or(1200.0) as f32;
                    let x = ev.client_x() as f32;
                    let new_w = (vw - x).clamp(240.0, 600.0);
                    width_px.set(new_w);
                    save_f32("sidebar_width_px", new_w);
                }) as Box<dyn FnMut(_)> )
            };
let up_cb = {
                let mut is_resizing = is_resizing.clone();
                Closure::wrap(Box::new(move |_ev: MouseEvent| { is_resizing.set(false); }) as Box<dyn FnMut(_)> )
            };
            let _ = win.add_event_listener_with_callback("mousemove", move_cb.as_ref().unchecked_ref());
            let _ = win.add_event_listener_with_callback("mouseup", up_cb.as_ref().unchecked_ref());
            move_cb.forget();
            up_cb.forget();
        }
    };

    // Mock until wired from route
    let block_label = format!("Block#{}", 150);
    let images_count: u32 = 50;

    rsx! {
        div {
            class: if sidebar_open() { "canvas-sidebar open" } else { "canvas-sidebar" },
            style: format_args!("width: {}px;", width_px()),

            // Left edge resize handle
            div { class: "sidebar-resize-handle", onmousedown: on_resize_mousedown }

            // Header
            SidebarHeader { block_label: block_label.clone(), images_count: images_count }

            // Tabs
            Tabs { active_tab: active_tab }

            // Divider
            div { class: "sidebar-divider" }

            // Content
            match active_tab() {
                SidebarTab::Classes => rsx! { ClassesPanel { class_counter: class_counter } },
                SidebarTab::Comments => rsx! { CommentsPanel {} },
            }
        }
    }
}
