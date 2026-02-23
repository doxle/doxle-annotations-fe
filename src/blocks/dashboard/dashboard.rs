use crate::Route;
use dioxus::prelude::*;
use crate::shell::{THEME, Theme, AppNavbar, ProtectedRoute};
use crate::blocks::dashboard::state::{BLOCKS, BLOCKS_LOADING, state_load_blocks, state_load_blocks_silent,  state_delete_block, state_set_current_block, state_rename_block};
use crate::blocks::dashboard::api::BlockType;
use crate::users::state::USER;
use crate::users::api::UserRole;
use super::block_menu::BlockMenu;
use super::edit_block_modal::EditBlockModal;

const DASHBOARD_CSS: &str = include_str!("dashboard.css");
const BLOCKS_ICON_LIGHT: Asset = asset!("/assets/icons/blocks-light.svg");
const BLOCKS_ICON_DARK: Asset = asset!("/assets/icons/blocks-dark.svg");
const TRASH_ICON_LIGHT: Asset = asset!("/assets/icons/trash-light.svg");
const TRASH_ICON_DARK: Asset = asset!("/assets/icons/trash-dark.svg");
const ANNOTATION_BLOCK_ICON: Asset = asset!("/assets/icons/annotation-block-badge.svg");
const ADD_ICON_LIGHT: Asset = asset!("/assets/icons/add-light.svg");
const ADD_ICON_DARK: Asset = asset!("/assets/icons/add-dark.svg");
const CARD_ICON_LIGHT: Asset = asset!("/assets/icons/card-light.svg");
const CARD_ICON_DARK: Asset = asset!("/assets/icons/card-dark.svg");


fn format_date(rfc3339: &str) -> String {
    if rfc3339.is_empty() { return "—".to_string(); }
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(rfc3339));
    let months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    let month = months[date.get_month() as usize];
    format!("{} {}, {}", month, date.get_date(), date.get_full_year())
}

fn format_relative_time(rfc3339: &str) -> String {
    if rfc3339.is_empty() { return "—".to_string(); }
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(rfc3339));
    let now = js_sys::Date::new_0();
    let diff_secs = ((now.get_time() - date.get_time()) / 1000.0) as u64;
    match diff_secs {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{} min ago", diff_secs / 60),
        3600..=86399 => format!("{} hours ago", diff_secs / 3600),
        86400..=2591999 => format!("{} days ago", diff_secs / 86400),
        _ => format_date(rfc3339),
    }
}

#[component]
pub fn DashboardPage()->Element{
	// Get current theme
	let is_dark = THEME() == Theme::Dark;
	let blocks_icon = if is_dark { BLOCKS_ICON_DARK } else { BLOCKS_ICON_LIGHT };
	let trash_icon = if is_dark { TRASH_ICON_DARK } else { TRASH_ICON_LIGHT };
	let add_icon = if is_dark { ADD_ICON_DARK } else { ADD_ICON_LIGHT };
	let card_icon = if is_dark { CARD_ICON_DARK } else { CARD_ICON_LIGHT };
    let navigator = use_navigator();
    let mut open_menu_id: Signal<Option<String>> = use_signal(|| None);
    let mut editing_block: Signal<Option<(String, String)>> = use_signal(|| None); // (block_id, current_name)
    let mut deleting_block_id: Signal<Option<String>> = use_signal(|| None);

    info!("Loading dashboard page: ");

    // Load blocks on mount
    use_resource(move || async move {
        if BLOCKS.peek().is_empty() {
            info!("Loading dashboard page - first load");
            state_load_blocks().await;
        }
        else {
            info!("Loading dashboard page - silent refresh");
            state_load_blocks_silent().await;
        }
    });



    // Show loading or existing blocks
    info!("BLOCKS-LIST Page :-");

    // Filter blocks by role: builders see only File + Building
    let user_role = USER.read().as_ref().map(|u| u.user_role.clone()).unwrap_or(UserRole::Annotator);
    let is_builder = user_role == UserRole::Builder;

    // Show loading 
    if BLOCKS_LOADING() {
        return rsx! {
            ProtectedRoute {
                style { {DASHBOARD_CSS} }
                AppNavbar {}
                div { class: "blocks-page blocks-page-loading",
                    div { class: "blocks-loading", "Loading blocks..." }
                }
            }
        };
    }

    let mut filtered_blocks: Vec<_> = BLOCKS.read().iter().filter(|b| {
        if is_builder {
            b.block_type == BlockType::File || b.block_type == BlockType::Building
        } else {
            true
        }
    }).cloned().collect();
    filtered_blocks.sort_by(|a, b| b.block_created_at.cmp(&a.block_created_at));

    // If no blocks, show centered "+ NEW BLOCK" button
    if filtered_blocks.is_empty() {
        return rsx! {
            ProtectedRoute {
                style { {DASHBOARD_CSS} }
                AppNavbar {}
                div { class: "blocks-empty-page",
                    button {
                        class: "blocks-create-button-centered",
                        onclick: move |_| {
                            navigator.push(Route::CreateBlockPage {});
                        },
                        img { src: add_icon, class: "blocks-create-icon" }
                        "New Block"
                    }
                }
            }
        };
    }
    
    // If blocks exist, show list
    rsx!{
        ProtectedRoute {
        style { {DASHBOARD_CSS} }
        AppNavbar {
            button {
                class: "app-navbar-center-button",
                onclick: move |_| {
                    navigator.push(Route::CreateBlockPage {});
                },
                img { src: add_icon, class: "app-navbar-center-button-icon" }
                "New block"
            }
        }
        div{
            class:"blocks-page",
            // Close drop down menu when clicked outside 
            onclick: move |_| {
                if open_menu_id().is_some() {
                    open_menu_id.set(None);
                }
            },
            div{
                class:"blocks-list-container",
                ul{
                    class:"blocks-list",
                    for block in filtered_blocks.iter() {
                        {
                            let block_id = block.block_id.clone();
                            let block_id_for_delete = block.block_id.clone();
                            let block_name = block.block_name.clone();
                            let block_type = block.block_type.clone();
                            let image_count = block.image_count.clone();
                            let approved_image_count = block.approved_image_count.clone();
                            let annotation_count = block.annotation_count.clone();

                            let is_deleting = deleting_block_id() == Some(block.block_id.clone());
                            rsx!{
                                li {
                                    key:"{block_id}",
                                    class: if is_deleting { "block-card deleting" } else { "block-card" },
                                    onclick:move|_|{

                                        // Don't navigate if menu is open
                                        if open_menu_id().is_some() {
                                            open_menu_id.set(None);
                                            return;
                                        }
                                        let id = block_id.clone();
                                        state_set_current_block(&id.clone());
                                        navigator.push(Route::TasksListPage { block_id: id, block_name: block_name.clone(), block_type: block_type.as_str().to_string() });
                                    },
                                    
                                    // Row 1: Block name + three dots
                                    div {
                                        class: "block-row-1",
                                        div {
                                            class: "block-name-section",
                                            span { class: "block-name", "{block_name}" }
                                        }
                                        // Block drop down menu
                                        div {
                                            class: "block-actions",
                                            {
                                                let bid = block_id.clone();
                                                let bid_toggle = block_id.clone();
                                                let bid_delete = block.block_id.clone();
                                                rsx!{
                                                    BlockMenu {
                                                    block_id: bid.clone(),
                                                    block_type: block_type.as_str().to_string(),
                                                    is_open: open_menu_id() == Some(block.block_id.clone()),
                                                    on_toggle: move |_| {
                                                        let b = bid_toggle.clone();
                                                        if open_menu_id() == Some(b.clone()) {
                                                            open_menu_id.set(None);
                                                        } else {
                                                            open_menu_id.set(Some(b));
                                                        }
                                                    },
                                                    on_edit: {
                                                        let bid_edit = block.block_id.clone();
                                                        let bname_edit = block.block_name.clone();
                                                        move |_| {
                                                            editing_block.set(Some((bid_edit.clone(), bname_edit.clone())));
                                                            open_menu_id.set(None);
                                                        }
                                                    },
                                                    on_archive: move |_| {
                                                        // TODO: archive block
                                                        open_menu_id.set(None);
                                                    },
                                                    on_import: {
                                                        let bid_import = block.block_id.clone();
                                                        let bname_import = block.block_name.clone();
                                                        let btype_import = block.block_type.clone();
                                                        move |_| {
                                                            open_menu_id.set(None);
                                                            navigator.push(Route::ImportBlockPage {
                                                                block_id: bid_import.clone(),
                                                                block_name: bname_import.clone(),
                                                                block_type: btype_import.as_str().to_string(),
                                                            });
                                                        }
                                                    },
                                                    on_delete: move |_| {
                                                        let id = bid_delete.clone();
                                                        deleting_block_id.set(Some(id.clone()));
                                                        spawn(async move {
                                                            state_delete_block(&id).await;
                                                            deleting_block_id.set(None);
                                                        });
                                                        open_menu_id.set(None);
                                                    },
                                                }
                                            }
                                                
                                            }
                                        }
                                    }
                                    
                                    // Row 2: Image count
                                    div {
                                        class: "block-row-2",
                                        span { class: "block-image-count", "{approved_image_count}/{image_count}" }
                                    }
                                    
                                    // Row 3: Divider
                                    div { class: "block-divider" }
                                    
                                    // Row 5: Labels with counts (vertical, numbered) - admin only
                                    if user_role == UserRole::Admin {
                                    {
                                        let total_annotations: u32 = block.labels.iter().map(|l| l.label_count).sum();
                                        rsx! {
                                            div {
                                                class: "block-row-5",
                                                for (idx, label) in block.labels.iter().enumerate(){
                                                    {{
                                                        let max_count = block.labels.iter().map(|l| l.label_count).max().unwrap_or(1).max(1);
                                                        let bar_pct = (label.label_count as f64 / max_count as f64 * 100.0) as u32;
                                                        rsx! {
                                                            div {
                                                                class:"block-label",
                                                                style:"--label-color: {label.label_color}; --bar-width: {bar_pct}%;",
                                                                span { class: "block-label-num", "#{idx + 1}.{label.label_name}" }
                                                                div { class: "block-label-bar-bg",
                                                                    div { class: "block-label-bar-fill" }
                                                                }
                                                                span { class: "block-label-count", "{label.label_count}" }
                                                            }
                                                        }
                                                    }}
                                                }
                                                // Total row
                                                div {
                                                    class: "block-label block-label-total",
                                                    span { class: "block-label-num", "#total" }
                                                    span { class: "block-label-line-total" }
                                                    span { class: "block-label-count", "{total_annotations}" }
                                                }
                                            }
                                        }
                                    }
                                    }
                                    
                                    // Row 6: Dates (bottom right)
                                    div {
                                        class: "block-row-6",
                                        div { class: "block-dates",
                                            div { class: "block-date", "Created: {format_date(&block.block_created_at)}" }
                                            div { class: "block-date", "Updated: {format_relative_time(&block.block_updated_at)}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
            }
        }
        // Edit modal
        if let Some((block_id, current_name)) = editing_block() {
            EditBlockModal {
                block_id: block_id.clone(),
                current_name: current_name.clone(),
                on_save: move |new_name: String| {
                    let bid = block_id.clone();
                    spawn(async move {
                        state_rename_block(&bid, new_name).await;
                    });
                    editing_block.set(None);
                },
                on_cancel: move |_| {
                    editing_block.set(None);
                }
            }
        }
        }
    }
}
