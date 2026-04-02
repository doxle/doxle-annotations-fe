use crate::blocks::annotations::api::api_list_threads;
use crate::media::api::{list_block_media, upload_image_for_block};
use crate::media::Image;
use crate::core::client::to_cloudfront_url;
use crate::core::{AppNavbar, LoadingScreen, Theme, THEME};
use crate::Route;
use dioxus::prelude::*;
use futures::stream::{self, StreamExt};
use std::collections::HashMap;

const FILE_BLOCK_PAGE_CSS: &str = include_str!("file_block_page.css");
const DOG_LIGHT_ICON: Asset = asset!("/assets/icons/dog-light.svg");
const DOG_DARK_ICON: Asset = asset!("/assets/icons/dog-dark.svg");
const COMMENT_BLUE_ICON: Asset = asset!("/assets/icons/comment-blue.svg");

#[derive(Clone, PartialEq)]
struct UploadItem {
    name: String,
    status: String,
}


fn is_video(media: &Image) -> bool {
    media.media_type == "video"
}

fn is_image(media: &Image) -> bool {
    media.media_type == "image"
}

fn is_pdf(media: &Image) -> bool {
    media.image_name.to_ascii_lowercase().ends_with(".pdf")
        || media.url.to_ascii_lowercase().contains(".pdf")
}

fn pdf_thumbnail_src(url: &str) -> String {
    format!("{url}#page=1&toolbar=0&navpanes=0&scrollbar=0&view=FitH")
}

async fn fetch_comment_counts(image_ids: Vec<String>) -> HashMap<String, usize> {
    let comment_count_futures = image_ids.into_iter().map(|image_id| async move {
        let count = match api_list_threads(&image_id).await {
            Ok(threads) => threads.iter().map(|thread| thread.comments.len()).sum::<usize>(),
            Err(_) => 0,
        };
        (image_id, count)
    });
    let mut stream = stream::iter(comment_count_futures).buffer_unordered(6);
    let mut counts = HashMap::new();
    while let Some((image_id, count)) = stream.next().await {
        counts.insert(image_id, count);
    }
    counts
}


#[component]
pub fn FileBlockPage(
    project_id: String,
    block_id: String,
    block_name: String,
    block_type: String,
) -> Element {
    let block_name_display = crate::core::route_utils::decode_route_segment(&block_name);
    let nav = use_navigator();
    let mut media_items = use_signal(Vec::<Image>::new);
    let mut comment_counts = use_signal(HashMap::<String, usize>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut upload_items = use_signal(Vec::<UploadItem>::new);
    let mut upload_current = use_signal(|| 0usize);
    let mut upload_total = use_signal(|| 0usize);
    let mut is_uploading = use_signal(|| false);
    let dog_icon = if THEME() == Theme::Dark { DOG_DARK_ICON } else { DOG_LIGHT_ICON };

    let project_id_for_load = project_id.clone();
    let block_id_for_load = block_id.clone();
    use_effect(move || {
        loading.set(true);
        error.set(None);
        let project_id = project_id_for_load.clone();
        let block_id = block_id_for_load.clone();
        spawn(async move {
            match list_block_media(&project_id, &block_id).await {
                Ok(items) => {
                    let image_ids = items.iter().map(|item| item.image_id.clone()).collect::<Vec<_>>();
                    media_items.set(items);
                    comment_counts.set(fetch_comment_counts(image_ids).await);
                }
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
                h1 { class: "file-block-title", "{block_name_display}" }
                label {
                    r#for: "file-block-upload-input",
                    class: "file-block-upload-btn",
                    if is_uploading() { "Uploading..." } else { "Upload files" }
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
                                            let total_uploads = upload_total();
                                            let upload_started = web_time::Instant::now();
                                            let mut failed = 0usize;
                                            crate::core::progress::show_progress(
                                                &format!("Uploading files 0/{}", total_uploads),
                                                0,
                                                total_uploads,
                                                0,
                                            );

                                            let mut stream = stream::iter(upload_futures).buffer_unordered(3);
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

                                            match list_block_media(&project_id, &block_id).await {
                                                Ok(items) => {
                                                    let image_ids = items.iter().map(|item| item.image_id.clone()).collect::<Vec<_>>();
                                                    media_items.set(items);
                                                    comment_counts.set(fetch_comment_counts(image_ids).await);
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
                    div {
                        class: "file-block-grid",
                        for media in media_items().iter() {
                            {
                                let image_id_for_route = media.image_id.clone();
                                let image_name_for_route = crate::core::route_utils::encode_route_segment(&media.image_name);
                                let src = to_cloudfront_url(&media.url);
                                let project_id_for_route = project_id.clone();
                                let block_id_for_route = block_id.clone();
                                let block_name_for_route = crate::core::route_utils::encode_route_segment(&block_name);
                                let block_type_for_route = block_type.clone();
                                let nav_for_route = nav.clone();
                                rsx! {
                                    button {
                                        class: "file-card",
                                        onclick: move |_| {
                                            nav_for_route.push(Route::FileItemPage {
                                                project_id: project_id_for_route.clone(),
                                                block_id: block_id_for_route.clone(),
                                                block_name: block_name_for_route.clone(),
                                                block_type: block_type_for_route.clone(),
                                                image_id: image_id_for_route.clone(),
                                                image_name: image_name_for_route.clone(),
                                            });
                                        },
                                        div {
                                            class: "file-card-thumb",
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
                                            } else if is_pdf(media) {
                                                iframe {
                                                    class: "file-card-file-preview",
                                                    src: "{pdf_thumbnail_src(&src)}",
                                                    title: "{media.image_name}",
                                                }
                                            } else {
                                                div { class: "file-card-generic", "FILE" }
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
                                            if let Some(comment_count) = comment_counts().get(&media.image_id).copied() {
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
        }

    }
}
