use crate::blocks::annotations::api::api_list_threads;
use crate::blocks::note::note_context_menu::FileContextMenu;
use crate::media::api::{api_delete_note_attachment, list_note_block_attachments, upload_attachment_for_note_block};
use crate::media::FileAttachment;
use crate::core::client::{to_cloudfront_media_url, to_cloudfront_url};
use crate::core::{AppNavbar, BottomBar, LoadingScreen, Theme, THEME};
use crate::Route;
use dioxus::prelude::*;
use futures::stream::{self, StreamExt};
use std::collections::HashMap;

const NOTE_BLOCK_PAGE_CSS: &str = include_str!("note_block_page.css");
const BLOCK_LIST_CSS: &str = include_str!("../../blocks/block_list.css");
const DOG_LIGHT_ICON: Asset = asset!("/assets/icons/dog-light.svg");
const DOG_DARK_ICON: Asset = asset!("/assets/icons/dog-dark.svg");
const COMMENT_BLUE_ICON: Asset = asset!("/assets/icons/comment-blue.svg");
const ADD_ICON_LIGHT: Asset = asset!("/assets/icons/add-light.svg");
const ADD_ICON_DARK: Asset = asset!("/assets/icons/add-dark.svg");
const SEARCH_ICON_LIGHT: Asset = asset!("/assets/icons/search-light.svg");
const SEARCH_ICON_DARK: Asset = asset!("/assets/icons/search-dark.svg");
const CLOSE_ICON_LIGHT: Asset = asset!("/assets/icons/close-light.svg");
const CLOSE_ICON_DARK: Asset = asset!("/assets/icons/close-dark.svg");

#[derive(Clone, PartialEq)]
struct UploadItem {
    name: String,
    status: String,
}


fn is_video(media: &FileAttachment) -> bool {
    media.media_type == "video"
}

fn is_image(media: &FileAttachment) -> bool {
    media.media_type == "image"
}

fn is_pdf(media: &FileAttachment) -> bool {
    media.attachment_name.to_ascii_lowercase().ends_with(".pdf")
        || media.url.to_ascii_lowercase().contains(".pdf")
}

fn pdf_thumbnail_src(url: &str) -> String {
    format!("{url}#page=1&toolbar=0&navpanes=0&scrollbar=0&view=FitH")
}
/// How long to ignore clicks after file cards become visible (prevents ghost taps on iOS).
const MOBILE_CARDS_GUARD_MS: u128 = 800;

async fn fetch_comment_counts(attachment_ids: Vec<String>) -> HashMap<String, usize> {
    let comment_count_futures = attachment_ids.into_iter().map(|attachment_id| async move {
        let count = match api_list_threads(&attachment_id).await {
            Ok(threads) => threads.iter().map(|thread| thread.comments.len()).sum::<usize>(),
            Err(_) => 0,
        };
        (attachment_id, count)
    });
    let mut stream = stream::iter(comment_count_futures).buffer_unordered(6);
    let mut counts = HashMap::new();
    while let Some((attachment_id, count)) = stream.next().await {
        counts.insert(attachment_id, count);
    }
    counts
}


#[component]
pub fn NoteBlockPage(
    project_id: String,
    block_id: String,
    block_name: String,
    block_type: String,
) -> Element {
    let block_name_display = crate::core::route_utils::decode_route_segment(&block_name);
    let nav = use_navigator();
    let mut media_items = use_signal(Vec::<FileAttachment>::new);
    let mut comment_counts = use_signal(HashMap::<String, usize>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut upload_items = use_signal(Vec::<UploadItem>::new);
    let mut upload_current = use_signal(|| 0usize);
    let mut upload_total = use_signal(|| 0usize);
    let mut is_uploading = use_signal(|| false);
    // Context menu state: Option<(image_id, file_name, media_type, x, y)>
    let mut context_menu: Signal<Option<(String, String, String, f64, f64)>> = use_signal(|| None);
    // Long-press state for mobile
    let mut long_press_active: Signal<bool> = use_signal(|| false);
    let mut long_press_id: Signal<Option<String>> = use_signal(|| None);
    // Search state
    let mut search_active = use_signal(|| false);
    let mut search_query = use_signal(String::new);
    // Single timestamp: when were file cards last made visible?
    // All mobile clicks are blocked for MOBILE_CARDS_GUARD_MS after this.
    let mut cards_visible_since = use_signal(web_time::Instant::now);
    let is_dark = THEME() == Theme::Dark;
    let dog_icon = if is_dark { DOG_DARK_ICON } else { DOG_LIGHT_ICON };
    let add_icon = if is_dark { ADD_ICON_DARK } else { ADD_ICON_LIGHT };
    let search_icon = if is_dark { SEARCH_ICON_DARK } else { SEARCH_ICON_LIGHT };
    let close_icon = if is_dark { CLOSE_ICON_DARK } else { CLOSE_ICON_LIGHT };

    let project_id_for_load = project_id.clone();
    let block_id_for_load = block_id.clone();
    use_effect(move || {
        loading.set(true);
        error.set(None);
        let project_id = project_id_for_load.clone();
        let block_id = block_id_for_load.clone();
        spawn(async move {
            match list_note_block_attachments(&project_id, &block_id).await {
                Ok(items) => {
                    let attachment_ids = items.iter().map(|item| item.attachment_id.clone()).collect::<Vec<_>>();
                    media_items.set(items);
                    comment_counts.set(fetch_comment_counts(attachment_ids).await);
                }
                Err(e) => error.set(Some(e)),
            }
            // Start the guard RIGHT BEFORE cards become visible
            cards_visible_since.set(web_time::Instant::now());
            loading.set(false);
        });
    });

    rsx! {
        style { {NOTE_BLOCK_PAGE_CSS} }
        style { {BLOCK_LIST_CSS} }
        AppNavbar {}
        div {
            class: "file-block-page",
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
                        placeholder: "Search files...",
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
                        r#type: "button",
                        class: "new-project-action",
                        disabled: is_uploading(),
                        onclick: move |_| {
                            if is_uploading() {
                                return;
                            }
                            if crate::core::is_mobile() && cards_visible_since().elapsed().as_millis() < MOBILE_CARDS_GUARD_MS {
                                return;
                            }
                            #[cfg(target_arch = "wasm32")]
                            {
                                use wasm_bindgen::JsCast;
                                if let Some(window) = web_sys::window() {
                                    if let Some(document) = window.document() {
                                        if let Some(element) = document.get_element_by_id("file-block-upload-input") {
                                            if let Ok(input) = element.dyn_into::<web_sys::HtmlInputElement>() {
                                                input.click();
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        if !is_uploading() {
                            img { src: add_icon, class: "action-icon" }
                        }
                        if is_uploading() { "Uploading..." } else { "New File" }
                    }
                    div { class: "action-divider" }
                    button {
                        class: "action-btn",
                        onclick: move |_| search_active.set(true),
                        img { src: search_icon, class: "action-icon search-icon" }
                    }
                }
            }
            input {
                r#type: "file",
                id: "file-block-upload-input",
                class: "file-block-upload-input",
                multiple: true,
                accept: "image/*,video/*,application/pdf,.pdf",
                disabled: is_uploading(),
                onchange: move |evt| {
                    #[cfg(target_arch = "wasm32")]
                    {
                        use wasm_bindgen::JsCast;
                        if let Some(web_evt) = evt.downcast::<web_sys::Event>() {
                            if let Some(target) = web_evt.target() {
                                if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
                                    if let Some(files) = input.files() {
                                        let mut files_to_upload: Vec<web_sys::File> = Vec::new();
                                        let mut rows: Vec<UploadItem> = Vec::new();
                                        for i in 0..files.length() {
                                            if let Some(file) = files.get(i) {
                                                rows.push(UploadItem {
                                                    name: file.name(),
                                                    status: "Queued".to_string(),
                                                });
                                                files_to_upload.push(file);
                                            }
                                        }
                                        input.set_value("");
                                        if crate::core::is_mobile() {
                                            cards_visible_since.set(web_time::Instant::now());
                                        }

                                        if files_to_upload.is_empty() {
                                            return;
                                        }

                                        upload_items.set(rows);
                                        upload_current.set(0);
                                        upload_total.set(files_to_upload.len());
                                        is_uploading.set(true);

                                        let project_id = project_id.clone();
                                        let block_id = block_id.clone();
                                        let project_id = project_id.clone();
                                        spawn(async move {
                                            // Small delay to let the UI recover (iOS camera return)
                                            gloo_timers::future::TimeoutFuture::new(300).await;

                                            let upload_futures = files_to_upload
                                                .into_iter()
                                                .enumerate()
                                                .map(|(idx, file)| {
                                                    let project_id = project_id.clone();
                                                    let block_id = block_id.clone();
                                                    async move {
                                                        let name = file.name();
                                                        let result = upload_attachment_for_note_block(&project_id, &block_id, file).await;
                                                        (idx, name, result)
                                                    }
                                                });
                                            let total_uploads = upload_total();
                                            let upload_started = web_time::Instant::now();
                                            let mut failed = 0usize;
                                            crate::core::progress::show_progress(
                                                &format!("Uploading files 0/{}", total_uploads),
                                                0,
                                                total_uploads,
                                                0,
                                            );

                                            // Use 1 concurrent upload on mobile to avoid memory issues
                                            let concurrency = if crate::core::is_mobile() { 1 } else { 3 };
                                            let mut stream = stream::iter(upload_futures).buffer_unordered(concurrency);
                                            let mut finished = 0usize;

                                            while let Some((idx, name, result)) = stream.next().await {
                                                finished += 1;
                                                upload_current.set(finished);
                                                let mut rows = upload_items.write();
                                                if let Some(row) = rows.get_mut(idx) {
                                                    row.status = match result {
                                                        Ok(_) => "Uploaded".to_string(),
                                                        Err(e) => {
                                                            failed += 1;
                                                            format!("Failed: {}", e)
                                                        },
                                                    };
                                                }
                                                crate::core::progress::show_progress(
                                                    &format!(
                                                        "Uploading files {}/{}{}",
                                                        finished,
                                                        total_uploads,
                                                        if failed > 0 {
                                                            format!(" · {} failed", failed)
                                                        } else {
                                                            String::new()
                                                        }
                                                    ),
                                                    finished,
                                                    total_uploads,
                                                    upload_started.elapsed().as_secs(),
                                                );
                                            }

                                            if failed > 0 {
                                                crate::core::progress::show_error_persistent(
                                                    &format!(
                                                        "Uploaded {} files, {} failed",
                                                        total_uploads.saturating_sub(failed),
                                                        failed
                                                    ),
                                                );
                                            } else {
                                                crate::core::progress::show_success(
                                                    &format!("Uploaded {} files", total_uploads),
                                                );
                                            }

                                            match list_note_block_attachments(&project_id, &block_id).await {
                                                Ok(items) => {
                                                    let attachment_ids = items.iter().map(|item| item.attachment_id.clone()).collect::<Vec<_>>();
                                                    media_items.set(items);
                                                    comment_counts.set(fetch_comment_counts(attachment_ids).await);
                                                }
                                                Err(e) => error.set(Some(e)),
                                            }
                                            is_uploading.set(false);
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div {
                class: "file-block-content",
                if loading() {
                    LoadingScreen { text: "Loading media".to_string() }
                } else if let Some(err) = error() {
                    div { class: "file-block-error", "{err}" }
                } else if media_items().is_empty() {
                    div {
                        class: "file-block-empty",
                        img {
                            class: "file-block-empty-icon",
                            src: dog_icon,
                            alt: "No files"
                        }
                        span { "No files uploaded yet" }
                    }
                } else {
                    {
                        let query = search_query().to_lowercase();
                        let filtered: Vec<FileAttachment> = if query.is_empty() {
                            media_items()
                        } else {
                            media_items().into_iter()
                                .filter(|m| m.attachment_name.to_lowercase().contains(&query))
                                .collect()
                        };
                        rsx! {
                    if filtered.is_empty() && search_active() {
                        div { class: "no-results", "No files matched" }
                    }
                    div {
                        class: "file-block-grid",
                        for media in filtered.iter() {
                            {
                                let attachment_id_for_route = media.attachment_id.clone();
                                let attachment_name_for_route = crate::core::route_utils::encode_route_segment(&media.attachment_name);
                                let src = if is_video(media) {
                                    to_cloudfront_media_url(&media.url)
                                } else {
                                    to_cloudfront_url(&media.url)
                                };
                                let project_id_for_route = project_id.clone();
                                let block_id_for_route = block_id.clone();
                                let block_name_for_route = crate::core::route_utils::encode_route_segment(&block_name);
                                let block_type_for_route = block_type.clone();
                                let nav_for_route = nav.clone();
                                let src_for_open = src.clone();
                                let is_file_media = !is_image(media) && !is_video(media);
                                // For long-press context menu
                                let ctx_attachment_id = media.attachment_id.clone();
                                let ctx_file_name = media.attachment_name.clone();
                                let ctx_media_type = media.media_type.clone();
                                // For long-press touch
                                let lp_attachment_id = media.attachment_id.clone();
                                let lp_file_name = media.attachment_name.clone();
                                let lp_media_type = media.media_type.clone();
                                rsx! {
                                    button {
                                        class: "file-card",
                                        onclick: move |_| {
                                            let elapsed = cards_visible_since().elapsed().as_millis();
                                            tracing::info!("FILE-CARD onclick: elapsed={}ms guard={}ms mobile={}", elapsed, MOBILE_CARDS_GUARD_MS, crate::core::is_mobile());
                                            if is_uploading() { return; }
                                            if crate::core::is_mobile() && elapsed < MOBILE_CARDS_GUARD_MS {
                                                tracing::info!("FILE-CARD onclick BLOCKED by guard");
                                                return;
                                            }
                                            if context_menu().is_some() { return; }
                                            if long_press_active() {
                                                long_press_active.set(false);
                                                return;
                                            }
                                            // PDFs & generic files: open directly (no NoteItemPage)
                                            if is_file_media {
                                                #[cfg(target_arch = "wasm32")]
                                                {
                                                    if let Some(window) = web_sys::window() {
                                                        if crate::core::is_mobile() {
                                                            let _ = window.location().set_href(&src_for_open);
                                                        } else {
                                                            let _ = window.open_with_url(&src_for_open);
                                                        }
                                                    }
                                                }
                                                return;
                                            }
                                            nav_for_route.push(Route::NoteItemPage {
                                                project_id: project_id_for_route.clone(),
                                                block_id: block_id_for_route.clone(),
                                                block_name: block_name_for_route.clone(),
                                                block_type: block_type_for_route.clone(),
                                                attachment_id: attachment_id_for_route.clone(),
                                                attachment_name: attachment_name_for_route.clone(),
                                            });
                                        },
                                        oncontextmenu: move |e| {
                                            e.prevent_default();
                                            let coords = e.client_coordinates();
                                            context_menu.set(Some((
                                                ctx_attachment_id.clone(),
                                                ctx_file_name.clone(),
                                                ctx_media_type.clone(),
                                                coords.x,
                                                coords.y,
                                            )));
                                        },
                                        ontouchstart: move |e| {
                                            let id = lp_attachment_id.clone();
                                            let fname = lp_file_name.clone();
                                            let mtype = lp_media_type.clone();
                                            long_press_active.set(false);
                                            long_press_id.set(Some(id.clone()));
                                            spawn(async move {
                                                gloo_timers::future::TimeoutFuture::new(500).await;
                                                if long_press_id() == Some(id.clone()) {
                                                    long_press_active.set(true);
                                    long_press_id.set(None);
                                                    // Center menu on screen for mobile
                                                    context_menu.set(Some((
                                                        id,
                                                        fname,
                                                        mtype,
                                                        40.0,
                                                        200.0,
                                                    )));
                                                }
                                            });
                                        },
                                        ontouchmove: move |_| {
                                            long_press_id.set(None);
                                        },
                                        ontouchend: move |_| {
                                            long_press_id.set(None);
                                        },
                                        div {
                                            class: "file-card-thumb",
                                            if is_video(media) {
                                                video {
                                                    class: "file-card-video",
                                                    src: "{src}#t=0.1",
                                                    preload: "metadata",
                                                    muted: true,
                                                    playsinline: true,
                                                }
                                            } else if is_image(media) {
                                                img {
                                                    class: "file-card-image",
                                                    src: "{src}",
                                                    alt: "{media.attachment_name}",
                                                }
                                            } else {
                                                div { class: "file-card-generic",
                                                    if is_pdf(media) { "PDF" } else { "FILE" }
                                                }
                                            }

                                            if !is_video(media) {
                                                for rect in media.markup_rects.iter() {
                                                    {
                                                        let rect_style = format!(
                                                            "left:{}%;top:{}%;width:{}%;height:{}%;",
                                                            rect.x * 100.0,
                                                            rect.y * 100.0,
                                                            rect.width * 100.0,
                                                            rect.height * 100.0
                                                        );
                                                        rsx! {
                                                            div {
                                                                class: "file-card-markup-rect",
                                                                style: "{rect_style}",
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            if let Some(comment_count) = comment_counts().get(&media.attachment_id).copied() {
                                                if comment_count > 0 {
                                                    div {
                                                        class: "file-card-comment-indicator",
                                                        img {
                                                            class: "file-card-comment-icon",
                                                            src: COMMENT_BLUE_ICON,
                                                            alt: "Comments"
                                                        }
                                                        sup {
                                                            class: "file-card-comment-count",
                                                            "{comment_count}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        div {
                                            class: "file-card-footer",
                                            span { class: "file-card-name", "{media.attachment_name}" }
                                            {
                                                let dot_attachment_id = media.attachment_id.clone();
                                                let dot_file_name = media.attachment_name.clone();
                                                let dot_media_type = media.media_type.clone();
                                                rsx! {
                                                    span {
                                                        class: "file-card-dots",
                                                        onclick: move |e| {
                                                            e.stop_propagation();
                                                            let coords = e.client_coordinates();
                                                            context_menu.set(Some((
                                                                dot_attachment_id.clone(),
                                                                dot_file_name.clone(),
                                                                dot_media_type.clone(),
                                                                coords.x,
                                                                coords.y,
                                                            )));
                                                        },
                                                        "•••"
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
                    }
                }
            }
        }
        // Context menu
        if let Some((ref ctx_attachment_id, ref ctx_file_name, ref ctx_media_type, ctx_x, ctx_y)) = context_menu() {
            {
                let attachment_id_for_delete = ctx_attachment_id.clone();
                let project_id_for_delete = project_id.clone();
                let block_id_for_delete = block_id.clone();
                rsx! {
                    FileContextMenu {
                        file_name: ctx_file_name.clone(),
                        media_type: ctx_media_type.clone(),
                        pos_x: ctx_x,
                        pos_y: ctx_y,
                        on_rename: move |_| {
                            // TODO: implement rename
                            context_menu.set(None);
                        },
                        on_delete: move |_| {
                            let pid = project_id_for_delete.clone();
                            let bid = block_id_for_delete.clone();
                            let iid = attachment_id_for_delete.clone();
                            context_menu.set(None);
                            spawn(async move {
                                match api_delete_note_attachment(&pid, &bid, &iid).await {
                                    Ok(_) => {
                                        media_items.write().retain(|m| m.attachment_id != iid);
                                        crate::core::progress::show_success("File deleted");
                                    }
                                    Err(e) => {
                                        crate::core::progress::show_error_persistent(&format!("Delete failed: {}", e));
                                    }
                                }
                            });
                        },
                        on_close: move |_| {
                            context_menu.set(None);
                        },
                    }
                }
            }
        }
        BottomBar { project_id: project_id.clone() }
    }
}
