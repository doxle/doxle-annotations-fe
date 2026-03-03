use std::cell::Cell;
use std::rc::Rc;

use dioxus::prelude::*;
use crate::shell::{THEME, Theme, AppNavbar};
use crate::Route;
use super::bulk_import_api::{self, UploadSession};

const IMPORT_BLOCK_CSS: &str = include_str!("import_block_page.css");

fn format_size(bytes: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    if bytes >= GB {
        format!("{:.1} GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.0} MB", bytes / MB)
    } else {
        format!("{:.0} KB", bytes / KB)
    }
}

fn format_elapsed(secs: u32) -> String {
    let m = secs / 60;
    let s = secs % 60;
    if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ImportBlockPageProps {
    pub block_id: String,
    pub block_name: String,
    pub block_type: String,
}

#[component]
pub fn ImportBlockPage(props: ImportBlockPageProps) -> Element {
    let mut selected_file: Signal<Option<web_sys::File>> = use_signal(|| None);
    let mut is_uploading = use_signal(|| false);
    let mut bytes_uploaded = use_signal(|| 0usize);
    let mut bytes_total = use_signal(|| 0usize);
    let mut elapsed_secs = use_signal(|| 0u32);
    let mut import_phase = use_signal(|| String::new());   // current phase label
    let mut phase_done = use_signal(|| 0usize);             // items done in phase
    let mut phase_total = use_signal(|| 0usize);            // total items in phase
    let mut total_annotations = use_signal(|| 0usize);      // running annotation count
    let nav = use_navigator();

    // Shared cancel flag — set to true to stop the upload loop
    let mut cancel_flag: Signal<Rc<Cell<bool>>> = use_signal(|| Rc::new(Cell::new(false)));
    // Session info needed to call abort
    let mut upload_session: Signal<Rc<Cell<Option<UploadSession>>>> = use_signal(|| Rc::new(Cell::new(None)));

    let block_id = props.block_id.clone();
    let block_name = props.block_name.clone();
    let block_type = props.block_type.clone();

    // Timer: ticks every second while uploading
    let _timer = use_future(move || {
        let is_uploading = is_uploading.clone();
        let mut elapsed_secs = elapsed_secs.clone();
        async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(1_000).await;
                if *is_uploading.read() {
                    let cur = *elapsed_secs.read();
                    elapsed_secs.set(cur + 1);
                }
            }
        }
    });

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        evt.stop_propagation();

        if *is_uploading.read() {
            crate::shell::progress::show_info("Upload in progress...");
            return;
        }

        let file = match selected_file.read().clone() {
            Some(f) => f,
            None => {
                crate::shell::progress::show_error("Please select a zip file to import");
                return;
            }
        };

        let block_id = block_id.clone();
        let block_name = block_name.clone();
        let block_type = block_type.clone();
        let nav = nav.clone();

        // Reset state
        let flag = Rc::new(Cell::new(false));
        let sess: Rc<Cell<Option<UploadSession>>> = Rc::new(Cell::new(None));
        cancel_flag.read(); // just to subscribe
        upload_session.read();
        cancel_flag.with_mut(|f| *f = flag.clone());
        upload_session.with_mut(|s| *s = sess.clone());

        spawn(async move {
            is_uploading.set(true);
            bytes_uploaded.set(0);
            bytes_total.set(0);
            elapsed_secs.set(0);
            import_phase.set(String::new());
            phase_done.set(0);
            phase_total.set(0);
            total_annotations.set(0);

            match bulk_import_api::upload_import_zip(
                &block_id,
                file,
                flag.clone(),
                sess.clone(),
                move |current, total| {
                    bytes_uploaded.set(current);
                    bytes_total.set(total);
                },
            )
            .await
            {
                Ok(result) => {
                    let s3_key = result.s3_key.clone();
                    let import_id = result.import_id.clone();

                    // ── Step 1: Parse ──
                    import_phase.set("Parsing zip...".to_string());
                    let manifest = match bulk_import_api::parse_import(&block_id, &import_id, &s3_key).await {
                        Ok(m) => m,
                        Err(e) => {
                            crate::shell::progress::show_error_for(&format!("Parse failed: {}", e), 5);
                            is_uploading.set(false);
                            return;
                        }
                    };

                    let mut cumulative_labels = 0usize;
                    let mut cumulative_tasks = 0usize;
                    let mut cumulative_images = 0usize;
                    let mut cumulative_annotations = 0usize;

                    // ── Step 2: Labels ──
                    if manifest.total_labels > 0 {
                        import_phase.set("Creating labels...".to_string());
                        phase_done.set(0);
                        phase_total.set(manifest.total_labels);
                        let mut offset = 0usize;
                        let limit = 50usize;
                        while offset < manifest.total_labels {
                            match bulk_import_api::process_batch(&block_id, &s3_key, "labels", offset, limit).await {
                                Ok(r) => {
                                    cumulative_labels += r.labels_created;
                                    offset += r.processed;
                                    phase_done.set(offset.min(manifest.total_labels));
                                }
                                Err(e) => {
                                    crate::shell::progress::show_error_for(&format!("Labels failed: {}", e), 5);
                                    is_uploading.set(false);
                                    return;
                                }
                            }
                        }
                    }

                    // ── Step 3: Tasks ──
                    if manifest.total_tasks > 0 {
                        import_phase.set("Creating tasks...".to_string());
                        phase_done.set(0);
                        phase_total.set(manifest.total_tasks);
                        let mut offset = 0usize;
                        let limit = 50usize;
                        while offset < manifest.total_tasks {
                            match bulk_import_api::process_batch(&block_id, &s3_key, "tasks", offset, limit).await {
                                Ok(r) => {
                                    cumulative_tasks += r.tasks_created;
                                    offset += r.processed;
                                    phase_done.set(offset.min(manifest.total_tasks));
                                }
                                Err(e) => {
                                    crate::shell::progress::show_error_for(&format!("Tasks failed: {}", e), 5);
                                    is_uploading.set(false);
                                    return;
                                }
                            }
                        }
                    }

                    // ── Step 4: Images + Annotations ──
                    if manifest.total_images > 0 {
                        import_phase.set("Uploading images...".to_string());
                        phase_done.set(0);
                        phase_total.set(manifest.total_images);
                        let mut offset = 0usize;
                        let limit = 5usize;
                        while offset < manifest.total_images {
                            match bulk_import_api::process_batch(&block_id, &s3_key, "images", offset, limit).await {
                                Ok(r) => {
                                    cumulative_images += r.images_created;
                                    cumulative_annotations += r.annotations_created;
                                    offset += r.processed;
                                    phase_done.set(offset.min(manifest.total_images));
                                    total_annotations.set(cumulative_annotations);
                                }
                                Err(e) => {
                                    crate::shell::progress::show_error_for(&format!("Images failed: {}", e), 5);
                                    is_uploading.set(false);
                                    return;
                                }
                            }
                        }
                    }

                    // ── Step 5: Cleanup ──
                    let _ = bulk_import_api::cleanup_import(&block_id, &s3_key).await;

                    crate::shell::progress::show_success_for(
                        &format!("Import complete — {} images, {} annotations, {} labels",
                            cumulative_images, cumulative_annotations, cumulative_labels),
                        5,
                    );
                    nav.push(Route::TasksListPage {
                        block_id,
                        block_name,
                        block_type,
                    });
                }
                Err(e) if e == "cancelled" => {
                    crate::shell::progress::show_info("Upload cancelled");
                }
                Err(e) => {
                    dioxus::logger::tracing::error!("❌ Import upload failed: {}", e);
                    crate::shell::progress::show_error(&format!("Import failed: {}", e));
                }
            }
            is_uploading.set(false);
        });
    };

    // Stop / cancel handler
    let handle_stop = move |_| {
        // 1. Signal the upload loop to stop
        cancel_flag.read().set(true);

        // 2. Fire-and-forget: call BE to abort the multipart upload and clean orphaned parts
        let sess_rc = upload_session.read().clone();
        spawn(async move {
            let maybe_sess = sess_rc.take();
            if let Some(sess) = maybe_sess {
                if let Err(e) = bulk_import_api::abort_import_upload(&sess).await {
                    dioxus::logger::tracing::error!("Failed to abort S3 upload: {}", e);
                }
            }
        });
    };

    let uploading = *is_uploading.read();
    let total = *bytes_total.read();
    let current = *bytes_uploaded.read();
    let pct = if total > 0 { (current as f64 / total as f64 * 100.0) as u32 } else { 0 };

    rsx! {
        style { {IMPORT_BLOCK_CSS} }
        AppNavbar {}
        div {
            class: "import-page",
            div {
                class: "import-form-container",
                div {
                    class: "import-title-row",
                    h1 { class: "import-title", "Import block ..." }
                }
                form {
                    class: "import-form",
                    onsubmit: handle_submit,
                    autocomplete: "off",

                    div {
                        class: "import-form-group",
                        div {
                            class: "import-file-upload-container",

                            // Hidden file input
                            input {
                                r#type: "file",
                                id: "import-file-input",
                                class: "import-hidden-file-input",
                                accept: ".zip",
                                onchange: move |evt| {
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        use wasm_bindgen::JsCast;
                                        if let Some(web_evt) = evt.downcast::<web_sys::Event>() {
                                            if let Some(target) = web_evt.target() {
                                                if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
                                                    if let Some(files) = input.files() {
                                                        if let Some(file) = files.get(0) {
                                                            let size = file.size();
                                                            crate::shell::progress::show_info_for(
                                                                &format!("Selected: {} ({})", file.name(), format_size(size)),
                                                                2,
                                                            );
                                                            selected_file.set(Some(file));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Custom upload button
                            label {
                                r#for: "import-file-input",
                                class: "import-upload-button",
                                if selected_file.read().is_some() {
                                    {
                                        let f = selected_file.read();
                                        let file = f.as_ref().unwrap();
                                        format!("{} ({})", file.name(), format_size(file.size()))
                                    }
                                } else {
                                    "Choose CVAT export zip file"
                                }
                            }
                        }
                    }

                    // Upload progress (S3 upload phase)
                    if uploading && total > 0 && import_phase.read().is_empty() {
                        div {
                            class: "import-progress-container",
                            div {
                                class: "import-progress-bar",
                                div {
                                    class: "import-progress-fill",
                                    style: "width: {pct}%",
                                }
                            }
                            span {
                                class: "import-progress-text",
                                "{format_size(current as f64)} / {format_size(total as f64)}  ·  {pct}%  ·  {format_elapsed(*elapsed_secs.read())}"
                            }
                        }
                    }

                    // Batch processing progress
                    if uploading && !import_phase.read().is_empty() {
                        {
                            let p_done = *phase_done.read();
                            let p_total = *phase_total.read();
                            let p_pct = if p_total > 0 { (p_done as f64 / p_total as f64 * 100.0) as u32 } else { 0 };
                            let ann_count = *total_annotations.read();
                            rsx! {
                                div {
                                    class: "import-progress-container",
                                    div {
                                        class: "import-progress-bar",
                                        div {
                                            class: "import-progress-fill",
                                            style: "width: {p_pct}%",
                                        }
                                    }
                                    span {
                                        class: "import-progress-text",
                                        "{import_phase.read()}  {p_done}/{p_total}"
                                        if ann_count > 0 {
                                            "  ·  {ann_count} annotations"
                                        }
                                        "  ·  {format_elapsed(*elapsed_secs.read())}"
                                    }
                                }
                            }
                        }
                    }

                    div {
                        class: "import-form-actions",
                        if uploading {
                            button {
                                r#type: "button",
                                class: "import-stop-button",
                                onclick: handle_stop,
                                "Stop"
                            }
                        } else {
                            button {
                                r#type: "button",
                                class: "import-back-button",
                                onclick: move |_| {
                                    nav.push(Route::DashboardPage {});
                                },
                                "Back"
                            }
                            button {
                                r#type: "submit",
                                class: "import-submit-button",
                                "Import"
                            }
                        }
                    }
                }
            }
        }
    }
}
