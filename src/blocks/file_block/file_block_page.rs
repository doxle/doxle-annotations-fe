use crate::atoms::media::api::{api_update_image_markup, list_block_media, upload_image_for_block};
use crate::atoms::media::{Image, MarkupRect};
use crate::shell::client::to_cloudfront_url;
use crate::shell::AppNavbar;
use dioxus::prelude::*;
use futures::stream::{self, StreamExt};

const FILE_BLOCK_PAGE_CSS: &str = include_str!("file_block_page.css");

#[derive(Clone, PartialEq)]
struct UploadItem {
    name: String,
    status: String,
}

fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

fn rect_from_points(x1: f64, y1: f64, x2: f64, y2: f64) -> MarkupRect {
    let x = x1.min(x2);
    let y = y1.min(y2);
    let width = (x2 - x1).abs();
    let height = (y2 - y1).abs();
    MarkupRect { x, y, width, height }
}

fn is_video(media: &Image) -> bool {
    media.media_type == "video"
}

fn is_image(media: &Image) -> bool {
    media.media_type == "image"
}

#[cfg(target_arch = "wasm32")]
fn element_dimensions(element_id: &str) -> Option<(f64, f64)> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let element = document.get_element_by_id(element_id)?;
    let rect = element.get_bounding_client_rect();
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return None;
    }
    Some((rect.width(), rect.height()))
}

#[cfg(not(target_arch = "wasm32"))]
fn element_dimensions(_element_id: &str) -> Option<(f64, f64)> {
    None
}

#[component]
pub fn FileBlockPage(
    project_id: String,
    block_id: String,
    block_name: String,
    block_type: String,
) -> Element {
    let _ = &block_type;
    let mut media_items = use_signal(Vec::<Image>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut upload_items = use_signal(Vec::<UploadItem>::new);
    let mut upload_current = use_signal(|| 0usize);
    let mut upload_total = use_signal(|| 0usize);
    let mut is_uploading = use_signal(|| false);
    let mut selected_media = use_signal(|| None::<Image>);
    let mut markup_rects = use_signal(Vec::<MarkupRect>::new);
    let mut drag_start = use_signal(|| None::<(f64, f64)>);
    let mut draft_rect = use_signal(|| None::<MarkupRect>);

    let project_id_for_load = project_id.clone();
    let block_id_for_load = block_id.clone();
    use_effect(move || {
        loading.set(true);
        error.set(None);
        let project_id = project_id_for_load.clone();
        let block_id = block_id_for_load.clone();
        spawn(async move {
            match list_block_media(&project_id, &block_id).await {
                Ok(items) => media_items.set(items),
                Err(e) => error.set(Some(e)),
            }
            loading.set(false);
        });
    });

    rsx! {
        style { {FILE_BLOCK_PAGE_CSS} }
        AppNavbar {}
        div {
            class: "file-block-page",
            div {
                class: "file-block-head",
                h1 { class: "file-block-title", "{block_name}" }
                p { class: "file-block-subtitle", "Upload and review files in this block" }
            }
            div {
                class: "file-block-upload",
                input {
                    r#type: "file",
                    id: "file-block-upload-input",
                    class: "file-block-upload-input",
                    multiple: true,
                    accept: "image/*,video/*",
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
                                                let upload_futures = files_to_upload
                                                    .into_iter()
                                                    .enumerate()
                                                    .map(|(idx, file)| {
                                                        let project_id = project_id.clone();
                                                        let block_id = block_id.clone();
                                                        async move {
                                                            let name = file.name();
                                                            let result = upload_image_for_block(&project_id, &block_id, file).await;
                                                            (idx, name, result)
                                                        }
                                                    });

                                                let mut stream = stream::iter(upload_futures).buffer_unordered(3);
                                                let mut finished = 0usize;

                                                while let Some((idx, name, result)) = stream.next().await {
                                                    finished += 1;
                                                    upload_current.set(finished);
                                                    let mut rows = upload_items.write();
                                                    if let Some(row) = rows.get_mut(idx) {
                                                        row.status = match result {
                                                            Ok(_) => "Uploaded".to_string(),
                                                            Err(e) => format!("Failed: {}", e),
                                                        };
                                                    }
                                                    crate::shell::progress::show_info_for(
                                                        &format!("{} ({}/{})", name, finished, upload_total()),
                                                        1,
                                                    );
                                                }

                                                match list_block_media(&project_id, &block_id).await {
                                                    Ok(items) => media_items.set(items),
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
                label {
                    r#for: "file-block-upload-input",
                    class: "file-block-upload-btn",
                    if is_uploading() { "Uploading..." } else { "Upload files" }
                }
                if upload_total() > 0 {
                    div {
                        class: "file-block-upload-progress",
                        span { "{upload_current()}/{upload_total()}" }
                        for row in upload_items().iter() {
                            div {
                                class: "file-block-upload-row",
                                span { class: "file-block-upload-name", "{row.name}" }
                                span { class: "file-block-upload-status", "{row.status}" }
                            }
                        }
                    }
                }
            }

            if loading() {
                div { class: "file-block-loading", "Loading media..." }
            } else if let Some(err) = error() {
                div { class: "file-block-error", "{err}" }
            } else if media_items().is_empty() {
                div { class: "file-block-empty", "No files uploaded yet" }
            } else {
                div {
                    class: "file-block-grid",
                    for media in media_items().iter() {
                        {
                            let media_for_click = media.clone();
                            let src = to_cloudfront_url(&media.url);
                            rsx! {
                                button {
                                    class: "file-card",
                                    onclick: move |_| {
                                        selected_media.set(Some(media_for_click.clone()));
                                        markup_rects.set(media_for_click.markup_rects.clone());
                                        drag_start.set(None);
                                        draft_rect.set(None);
                                    },
                                    if is_video(media) {
                                        video {
                                            class: "file-card-video",
                                            src: "{src}",
                                            preload: "metadata",
                                            muted: true,
                                        }
                                    } else if is_image(media) {
                                        img {
                                            class: "file-card-image",
                                            src: "{src}",
                                            alt: "{media.image_name}",
                                        }
                                    } else {
                                        div { class: "file-card-generic", "FILE" }
                                    }
                                    div {
                                        class: "file-card-footer",
                                        span { class: "file-card-name", "{media.image_name}" }
                                        span { class: "file-card-type", "{media.media_type}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(current_media) = selected_media() {
            {
                let canvas_id = format!("file-markup-canvas-{}", current_media.image_id);
                let canvas_id_for_down = canvas_id.clone();
                let canvas_id_for_move = canvas_id.clone();
                let current_media_for_save = current_media.clone();
                let project_id_for_save = project_id.clone();
                let block_id_for_save = block_id.clone();
                let src = to_cloudfront_url(&current_media.url);
                rsx! {
                    div {
                        class: "file-modal-backdrop",
                        onclick: move |_| selected_media.set(None),
                        div {
                            class: "file-modal",
                            onclick: move |e| e.stop_propagation(),
                            div {
                                class: "file-modal-head",
                                h2 { "{current_media.image_name}" }
                                button {
                                    class: "file-modal-close",
                                    onclick: move |_| selected_media.set(None),
                                    "Close"
                                }
                            }

                            if is_image(&current_media) {
                                div {
                                    class: "file-markup-toolbar",
                                    button {
                                        class: "file-markup-save",
                                        onclick: move |_| {
                                            let project_id = project_id_for_save.clone();
                                            let block_id = block_id_for_save.clone();
                                            let image_id = current_media_for_save.image_id.clone();
                                            let rects = markup_rects();
                                            spawn(async move {
                                                match api_update_image_markup(&project_id, &block_id, &image_id, rects.clone()).await {
                                                    Ok(updated) => {
                                                        {
                                                            let mut all_media = media_items.write();
                                                            for media in all_media.iter_mut() {
                                                                if media.image_id == updated.image_id {
                                                                    *media = updated.clone();
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                        markup_rects.set(updated.markup_rects.clone());
                                                        selected_media.set(Some(updated));
                                                        crate::shell::progress::show_success("Markup saved");
                                                    }
                                                    Err(e) => {
                                                        crate::shell::progress::show_error_persistent(
                                                            &format!("Failed to save markup: {}", e),
                                                        );
                                                    }
                                                }
                                            });
                                        },
                                        "Save"
                                    }
                                    button {
                                        class: "file-markup-clear",
                                        onclick: move |_| {
                                            markup_rects.set(Vec::new());
                                            draft_rect.set(None);
                                            drag_start.set(None);
                                        },
                                        "Clear"
                                    }
                                }
                                div {
                                    id: "{canvas_id}",
                                    class: "file-markup-canvas",
                                    onmousedown: move |evt| {
                                        if let Some((w, h)) = element_dimensions(&canvas_id_for_down) {
                                            let p = evt.element_coordinates();
                                            let x = clamp01(p.x / w);
                                            let y = clamp01(p.y / h);
                                            drag_start.set(Some((x, y)));
                                            draft_rect.set(None);
                                        }
                                    },
                                    onmousemove: move |evt| {
                                        if let Some((sx, sy)) = drag_start() {
                                            if let Some((w, h)) = element_dimensions(&canvas_id_for_move) {
                                                let p = evt.element_coordinates();
                                                let x = clamp01(p.x / w);
                                                let y = clamp01(p.y / h);
                                                draft_rect.set(Some(rect_from_points(sx, sy, x, y)));
                                            }
                                        }
                                    },
                                    onmouseup: move |_| {
                                        if let Some(rect) = draft_rect() {
                                            if rect.width > 0.003 && rect.height > 0.003 {
                                                let mut rects = markup_rects.write();
                                                rects.push(rect);
                                            }
                                        }
                                        draft_rect.set(None);
                                        drag_start.set(None);
                                    },
                                    onmouseleave: move |_| {
                                        if let Some(rect) = draft_rect() {
                                            if rect.width > 0.003 && rect.height > 0.003 {
                                                let mut rects = markup_rects.write();
                                                rects.push(rect);
                                            }
                                        }
                                        draft_rect.set(None);
                                        drag_start.set(None);
                                    },
                                    img {
                                        class: "file-markup-image",
                                        src: "{src}",
                                        alt: "{current_media.image_name}",
                                    }
                                    for (idx, rect) in markup_rects().iter().enumerate() {
                                        {
                                            let idx_for_remove = idx;
                                            let rect_style = format!(
                                                "left:{}%;top:{}%;width:{}%;height:{}%;",
                                                rect.x * 100.0,
                                                rect.y * 100.0,
                                                rect.width * 100.0,
                                                rect.height * 100.0
                                            );
                                            rsx! {
                                                div {
                                                    class: "file-markup-rect",
                                                    style: "{rect_style}",
                                                    button {
                                                        class: "file-markup-rect-remove",
                                                        onclick: move |e| {
                                                            e.stop_propagation();
                                                            let mut rects = markup_rects.write();
                                                            if idx_for_remove < rects.len() {
                                                                rects.remove(idx_for_remove);
                                                            }
                                                        },
                                                        "×"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if let Some(rect) = draft_rect() {
                                        {
                                            let draft_style = format!(
                                                "left:{}%;top:{}%;width:{}%;height:{}%;",
                                                rect.x * 100.0,
                                                rect.y * 100.0,
                                                rect.width * 100.0,
                                                rect.height * 100.0
                                            );
                                            rsx! {
                                                div {
                                                    class: "file-markup-rect draft",
                                                    style: "{draft_style}",
                                                }
                                            }
                                        }
                                    }
                                }
                            } else if is_video(&current_media) {
                                video {
                                    class: "file-modal-video",
                                    src: "{src}",
                                    controls: true,
                                    autoplay: false,
                                }
                            } else {
                                a {
                                    class: "file-modal-link",
                                    href: "{src}",
                                    target: "_blank",
                                    "Open file"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
