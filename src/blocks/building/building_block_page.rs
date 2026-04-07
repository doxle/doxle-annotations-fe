use crate::media::api::{list_block_media, upload_image_for_block};
use crate::media::Image;
use crate::core::client::to_cloudfront_url;
use crate::core::{AppNavbar, BottomBar, LoadingScreen, Theme, THEME};
use dioxus::prelude::*;
use futures::stream::{self, StreamExt};

const BUILDING_BLOCK_PAGE_CSS: &str = include_str!("building_block_page.css");
const DOG_LIGHT_ICON: Asset = asset!("/assets/icons/dog-light.svg");
const DOG_DARK_ICON: Asset = asset!("/assets/icons/dog-dark.svg");

#[derive(Clone, PartialEq)]
struct UploadItem {
    name: String,
    status: String,
}

fn is_pdf(media: &Image) -> bool {
    media.image_name.to_ascii_lowercase().ends_with(".pdf")
        || media.url.to_ascii_lowercase().contains(".pdf")
}

fn pdf_thumbnail_src(url: &str) -> String {
    format!("{url}#page=1&toolbar=0&navpanes=0&scrollbar=0&view=FitH")
}

#[component]
pub fn BuildingBlockPage(
    project_id: String,
    block_id: String,
    block_name: String,
    block_type: String,
) -> Element {
    let _ = &block_type;
    let block_name_display = crate::core::route_utils::decode_route_segment(&block_name);
    let mut media_items = use_signal(Vec::<Image>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut is_uploading = use_signal(|| false);
    let mut selected_media = use_signal(|| None::<Image>);
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
                Ok(items) => media_items.set(items),
                Err(e) => error.set(Some(e)),
            }
            loading.set(false);
        });
    });

    rsx! {
        style { {BUILDING_BLOCK_PAGE_CSS} }
        AppNavbar {}
        div {
            class: "building-block-page",
            div {
                class: "building-block-head",
                h1 { class: "building-block-title", "{block_name_display}" }
                label {
                    r#for: "building-block-upload-input",
                    class: "building-block-upload-btn",
                    if is_uploading() { "Uploading..." } else { "Upload plans" }
                }
            }
            input {
                r#type: "file",
                id: "building-block-upload-input",
                class: "building-block-upload-input",
                multiple: true,
                accept: "image/*,application/pdf,.pdf",
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
                                        for i in 0..files.length() {
                                            if let Some(file) = files.get(i) {
                                                files_to_upload.push(file);
                                            }
                                        }

                                        if files_to_upload.is_empty() {
                                            return;
                                        }

                                        is_uploading.set(true);

                                        let project_id = project_id.clone();
                                        let block_id = block_id.clone();
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
                                            let total_uploads = upload_futures.len();
                                            let upload_started = web_time::Instant::now();
                                            let mut failed = 0usize;
                                            crate::core::progress::show_progress(
                                                &format!("Uploading plans 0/{}", total_uploads),
                                                0,
                                                total_uploads,
                                                0,
                                            );

                                            let mut stream = stream::iter(upload_futures).buffer_unordered(3);
                                            let mut finished = 0usize;

                                            while let Some((idx, name, result)) = stream.next().await {
                                                finished += 1;
                                                match result {
                                                    Ok(_) => {},
                                                    Err(_) => {
                                                        failed += 1;
                                                    },
                                                };
                                                crate::core::progress::show_progress(
                                                    &format!(
                                                        "Uploading plans {}/{}{}",
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
                                                        "Uploaded {} plans, {} failed",
                                                        total_uploads.saturating_sub(failed),
                                                        failed
                                                    ),
                                                );
                                            } else {
                                                crate::core::progress::show_success(
                                                    &format!("Uploaded {} plans", total_uploads),
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

            div {
                class: "building-block-content",
                if loading() {
                    LoadingScreen { text: "Loading plans".to_string() }
                } else if let Some(err) = error() {
                    div { class: "building-block-error", "{err}" }
                } else if media_items().is_empty() {
                    div {
                        class: "building-block-empty",
                        img {
                            class: "building-block-empty-icon",
                            src: dog_icon,
                            alt: "No plans"
                        }
                        span { "No plans uploaded yet" }
                    }
                } else {
                    div {
                        class: "building-block-grid",
                        for media in media_items().iter() {
                            {
                                let media_for_click = media.clone();
                                let src = to_cloudfront_url(&media.url);
                                let open_url = src.clone();
                                let should_open_direct = is_pdf(media);
                                rsx! {
                                    button {
                                        class: "plan-card",
                                        onclick: move |_| {
                                            if should_open_direct {
                                                #[cfg(target_arch = "wasm32")]
                                                {
                                                    if let Some(window) = web_sys::window() {
                                                        let _ = window.open_with_url_and_target(&open_url, "_blank");
                                                    }
                                                }
                                                return;
                                            }
                                            selected_media.set(Some(media_for_click.clone()));
                                        },
                                        div {
                                            class: "plan-card-thumb",
                                            if is_pdf(media) {
                                                iframe {
                                                    class: "plan-card-preview",
                                                    src: "{pdf_thumbnail_src(&src)}",
                                                    title: "{media.image_name}",
                                                }
                                            } else {
                                                img {
                                                    class: "plan-card-image",
                                                    src: "{src}",
                                                    alt: "{media.image_name}",
                                                }
                                            }
                                        }
                                        div {
                                            class: "plan-card-footer",
                                            span { class: "plan-card-name", "{media.image_name}" }
                                        }
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
                let src = to_cloudfront_url(&current_media.url);
                rsx! {
                    div {
                        class: "plan-modal-backdrop",
                        onclick: move |_| selected_media.set(None),
                        div {
                            class: "plan-modal",
                            onclick: move |e| e.stop_propagation(),
                            div {
                                class: "plan-modal-head",
                                h2 { "{current_media.image_name}" }
                                button {
                                    class: "plan-modal-close",
                                    onclick: move |_| selected_media.set(None),
                                    "Close"
                                }
                            }

                            if is_pdf(&current_media) {
                                iframe {
                                    class: "plan-modal-pdf",
                                    src: "{src}",
                                    title: "{current_media.image_name}",
                                }
                            } else {
                                img {
                                    class: "plan-modal-image",
                                    src: "{src}",
                                    alt: "{current_media.image_name}",
                                }
                            }
                        }
                    }
                }
            }
        }
        BottomBar { project_id: project_id.clone() }
    }
}
