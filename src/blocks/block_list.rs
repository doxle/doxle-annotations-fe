use crate::Route;
use dioxus::prelude::*;
use crate::core::{THEME, Theme, AppNavbar, BottomBar, BLOCK_TYPE_FILTER, ProtectedRoute, is_mobile};
use crate::blocks::state::{BLOCKS, BLOCKS_LOADING, BLOCKS_ERROR, state_load_blocks,state_delete_block, state_set_current_block, state_rename_block};
use crate::blocks::api::BlockType;
use crate::users::state::USER;
use crate::core::status_dialog::LiveTimer;
use super::block_menu::BlockMenu;
use super::edit_block_modal::EditBlockModal;

const BLOCK_LIST_CSS: &str = include_str!("block_list.css");
const BLOCKS_ICON_LIGHT: Asset = asset!("/assets/icons/blocks-light.svg");
const BLOCKS_ICON_DARK: Asset = asset!("/assets/icons/blocks-dark.svg");
const TRASH_ICON_LIGHT: Asset = asset!("/assets/icons/delete-light.svg");
const TRASH_ICON_DARK: Asset = asset!("/assets/icons/delete-dark.svg");
const ANNOTATION_BLOCK_ICON: Asset = asset!("/assets/icons/annotation-block-badge.svg");
const ADD_ICON_LIGHT: Asset = asset!("/assets/icons/add-light.svg");
const ADD_ICON_DARK: Asset = asset!("/assets/icons/add-dark.svg");
const CARD_ICON_LIGHT: Asset = asset!("/assets/icons/card-light.svg");
const CARD_ICON_DARK: Asset = asset!("/assets/icons/card-dark.svg");
const ANNOTATION_BLOCK_LIGHT: Asset = asset!("/assets/icons/annotation-block-light.svg");
const ANNOTATION_BLOCK_DARK: Asset = asset!("/assets/icons/annotation-block-dark.svg");
const FILE_BLOCK_LIGHT: Asset = asset!("/assets/icons/file-block-light.svg");
const FILE_BLOCK_DARK: Asset = asset!("/assets/icons/file-block-dark.svg");
const BUILD_BLOCK_LIGHT: Asset = asset!("/assets/icons/build-block-light.svg");
const BUILD_BLOCK_DARK: Asset = asset!("/assets/icons/build-block-dark.svg");
const LIST_ICON_LIGHT: Asset = asset!("/assets/icons/list-light.svg");
const LIST_ICON_DARK: Asset = asset!("/assets/icons/list-dark.svg");
const GRID_ICON_LIGHT: Asset = asset!("/assets/icons/grid-light.svg");
const GRID_ICON_DARK: Asset = asset!("/assets/icons/grid-dark.svg");
const SEARCH_ICON_LIGHT: Asset = asset!("/assets/icons/search-light.svg");
const SEARCH_ICON_DARK: Asset = asset!("/assets/icons/search-dark.svg");
const CLOSE_ICON_LIGHT: Asset = asset!("/assets/icons/close-light.svg");
const CLOSE_ICON_DARK: Asset = asset!("/assets/icons/close-dark.svg");


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

fn encode_route_segment(value: &str) -> String {
    crate::core::route_utils::encode_route_segment(value)
}

/// Trigger a browser file download using Dioxus document::eval
fn trigger_json_download(content: &str, filename: &str) {
    let encoded = js_sys::encode_uri_component(content).as_string().unwrap_or_default();
    let js = format!(
        r#"var b=new Blob([decodeURIComponent("{}")],{{type:"application/json"}});var a=document.createElement("a");a.href=URL.createObjectURL(b);a.download="{}";document.body.appendChild(a);a.click();document.body.removeChild(a);URL.revokeObjectURL(a.href);"#,
        encoded, filename
    );
    document::eval(&js);
}

#[component]
pub fn BlocksPage(project_id: String)->Element{
    let project_id = use_signal(move || project_id.clone());
	// Get current theme
	let is_dark = THEME() == Theme::Dark;
	let blocks_icon = if is_dark { BLOCKS_ICON_DARK } else { BLOCKS_ICON_LIGHT };
	let trash_icon = if is_dark { TRASH_ICON_DARK } else { TRASH_ICON_LIGHT };
	let add_icon = if is_dark { ADD_ICON_DARK } else { ADD_ICON_LIGHT };
	let card_icon = if is_dark { CARD_ICON_DARK } else { CARD_ICON_LIGHT };
	let list_icon = if is_dark { LIST_ICON_DARK } else { LIST_ICON_LIGHT };
	let grid_icon = if is_dark { GRID_ICON_DARK } else { GRID_ICON_LIGHT };
	let search_icon = if is_dark { SEARCH_ICON_DARK } else { SEARCH_ICON_LIGHT };
	let close_icon = if is_dark { CLOSE_ICON_DARK } else { CLOSE_ICON_LIGHT };
    let navigator = use_navigator();
    let mut open_menu_id: Signal<Option<String>> = use_signal(|| None);
    let mut view_mode: Signal<String> = use_signal(|| "grid".to_string());
    let mut search_active: Signal<bool> = use_signal(|| false);
    let mut search_query = use_signal(String::new);
    let mut editing_block: Signal<Option<(String, String)>> = use_signal(|| None); // (block_id, current_name)
    let mut deleting_block_id: Signal<Option<String>> = use_signal(|| None);
    let mut exporting_block_id: Signal<Option<String>> = use_signal(|| None);
    let mut export_status: Signal<String> = use_signal(|| String::new());
    let mut reconciling_block_id: Signal<Option<String>> = use_signal(|| None);
    let mut reconcile_status: Signal<String> = use_signal(|| String::new());

    info!("Loading dashboard page: ");

    // Track whether the resource has started loading
    let mut did_start_load = use_signal(|| false);

    // Load blocks on mount
    use_resource(move || async move {
        info!("Loading dashboard page - project load");
        did_start_load.set(true);
        state_load_blocks(&project_id()).await;
    });



    // Show loading or existing blocks
    info!("BLOCKS-LIST Page :-");

    let is_admin = USER.read().as_ref().map(|u| u.is_admin()).unwrap_or(false);

    // Show loading: either explicitly loading, or initial state before use_resource fires
    if BLOCKS_LOADING() || !did_start_load() {
        return rsx! {
            ProtectedRoute {
                style { {BLOCK_LIST_CSS} }
                AppNavbar {}
                div { class: "blocks-page blocks-page-loading",
                    div { class: "blocks-loading", "Loading blocks..." }
                }
            }
        };
    }

    if let Some(err) = BLOCKS_ERROR.read().as_ref() {
        return rsx! {
            ProtectedRoute {
                style { {BLOCK_LIST_CSS} }
                AppNavbar {}
                div { class: "blocks-page",
                    div { class: "blocks-empty", "Failed to load blocks: {err}" }
                }
            }
        };
    }
    let mut filtered_blocks: Vec<_> = BLOCKS.read().iter().cloned().collect();
    filtered_blocks.sort_by(|a, b| b.block_created_at.cmp(&a.block_created_at));

    // Filter by block type (from bottom bar — mobile only)
    let type_filter = BLOCK_TYPE_FILTER();
    let filtered_blocks: Vec<_> = if !is_mobile() || type_filter == "all" {
        filtered_blocks
    } else {
        filtered_blocks.into_iter()
            .filter(|b| b.block_type.as_str() == type_filter)
            .collect()
    };
    
    // Filter by search query
    let filtered_blocks: Vec<_> = if search_query().is_empty() {
        filtered_blocks
    } else {
        let query = search_query().to_lowercase();
        filtered_blocks.iter()
            .filter(|b| b.block_name.to_lowercase().contains(&query))
            .cloned()
            .collect()
    };

    // Check if there are no blocks before filtering (for true empty state)
    let has_blocks = !BLOCKS.read().is_empty();
    
    // If no blocks at all, show empty state
    if !has_blocks {
        return rsx! {
            ProtectedRoute {
                style { {BLOCK_LIST_CSS} }
                AppNavbar {}
                div { class: "blocks-empty-page",
                    if is_admin {
                        button {
                            class: "create-block-btn",
                            onclick: {
                                let project_id = project_id.clone();
                                move |_| {
                                    navigator.push(Route::CreateBlockPage { project_id: project_id().clone() });
                                }
                            },
                            "Create block"
                        }
                    } else {
                        div { class: "blocks-empty", "No blocks available" }
                    }
                }
            }
        };
    }
    
    // If blocks exist, show list
    rsx!{
        ProtectedRoute {
        style { {BLOCK_LIST_CSS} }
        AppNavbar {}
        div{
            class:"blocks-page",
            // Close drop down menu when clicked outside 
            onclick: move |_| {
                if open_menu_id().is_some() {
                    open_menu_id.set(None);
                }
            },
            div{
                class: if view_mode() == "list" { "blocks-list-container list-mode" } else { "blocks-list-container" },
                div {
                    class: if search_active() { "new-page-btn search-mode" } else { "new-page-btn" },
                    if search_active() {
                        button {
                            class: "search-icon-btn",
                            img { src: search_icon, class: "action-icon search-icon" }
                        }
                        input {
                            class: "search-input",
                            r#type: "text",
                            placeholder: "Search blocks...",
                            value: "{search_query}",
                            oninput: move |e| search_query.set(e.value()),
                            onkeydown: move |e| {
                                if e.key() == Key::Escape {
                                    search_active.set(false);
                                    search_query.set(String::new());
                                }
                            },
                            onmounted: move |e| {
                                let _ = e.set_focus(true);
                            },
                        }
                        button {
                            class: "close-search-btn",
                            onclick: move |_| {
                                search_active.set(false);
                                search_query.set(String::new());
                            },
                            img { src: close_icon, class: "action-icon" }
                        }
                    } else {
                        button {
                            class: "new-project-action",
                            onclick: move |_| {
                                let pid = project_id().clone();
                                navigator.push(Route::CreateBlockPage { project_id: pid });
                            },
                            img { src: add_icon, class: "action-icon" }
                            "New Block"
                        }
                        div { class: "action-divider" }
                        button {
                            class: if view_mode() == "list" { "action-btn active" } else { "action-btn" },
                            onclick: move |_| view_mode.set("list".to_string()),
                            img { src: list_icon, class: "action-icon" }
                        }
                        div { class: "action-divider" }
                        button {
                            class: if view_mode() == "grid" { "action-btn active" } else { "action-btn" },
                            onclick: move |_| view_mode.set("grid".to_string()),
                            img { src: grid_icon, class: "action-icon" }
                        }
                        div { class: "action-divider" }
                        button {
                            class: "action-btn",
                            onclick: move |_| search_active.set(true),
                            img { src: search_icon, class: "action-icon search-icon" }
                        }
                    }
                }
                if filtered_blocks.is_empty() && search_active() {
                    div { class: "no-results", "No blocks matched" }
                } else {
                    ul{
                        class: if view_mode() == "list" { "blocks-list blocks-list-view" } else { "blocks-list" },
                        for block in filtered_blocks.iter() {
                        {
                            let project_id_for_card = project_id().clone();
                            let project_id_for_import = project_id().clone();
                            let project_id_for_export = project_id().clone();
                            let project_id_for_reconcile = project_id().clone();
                            let block_id = block.block_id.clone();
                            let block_id_for_delete = block.block_id.clone();
                            let block_name = block.block_name.clone();
                            let block_name_for_route = encode_route_segment(&block_name);
                            let block_type = block.block_type.clone();
                            let block_type_for_onclick = block.block_type.clone();
                            let image_count = block.image_count.clone();
                            let approved_image_count = block.approved_image_count.clone();
                            let annotation_count = block.annotation_count.clone();

                            let is_deleting = deleting_block_id() == Some(block.block_id.clone());
                            let is_exporting = exporting_block_id() == Some(block.block_id.clone());
                            let is_reconciling = reconciling_block_id() == Some(block.block_id.clone());
                            let card_class = if is_deleting { "block-card deleting" } else if is_exporting || is_reconciling { "block-card exporting" } else { "block-card" };
                            
                            // Select icon based on block type and theme
                            let block_type_icon = match block.block_type {
                                BlockType::Annotation => if is_dark { ANNOTATION_BLOCK_DARK } else { ANNOTATION_BLOCK_LIGHT },
                                BlockType::File => if is_dark { FILE_BLOCK_DARK } else { FILE_BLOCK_LIGHT },
                                BlockType::Building => if is_dark { BUILD_BLOCK_DARK } else { BUILD_BLOCK_LIGHT },
                            };
                            let block_id_for_ctx = block.block_id.clone();
                            rsx!{
                                li {
                                    key:"{block_id}",
                                    class: card_class,
                                    oncontextmenu: move |e| {
                                        e.prevent_default();
                                        e.stop_propagation();
                                        open_menu_id.set(Some(block_id_for_ctx.clone()));
                                    },
                                    onclick:move|_|{

                                        // Don't navigate if menu is open
                                        if open_menu_id().is_some() {
                                            open_menu_id.set(None);
                                            return;
                                        }
                                        let id = block_id.clone();
                                        let encoded_name = js_sys::encode_uri_component(&block_name).as_string().unwrap_or(block_name.clone());
                                        if block_type_for_onclick == BlockType::File {
                                            navigator.push(Route::FileBlockPage {
                                                project_id: project_id_for_card.clone(),
                                                block_id: id,
                                                block_name: block_name_for_route.clone(),
                                                block_type: block_type_for_onclick.as_str().to_string(),
                                            });
                                        } else if block_type_for_onclick == BlockType::Building {
                                            navigator.push(Route::BuildingBlockPage {
                                                project_id: project_id_for_card.clone(),
                                                block_id: id,
                                                block_name: block_name_for_route.clone(),
                                                block_type: block_type_for_onclick.as_str().to_string(),
                                            });
                                        } else {
                                            navigator.push(Route::TasksListPage {
                                                project_id: project_id_for_card.clone(),
                                                block_id: id,
                                                block_name: block_name_for_route.clone(),
                                                block_type: block_type_for_onclick.as_str().to_string(),
                                            });
                                        }
                                    },
                                    
                                    // Export overlay
                                    if is_exporting {
                                        div {
                                            class: "block-export-overlay",
                                            LiveTimer {}
                                            span { class: "block-export-label", "{export_status}" }
                                            div {
                                                class: "block-export-stats",
                                                span { "{block.labels.len()} labels" }
                                                span { "{image_count} images" }
                                            }
                                        }
                                    }
                                    if is_reconciling {
                                        div {
                                            class: "block-export-overlay",
                                            LiveTimer {}
                                            span { class: "block-export-label", "{reconcile_status}" }
                                            div {
                                                class: "block-export-stats",
                                                span { "{image_count} images" }
                                                span { "{annotation_count} annotations" }
                                            }
                                        }
                                    }
                                    
                                    // Block type icon at top
                                    div {
                                        class: "block-type-icon-container",
                                        img {
                                            src: block_type_icon,
                                            class: "block-type-icon-top",
                                            alt: "{block_type.label()}"
                                        }
                                    }
                                    
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
                                                        let bname_import = encode_route_segment(&block.block_name);
                                                        let btype_import = block.block_type.clone();
                                                        move |_| {
                                                            open_menu_id.set(None);
                                                            let encoded_name = js_sys::encode_uri_component(&bname_import).as_string().unwrap_or(bname_import.clone());
                                                            navigator.push(Route::ImportBlockPage {
                                                                project_id: project_id_for_import.clone(),
                                                                block_id: bid_import.clone(),
                                                                block_name: encoded_name,
                                                                block_type: btype_import.as_str().to_string(),
                                                            });
                                                        }
                                                    },
                                                    on_export: {
                                                        let bid_export = block.block_id.clone();
                                                        let bname_export = block.block_name.clone();
                                                        let pid_export = project_id_for_export.clone();
                                                        move |_| {
                                                            open_menu_id.set(None);
                                                            let bid = bid_export.clone();
                                                            let bname = bname_export.clone();
                                                            let pid = pid_export.clone();
                                                            exporting_block_id.set(Some(bid.clone()));
                                                            export_status.set("Starting backup...".into());
                                                            spawn(async move {
                                                                let start_payload = serde_json::json!({
                                                                    "mode": "full",
                                                                    "include_image_copy": true
                                                                });
                                                                let start_resp = crate::core::client::post::<serde_json::Value, serde_json::Value>(
                                                                    &format!("/projects/{}/blocks/{}/backups", pid, bid),
                                                                    &start_payload,
                                                                ).await;

                                                                let backup_id = match start_resp {
                                                                    Ok(resp) => {
                                                                        match resp.get("backup_id").and_then(|v| v.as_str()) {
                                                                            Some(id) if !id.is_empty() => id.to_string(),
                                                                            _ => {
                                                                                export_status.set("Error: failed to start backup".into());
                                                                                tracing::error!("Backup start missing backup_id: {:?}", resp);
                                                                                gloo_timers::future::TimeoutFuture::new(2000).await;
                                                                                exporting_block_id.set(None);
                                                                                export_status.set(String::new());
                                                                                return;
                                                                            }
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        export_status.set(format!("Error: {}", e));
                                                                        tracing::error!("Backup start failed: {}", e);
                                                                        gloo_timers::future::TimeoutFuture::new(2000).await;
                                                                        exporting_block_id.set(None);
                                                                        export_status.set(String::new());
                                                                        return;
                                                                    }
                                                                };

                                                                let mut completed = false;
                                                                for _ in 0..300 {
                                                                    gloo_timers::future::TimeoutFuture::new(1500).await;

                                                                    let status_endpoint = format!(
                                                                        "/projects/{}/blocks/{}/backups/{}",
                                                                        pid, bid, backup_id
                                                                    );
                                                                    match crate::core::client::get::<serde_json::Value>(&status_endpoint).await {
                                                                        Ok(status_resp) => {
                                                                            let status = status_resp.get("status").and_then(|v| v.as_str()).unwrap_or("queued");
                                                                            let phase = status_resp.get("phase").and_then(|v| v.as_str()).unwrap_or("pending");
                                                                            let images_total = status_resp.get("images_total").and_then(|v| v.as_u64()).unwrap_or(0);
                                                                            let images_copied = status_resp.get("images_copied").and_then(|v| v.as_u64()).unwrap_or(0);
                                                                            let anns_total = status_resp.get("annotations_total").and_then(|v| v.as_u64()).unwrap_or(0);
                                                                            let anns_exported = status_resp.get("annotations_exported").and_then(|v| v.as_u64()).unwrap_or(0);

                                                                            if status == "completed" {
                                                                                if let Some(url) = status_resp.get("download_url").and_then(|v| v.as_str()) {
                                                                                    export_status.set("Downloading backup...".into());
                                                                                    let safe_name = bname.replace('\"', "").replace('/', "_");
                                                                                    let js = format!(
                                                                                        r#"var a=document.createElement("a");a.href="{}";a.download="{}.zip";document.body.appendChild(a);a.click();document.body.removeChild(a);"#,
                                                                                        url,
                                                                                        safe_name
                                                                                    );
                                                                                    document::eval(&js);
                                                                                    export_status.set("Done!".into());
                                                                                } else {
                                                                                    export_status.set("Error: backup finished without download URL".into());
                                                                                    tracing::error!("Completed backup missing download_url: {:?}", status_resp);
                                                                                }
                                                                                completed = true;
                                                                                break;
                                                                            }

                                                                            if status == "failed" {
                                                                                let msg = status_resp
                                                                                    .get("error_message")
                                                                                    .and_then(|v| v.as_str())
                                                                                    .unwrap_or("Backup failed");
                                                                                export_status.set(format!("Error: {}", msg));
                                                                                tracing::error!("Backup job failed: {:?}", status_resp);
                                                                                completed = true;
                                                                                break;
                                                                            }

                                                                            let phase_text = match phase {
                                                                                "copy_images" => "Copying images",
                                                                                "finalize_metadata" => "Finalizing backup",
                                                                                "pending" => "Queued",
                                                                                _ => "Processing backup",
                                                                            };
                                                                            export_status.set(format!(
                                                                                "{}... img {}/{} • ann {}/{}",
                                                                                phase_text,
                                                                                images_copied,
                                                                                images_total,
                                                                                anns_exported,
                                                                                anns_total
                                                                            ));
                                                                        }
                                                                        Err(e) => {
                                                                            export_status.set(format!("Error: {}", e));
                                                                            tracing::error!("Backup status poll failed: {}", e);
                                                                            completed = true;
                                                                            break;
                                                                        }
                                                                    }
                                                                }

                                                                if !completed {
                                                                    export_status.set("Error: backup timed out".into());
                                                                    tracing::error!("Backup polling timed out for block {}", bid);
                                                                }
                                                                gloo_timers::future::TimeoutFuture::new(2500).await;
                                                                exporting_block_id.set(None);
                                                                export_status.set(String::new());
                                                            });
                                                        }
                                                    },
                                                    on_reconcile: {
                                                        let bid_reconcile = block.block_id.clone();
                                                        let pid_reconcile = project_id_for_reconcile.clone();
                                                        move |_| {
                                                            open_menu_id.set(None);
                                                            let bid = bid_reconcile.clone();
                                                            let pid = pid_reconcile.clone();
                                                            reconciling_block_id.set(Some(bid.clone()));
                                                            reconcile_status.set("Starting reconcile...".into());
                                                            spawn(async move {
                                                                let start_payload = serde_json::json!({});
                                                                let start_resp = crate::core::client::post::<serde_json::Value, serde_json::Value>(
                                                                    &format!("/projects/{}/blocks/{}/reconcile-counts", pid, bid),
                                                                    &start_payload,
                                                                ).await;

                                                                let reconcile_job_id = match start_resp {
                                                                    Ok(resp) => {
                                                                        match resp.get("reconcile_job_id").and_then(|v| v.as_str()) {
                                                                            Some(id) if !id.is_empty() => id.to_string(),
                                                                            _ => {
                                                                                reconcile_status.set("Error: failed to start reconcile".into());
                                                                                tracing::error!("Reconcile start missing reconcile_job_id: {:?}", resp);
                                                                                gloo_timers::future::TimeoutFuture::new(2000).await;
                                                                                reconciling_block_id.set(None);
                                                                                reconcile_status.set(String::new());
                                                                                return;
                                                                            }
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        reconcile_status.set(format!("Error: {}", e));
                                                                        tracing::error!("Reconcile start failed: {}", e);
                                                                        gloo_timers::future::TimeoutFuture::new(2000).await;
                                                                        reconciling_block_id.set(None);
                                                                        reconcile_status.set(String::new());
                                                                        return;
                                                                    }
                                                                };

                                                                let mut completed = false;
                                                                let mut succeeded = false;
                                                                for _ in 0..720 {
                                                                    gloo_timers::future::TimeoutFuture::new(1500).await;

                                                                    let status_endpoint = format!(
                                                                        "/projects/{}/blocks/{}/reconcile-counts/{}",
                                                                        pid, bid, reconcile_job_id
                                                                    );

                                                                    match crate::core::client::get::<serde_json::Value>(&status_endpoint).await {
                                                                        Ok(status_resp) => {
                                                                            let status = status_resp.get("status").and_then(|v| v.as_str()).unwrap_or("queued");
                                                                            let phase = status_resp.get("phase").and_then(|v| v.as_str()).unwrap_or("pending");
                                                                            let images_total = status_resp.get("images_total").and_then(|v| v.as_u64()).unwrap_or(0);
                                                                            let images_processed = status_resp.get("images_processed").and_then(|v| v.as_u64()).unwrap_or(0);
                                                                            let anns_total = status_resp.get("annotations_total").and_then(|v| v.as_u64()).unwrap_or(0);
                                                                            let anns_processed = status_resp.get("annotations_processed").and_then(|v| v.as_u64()).unwrap_or(0);
                                                                            let block_total = status_resp.get("block_annotation_count").and_then(|v| v.as_u64()).unwrap_or(0);

                                                                            if status == "completed" {
                                                                                reconcile_status.set(format!("Done! total {}", block_total));
                                                                                completed = true;
                                                                                succeeded = true;
                                                                                break;
                                                                            }

                                                                            if status == "failed" {
                                                                                let msg = status_resp
                                                                                    .get("error_message")
                                                                                    .and_then(|v| v.as_str())
                                                                                    .unwrap_or("Reconcile failed");
                                                                                reconcile_status.set(format!("Error: {}", msg));
                                                                                tracing::error!("Reconcile job failed: {:?}", status_resp);
                                                                                completed = true;
                                                                                break;
                                                                            }

                                                                            let phase_text = match phase {
                                                                                "reconciling" => "Reconciling",
                                                                                "pending" => "Queued",
                                                                                _ => "Processing reconcile",
                                                                            };
                                                                            reconcile_status.set(format!(
                                                                                "{}... img {}/{} • ann {}/{}",
                                                                                phase_text,
                                                                                images_processed,
                                                                                images_total,
                                                                                anns_processed,
                                                                                anns_total
                                                                            ));
                                                                        }
                                                                        Err(e) => {
                                                                            reconcile_status.set(format!("Error: {}", e));
                                                                            tracing::error!("Reconcile status poll failed: {}", e);
                                                                            completed = true;
                                                                            break;
                                                                        }
                                                                    }
                                                                }

                                                                if !completed {
                                                                    reconcile_status.set("Error: reconcile timed out".into());
                                                                    tracing::error!("Reconcile polling timed out for block {}", bid);
                                                                }

                                                                if succeeded {
                                                                    state_load_blocks(&pid).await;
                                                                }

                                                                gloo_timers::future::TimeoutFuture::new(2500).await;
                                                                reconciling_block_id.set(None);
                                                                reconcile_status.set(String::new());
                                                            });
                                                        }
                                                    },
                                                    on_delete: move |_| {
                                                        let id = bid_delete.clone();
                                                        deleting_block_id.set(Some(id.clone()));
                                                        spawn(async move {
                                                            state_delete_block(&project_id(), &id).await;
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
                                    if !(view_mode() == "list" && block_type != BlockType::Annotation) {
                                        div {
                                            class: "block-row-2",
                                            span { class: "block-image-count", "{approved_image_count}/{image_count}" }
                                        }
                                    }
                                    
                                    
                                    // Row 6: Dates (bottom right)
                                    div {
                                        class: "block-row-6",
                                        div { class: "block-dates",
                                            div { class: "block-date", "{format_date(&block.block_created_at)}" }
                                            div { class: "block-date", "{format_relative_time(&block.block_updated_at)}" }
                                        }
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
                        state_rename_block(&project_id(), &bid, new_name).await;
                    });
                    editing_block.set(None);
                },
                on_cancel: move |_| {
                    editing_block.set(None);
                }
            }
        }
        BottomBar { project_id: project_id().clone() }
        }
    }
}
