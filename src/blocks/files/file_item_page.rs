use crate::blocks::annotations::comment_dialog::CommentDialog;
use crate::blocks::annotations::models::CommentThread;
use crate::blocks::annotations::state::{
    state_add_comment, state_create_thread, state_delete_thread, state_load_threads, state_resolve_thread,
};
use crate::core::client::{to_cloudfront_media_url, to_cloudfront_url};
use crate::core::{LoadingScreen, Theme, THEME};
use crate::Route;
use crate::media::api::api_get_image;
use crate::media::Image;
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

const FILE_ITEM_PAGE_CSS: &str = include_str!("file_item_page.css");
const CLOSE_LIGHT: Asset = asset!("/assets/icons/close-light.svg");
const CLOSE_DARK: Asset = asset!("/assets/icons/close-dark.svg");

fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
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

fn comment_rect_style(world_x: f64, world_y: f64) -> String {
    format!("left:{}%;top:{}%;", clamp01(world_x) * 100.0, clamp01(world_y) * 100.0)
}

#[cfg(target_arch = "wasm32")]
fn element_rect(element_id: &str) -> Option<(f64, f64, f64, f64)> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let element = document.get_element_by_id(element_id)?;
    let rect = element.get_bounding_client_rect();
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return None;
    }
    Some((rect.left(), rect.top(), rect.width(), rect.height()))
}

#[cfg(not(target_arch = "wasm32"))]
fn element_rect(_element_id: &str) -> Option<(f64, f64, f64, f64)> {
    None
}

fn local_coords_from_client(element_id: &str, client_x: f64, client_y: f64) -> Option<(f64, f64, f64, f64)> {
    let (left, top, width, height) = element_rect(element_id)?;
    Some((client_x - left, client_y - top, width, height))
}

#[component]
pub fn FileItemPage(
    project_id: String,
    block_id: String,
    block_name: String,
    block_type: String,
    image_id: String,
    image_name: String,
) -> Element {
    let nav = use_navigator();
    let close_icon = if THEME() == Theme::Dark { CLOSE_DARK } else { CLOSE_LIGHT };
    let image_name_display = crate::core::route_utils::decode_route_segment(&image_name);
    let mut media_item = use_signal(|| None::<Image>);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut comment_threads: Signal<Vec<CommentThread>> = use_signal(|| Vec::new());
    let mut comment_dialog: Signal<Option<(f64, f64, f64, f64, String)>> = use_signal(|| None);
    let mut zoom_level: Signal<f64> = use_signal(|| 1.0);
    let mut pan_x: Signal<f64> = use_signal(|| 0.0);
    let mut pan_y: Signal<f64> = use_signal(|| 0.0);
    let mut drag_active: Signal<bool> = use_signal(|| false);
    let mut drag_moved: Signal<bool> = use_signal(|| false);
    let mut drag_start: Signal<(f64, f64)> = use_signal(|| (0.0, 0.0));
    let mut drag_last: Signal<(f64, f64)> = use_signal(|| (0.0, 0.0));
    let mut suppress_next_click: Signal<bool> = use_signal(|| false);
    let mut video_playing: Signal<bool> = use_signal(|| false);

    let block_id_for_load = block_id.clone();
    let image_id_for_load = image_id.clone();
    use_effect(move || {
        loading.set(true);
        error.set(None);
        let block_id = block_id_for_load.clone();
        let image_id = image_id_for_load.clone();
        spawn(async move {
            match api_get_image(&block_id, &image_id).await {
                Ok(item) => media_item.set(Some(item)),
                Err(e) => error.set(Some(e)),
            }
            state_load_threads(&image_id, comment_threads).await;
            loading.set(false);
        });
    });

    let pid = project_id.clone();
    let bid = block_id.clone();
    let bname = block_name.clone();
    let btype = block_type.clone();

    rsx! {
        style { {FILE_ITEM_PAGE_CSS} }
        div {
            class: "file-item-page",
            button {
                class: "file-item-close",
                onclick: move |_| {
                    nav.push(Route::FileBlockPage {
                        project_id: pid.clone(),
                        block_id: bid.clone(),
                        block_name: bname.clone(),
                        block_type: btype.clone(),
                    });
                },
                img { src: close_icon, alt: "Close", class: "file-item-close-icon" }
            }
            if loading() {
                LoadingScreen { text: "Loading file".to_string() }
            } else if let Some(err) = error() {
                div { class: "file-item-error", "{err}" }
            } else if let Some(media) = media_item() {
                {
                    let canvas_id = format!("file-item-canvas-{}", media.image_id);
                    let canvas_id_for_click = canvas_id.clone();
                    let src = if is_video(&media) {
                        to_cloudfront_media_url(&media.url)
                    } else {
                        to_cloudfront_url(&media.url)
                    };
                    let zoom_layer_style = format!("transform: translate({}px, {}px) scale({});", pan_x(), pan_y(), zoom_level());
                    rsx! {
                        div {
                            class: "file-item-stage",
                            div {
                                id: "{canvas_id}",
                                class: "file-item-canvas",
                                onwheel: move |evt| {
                                    evt.prevent_default();
                                    let old_zoom = zoom_level();
                                    let dy = evt.delta().strip_units().y;
                                    let factor = (-dy * 0.0015).exp().clamp(0.85, 1.2);
                                    let new_zoom = (old_zoom * factor).clamp(1.0, 6.0);
                                    let p = evt.element_coordinates();
                                    let next_pan_x = p.x - (p.x - pan_x()) * (new_zoom / old_zoom);
                                    let next_pan_y = p.y - (p.y - pan_y()) * (new_zoom / old_zoom);
                                    zoom_level.set(new_zoom);
                                    if new_zoom <= 1.0 {
                                        pan_x.set(0.0);
                                        pan_y.set(0.0);
                                    } else {
                                        pan_x.set(next_pan_x);
                                        pan_y.set(next_pan_y);
                                    }
                                },
                                onmousedown: move |evt| {
                                    let p = evt.element_coordinates();
                                    drag_active.set(true);
                                    drag_moved.set(false);
                                    drag_start.set((p.x, p.y));
                                    drag_last.set((p.x, p.y));
                                },
                                onmousemove: move |evt| {
                                    if !drag_active() { return; }
                                    let p = evt.element_coordinates();
                                    let (sx, sy) = drag_start();
                                    let travel = ((p.x - sx).powi(2) + (p.y - sy).powi(2)).sqrt();
                                    if travel > 4.0 {
                                        drag_moved.set(true);
                                    }
                                    if zoom_level() > 1.0 {
                                        let (lx, ly) = drag_last();
                                        pan_x.set(pan_x() + (p.x - lx));
                                        pan_y.set(pan_y() + (p.y - ly));
                                        drag_last.set((p.x, p.y));
                                    }
                                },
                                onmouseup: move |_| {
                                    if drag_active() && drag_moved() {
                                        suppress_next_click.set(true);
                                    }
                                    drag_active.set(false);
                                },
                                onmouseleave: move |_| {
                                    if drag_active() && drag_moved() {
                                        suppress_next_click.set(true);
                                    }
                                    drag_active.set(false);
                                },
                                onpointerdown: move |evt| {
                                    let pointer_type = format!("{:?}", evt.data.pointer_type());
                                    if !pointer_type.contains("Touch") { return; }
                                    evt.prevent_default();
                                    let p = evt.element_coordinates();
                                    drag_active.set(true);
                                    drag_moved.set(false);
                                    drag_start.set((p.x, p.y));
                                    drag_last.set((p.x, p.y));
                                },
                                onpointermove: move |evt| {
                                    let pointer_type = format!("{:?}", evt.data.pointer_type());
                                    if !pointer_type.contains("Touch") { return; }
                                    if !drag_active() { return; }
                                    evt.prevent_default();
                                    let p = evt.element_coordinates();
                                    let (sx, sy) = drag_start();
                                    let travel = ((p.x - sx).powi(2) + (p.y - sy).powi(2)).sqrt();
                                    if travel > 4.0 {
                                        drag_moved.set(true);
                                    }
                                    if zoom_level() > 1.0 {
                                        let (lx, ly) = drag_last();
                                        pan_x.set(pan_x() + (p.x - lx));
                                        pan_y.set(pan_y() + (p.y - ly));
                                        drag_last.set((p.x, p.y));
                                    }
                                },
                                onpointerup: move |evt| {
                                    let pointer_type = format!("{:?}", evt.data.pointer_type());
                                    if !pointer_type.contains("Touch") { return; }
                                    if drag_active() && drag_moved() {
                                        suppress_next_click.set(true);
                                    }
                                    drag_active.set(false);
                                },
                                onpointercancel: move |_| {
                                    if drag_active() && drag_moved() {
                                        suppress_next_click.set(true);
                                    }
                                    drag_active.set(false);
                                },
                                onclick: move |evt| {
                                    if suppress_next_click() {
                                        suppress_next_click.set(false);
                                        return;
                                    }
                                    let c = evt.client_coordinates();
                                    if let Some((local_x, local_y, w, h)) = local_coords_from_client(&canvas_id_for_click, c.x, c.y) {
                                        let media_x = (local_x - pan_x()) / zoom_level();
                                        let media_y = (local_y - pan_y()) / zoom_level();
                                        let world_x = clamp01(media_x / w);
                                        let world_y = clamp01(media_y / h);
                                        let tid = uuid::Uuid::new_v4().to_string();
                                        comment_threads.write().push(CommentThread {
                                            id: tid.clone(),
                                            world_x,
                                            world_y,
                                            resolved: false,
                                            comments: Vec::new(),
                                            persisted: false,
                                        });
                                        comment_dialog.set(Some((world_x, world_y, c.x, c.y, tid)));
                                    }
                                },
                                div {
                                    class: "file-item-zoom-layer",
                                    style: "{zoom_layer_style}",
                                    if is_video(&media) {
                                        div {
                                            class: "file-item-video-wrapper",
                                            video {
                                                id: "file-item-video-player",
                                                class: "file-item-video",
                                                src: "{src}",
                                                controls: true,
                                                autoplay: false,
                                                playsinline: true,
                                                onmousedown: move |e| e.stop_propagation(),
                                                onpointerdown: move |e| e.stop_propagation(),
                                                onclick: move |e| e.stop_propagation(),
                                            }
                                            if !video_playing() {
                                                div {
                                                    class: "file-item-play-overlay",
                                                    onclick: move |e| {
                                                        e.stop_propagation();
                                                        video_playing.set(true);
                                                        #[cfg(target_arch = "wasm32")]
                                                        {
                                                            if let Some(window) = web_sys::window() {
                                                                if let Some(doc) = window.document() {
                                                                    if let Some(el) = doc.get_element_by_id("file-item-video-player") {
                                                                        if let Ok(vid) = el.dyn_into::<web_sys::HtmlVideoElement>() {
                                                                            let _ = vid.play();
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    },
                                                    div { class: "file-item-play-btn",
                                                        svg {
                                                            width: "32",
                                                            height: "32",
                                                            view_box: "0 0 24 24",
                                                            fill: "white",
                                                            path { d: "M8 5v14l11-7z" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else if is_image(&media) {
                                        img {
                                            class: "file-item-image",
                                            src: "{src}",
                                            alt: "{media.image_name}",
                                        }
                                    } else if is_pdf(&media) {
                                        iframe {
                                            class: "file-item-pdf",
                                            src: "{pdf_thumbnail_src(&src)}",
                                            title: "{media.image_name}",
                                        }
                                    } else {
                                        div {
                                            class: "file-item-generic",
                                            a {
                                                href: "{src}",
                                                target: "_blank",
                                                "Open file"
                                            }
                                        }
                                    }
                                    for (idx, thread) in comment_threads().iter().enumerate() {
                                        {
                                            let tid = thread.id.clone();
                                            let world_x = thread.world_x;
                                            let world_y = thread.world_y;
                                            let rect_style = comment_rect_style(world_x, world_y);
                                            rsx! {
                                                // Rect marker (kept for later use)
                                                // button {
                                                //     key: "{tid}",
                                                //     class: if thread.resolved { "file-item-comment-rect resolved" } else { "file-item-comment-rect" },
                                                //     style: "{rect_style}",
                                                //     "data-index": "{idx + 1}",
                                                //     onmousedown: move |e| {
                                                //         e.stop_propagation();
                                                //     },
                                                //     onclick: move |e| {
                                                //         e.stop_propagation();
                                                //         comment_dialog.set(Some((world_x, world_y, e.client_coordinates().x, e.client_coordinates().y, tid.clone())));
                                                //     },
                                                // }
                                                button {
                                                    key: "{tid}",
                                                    class: if thread.resolved { "file-item-comment-arrow resolved" } else { "file-item-comment-arrow" },
                                                    style: "{rect_style}",
                                                    onmousedown: move |e| {
                                                        e.stop_propagation();
                                                    },
                                                    onclick: move |e| {
                                                        e.stop_propagation();
                                                        comment_dialog.set(Some((world_x, world_y, e.client_coordinates().x, e.client_coordinates().y, tid.clone())));
                                                    },
                                                    span { class: "file-item-comment-arrow-shaft" }
                                                    span { class: "file-item-comment-arrow-head" }
                                                    span { class: "file-item-comment-arrow-badge", "{idx + 1}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "file-item-error", "File not found" }
            }
        }

        if let Some((world_x, world_y, screen_x, screen_y, thread_id)) = comment_dialog() {
            {
                let thread = comment_threads.read().iter().find(|t| t.id == thread_id).cloned();
                let parent_id_for_post = image_id.clone();
                let parent_id_for_resolve = image_id.clone();
                let parent_id_for_delete = image_id.clone();
                rsx! {
                    CommentDialog {
                        screen_x: screen_x,
                        screen_y: screen_y,
                        thread: thread,
                        on_post: move |text: String| {
                            let parent_id = parent_id_for_post.clone();
                            let tid = thread_id.clone();
                            let has_comments = comment_threads.read().iter()
                                .find(|t| t.id == tid)
                                .map_or(false, |t| !t.comments.is_empty());
                            if has_comments {
                                state_add_comment(&parent_id, &tid, &text, comment_threads);
                            } else {
                                state_create_thread(&parent_id, &tid, world_x, world_y, &text, comment_threads);
                            }
                            comment_dialog.set(None);
                        },
                        on_resolve: move |_| {
                            if let Some((_, _, _, _, ref tid)) = comment_dialog() {
                                let parent_id = parent_id_for_resolve.clone();
                                state_resolve_thread(&parent_id, tid, comment_threads);
                            }
                        },
                        on_delete: move |_| {
                            if let Some((_, _, _, _, ref tid)) = comment_dialog() {
                                let parent_id = parent_id_for_delete.clone();
                                comment_dialog.set(None);
                                state_delete_thread(&parent_id, tid, comment_threads);
                            }
                        },
                        on_close: move |_| {
                            if let Some((_, _, _, _, ref tid)) = comment_dialog() {
                                let tid = tid.clone();
                                let has_comments = comment_threads.read().iter()
                                    .find(|t| t.id == tid)
                                    .map_or(false, |t| !t.comments.is_empty());
                                if !has_comments {
                                    comment_threads.write().retain(|t| t.id != tid);
                                }
                            }
                            comment_dialog.set(None);
                        },
                    }
                }
            }
        }
    }
}
