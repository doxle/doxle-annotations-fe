use crate::core::client::{API_BASE_URL, CLOUDFRONT_URL};
use crate::public::estimate_canvas::EstimateCanvas;
use crate::public::estimate_cabinetry::{total_cabinetry_lm_m, CabinetryData, CabinetryPanelSection};
use crate::public::estimate_ewalls::{compute_saved_ewalls_lm_m, EWallsData, EWallsPanelSection};
use crate::public::estimate_footprint::{
    polygon_area_px, scale_px_per_meter_for_area, FootprintData, FootprintPanelSection,
};
use crate::public::estimate_iwalls::{total_iwalls_lm_m, IWallsData, IWallsPanelSection};
use crate::public::estimate_scale::ScalePanelSection;
use crate::public::estimate_types::{
    build_page_layouts, PageLayout, ResultJson, SaveScaleRequest, SavedScaleResponse, ScaleData,
    ScalePoint,
};
use dioxus::prelude::*;
use gloo_net::http::Request;
use std::collections::HashMap;

const CSS: &str = include_str!("estimate_page.css");
const LAST_PUBLIC_PROJECT_ID_KEY: &str = "doxle_public_last_project_id";

#[component]
pub fn EstimatePage(project_id: String) -> Element {
    let mut pages: Signal<Vec<PageLayout>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let mut loading_message = use_signal(|| "Processing your plans...".to_string());
    let mut error = use_signal(|| None::<String>);
    let mut step: Signal<u32> = use_signal(|| 1);
    let mut page_scales: Signal<HashMap<String, ScaleData>> = use_signal(HashMap::new);
    let mut footprints: Signal<HashMap<String, FootprintData>> = use_signal(HashMap::new);
    let mut ewalls: Signal<HashMap<String, EWallsData>> = use_signal(HashMap::new);
    let mut iwalls: Signal<HashMap<String, IWallsData>> = use_signal(HashMap::new);
    let mut cabinetry: Signal<HashMap<String, CabinetryData>> = use_signal(HashMap::new);
    let mut project_name: Signal<String> = use_signal(|| "Project".to_string());
    let mut project_email: Signal<String> = use_signal(String::new);

    let mut scale_points: Signal<Vec<(f64, f64)>> = use_signal(Vec::new);
    let mut active_scale_page_id: Signal<Option<String>> = use_signal(|| None);
    let mut scale_distance_input = use_signal(String::new);

    let mut active_footprint_drawing: Signal<Vec<(f64, f64)>> = use_signal(Vec::new);
    let mut footprint_page_id: Signal<Option<String>> = use_signal(|| None);
    let live_footprint_area_m2: Signal<Option<f64>> = use_signal(|| None);
    let mut active_ewall_drawing: Signal<Vec<(f64, f64)>> = use_signal(Vec::new);
    let mut ewall_page_id: Signal<Option<String>> = use_signal(|| None);
    let live_ewalls_lm_m: Signal<Option<f64>> = use_signal(|| Some(0.0));
    let mut active_iwall_start_point: Signal<Option<(f64, f64)>> = use_signal(|| None);
    let mut iwall_page_id: Signal<Option<String>> = use_signal(|| None);
    let live_iwalls_lm_m: Signal<Option<f64>> = use_signal(|| Some(0.0));
    let mut active_cabinetry_start_point: Signal<Option<(f64, f64)>> = use_signal(|| None);
    let mut cabinetry_page_id: Signal<Option<String>> = use_signal(|| None);
    let live_cabinetry_lm_m: Signal<Option<f64>> = use_signal(|| Some(0.0));
    let mut focus_floor_plan_request = use_signal(|| 0_u64);

    let mut file_id_signal: Signal<Option<String>> = use_signal(|| None);
    let mut poll_count: Signal<u32> = use_signal(|| 0);
    let mut panel_open = use_signal(|| true);
    let mut panel_width_pct = use_signal(|| 30.0_f64);
    let mut is_resizing_panel = use_signal(|| false);
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if loading() {
                        let _ = dioxus::prelude::document::eval(
                            "document.documentElement.classList.add('estimate-loading-active')"
                        );
                    } else {
                        let _ = dioxus::prelude::document::eval(
                            "document.documentElement.classList.remove('estimate-loading-active')"
                        );
                    }
                }
            }
        }
    });

    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::prelude::*;
            use wasm_bindgen::JsCast;

            let window = web_sys::window().unwrap();
            let keydown =
                Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
                    if (e.meta_key() || e.ctrl_key())
                        && (e.code() == "Backslash" || e.key() == "\\")
                    {
                        e.prevent_default();
                        panel_open.set(!panel_open());
                        return;
                    }

                    if e.key() == "Escape" {
                        active_footprint_drawing.set(Vec::new());
                        footprint_page_id.set(None);
                        active_ewall_drawing.set(Vec::new());
                        ewall_page_id.set(None);
                        active_iwall_start_point.set(None);
                        iwall_page_id.set(None);
                        active_cabinetry_start_point.set(None);
                        cabinetry_page_id.set(None);
                    }
                });
            window
                .add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())
                .ok();
            keydown.forget();
        }
    });

    let project_id_for_resume = project_id.clone();
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item(LAST_PUBLIC_PROJECT_ID_KEY, &project_id_for_resume);
                }
            }
        }
    });

    let project_id_for_files = project_id.clone();
    use_effect(move || {
        let project_id_for_files = project_id_for_files.clone();
        loading_message.set("Processing your plans...".to_string());
        spawn(async move {
            let project_url = format!("{}/public/projects/{}", API_BASE_URL, project_id_for_files);
            if let Ok(resp) = Request::get(&project_url).send().await {
                if resp.ok() {
                    if let Ok(proj) = resp.json::<serde_json::Value>().await {
                        if let Some(name) = proj.get("project_name").and_then(|v| v.as_str()) {
                            project_name.set(name.to_string());
                        }
                        if let Some(email) = proj.get("project_email").and_then(|v| v.as_str()) {
                            project_email.set(email.to_string());
                        }
                    }
                }
            }

            let files_url = format!("{}/public/projects/{}/files", API_BASE_URL, project_id_for_files);
            match Request::get(&files_url).send().await {
                Ok(resp) if resp.ok() => {
                    if let Ok(files) = resp.json::<Vec<serde_json::Value>>().await {
                        if let Some(file_id) = files
                            .first()
                            .and_then(|f| f.get("file_id"))
                            .and_then(|v| v.as_str())
                        {
                            file_id_signal.set(Some(file_id.to_string()));
                            return;
                        }
                    }
                    error.set(Some("No uploaded plans found.".to_string()));
                    loading.set(false);
                }
                Ok(resp) => {
                    error.set(Some(format!("Failed to load files ({})", resp.status())));
                    loading.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Network error: {}", e)));
                    loading.set(false);
                }
            }
        });
    });

    let project_id_for_poll = project_id.clone();
    use_future(move || {
        let project_id_for_poll = project_id_for_poll.clone();
        async move {
            loop {
                let Some(file_id) = file_id_signal() else {
                    gloo_timers::future::TimeoutFuture::new(500).await;
                    continue;
                };

                let count = poll_count() + 1;
                poll_count.set(count);
                loading_message.set(format!("Processing your plans... ({}s)", count * 3));

                let result_url = format!(
                    "{}/cdn/app/upload-plan/{}/output/result.json",
                    CLOUDFRONT_URL, file_id
                );

                match Request::get(&result_url).send().await {
                    Ok(resp) if resp.ok() => {
                        if let Ok(result) = resp.json::<ResultJson>().await {
                            let next_pages = build_page_layouts(result.pages);
                            if next_pages.is_empty() {
                                gloo_timers::future::TimeoutFuture::new(1_000).await;
                                continue;
                            }
                            pages.set(next_pages);

                            let scales_url =
                                format!("{}/public/projects/{}/scales", API_BASE_URL, project_id_for_poll);
                            if let Ok(scale_resp) = Request::get(&scales_url).send().await {
                                if scale_resp.ok() {
                                    if let Ok(saved_scales) = scale_resp.json::<Vec<SavedScaleResponse>>().await
                                    {
                                        let mut next_scales = HashMap::new();
                                        for saved_scale in saved_scales {
                                            if let Some(page_id) = saved_scale.page_id {
                                                next_scales.insert(
                                                    page_id.clone(),
                                                    ScaleData {
                                                        page_id,
                                                        point1: (
                                                            saved_scale.reference_start.x,
                                                            saved_scale.reference_start.y,
                                                        ),
                                                        point2: (
                                                            saved_scale.reference_end.x,
                                                            saved_scale.reference_end.y,
                                                        ),
                                                        real_distance_m: saved_scale.real_distance_m,
                                                        px_per_meter: saved_scale.px_per_meter,
                                                    },
                                                );
                                            }
                                        }
                                        page_scales.set(next_scales);
                                    }
                                }
                            }

                            loading.set(false);
                            error.set(None);
                            return;
                        }
                    }
                    _ => {}
                }

                if count >= 100 {
                    error.set(Some(
                        "Processing is taking longer than expected. Please refresh.".to_string(),
                    ));
                    loading.set(false);
                    return;
                }

                gloo_timers::future::TimeoutFuture::new(3_000).await;
            }
        }
    });

    let step_title = match step() {
        1 => "Set your scale",
        2 => "Draw footprint",
        3 => "Trace external walls",
        4 => "Trace internal walls",
        5 => "Trace cabinetry",
        _ => "Generate budget",
    };
    let mobile_show_scale_input = step() == 1 && scale_points().len() == 2;
    let mobile_step_title = match step() {
        1 if mobile_show_scale_input => "Enter distance",
        1 => "Set Scale",
        _ => step_title,
    };
    let mobile_step_subtitle = match step() {
        1 if !mobile_show_scale_input => "Click any 2 points to set scale",
        _ => "",
    };
    let mobile_show_next = step() != 1 || mobile_show_scale_input;

    let step_description = match step() {
        1 => "",
        2 => "",
        3 => "",
        4 => "",
        5 => "",
        _ => "Review your selections and generate your budget estimate.",
    };

    let panel_style = if panel_open() {
        format!("flex: 0 0 {}%;", panel_width_pct())
    } else {
        "display: none;".to_string()
    };
    let project_id_for_mobile_bar = project_id.clone();

    rsx! {
        style { {CSS} }
        div {
            class: if loading() { "estimate-page estimate-page--loading" } else { "estimate-page" },

            onmouseup: move |_| {
                is_resizing_panel.set(false);
            },
            onmousemove: move |evt| {
                if is_resizing_panel() {
                    let x = evt.client_coordinates().x;
                    #[cfg(target_arch = "wasm32")]
                    {
                        if let Some(win) = web_sys::window() {
                            let total_width = win.inner_width().unwrap().as_f64().unwrap_or(1200.0);
                            let pct = ((total_width - x) / total_width * 100.0).clamp(15.0, 50.0);
                            panel_width_pct.set(pct);
                        }
                    }
                }
            },

            EstimateCanvas {
                pages: pages(),
                loading: loading(),
                loading_message: loading_message(),
                error: error(),
                scale_tool_active: step() == 1,
                footprint_tool_active: step() == 2,
                ewalls_tool_active: step() == 3,
                iwalls_tool_active: step() == 4,
                cabinetry_tool_active: step() == 5,
                scales: page_scales(),
                footprints: footprints,
                ewalls: ewalls,
                iwalls: iwalls,
                cabinetry: cabinetry,
                scale_points: scale_points,
                active_scale_page_id: active_scale_page_id,
                scale_distance_input: scale_distance_input,
                active_footprint_drawing: active_footprint_drawing,
                footprint_page_id: footprint_page_id,
                live_footprint_area_m2: live_footprint_area_m2,
                active_ewall_drawing: active_ewall_drawing,
                ewall_page_id: ewall_page_id,
                live_ewalls_lm_m: live_ewalls_lm_m,
                active_iwall_start_point: active_iwall_start_point,
                iwall_page_id: iwall_page_id,
                live_iwalls_lm_m: live_iwalls_lm_m,
                active_cabinetry_start_point: active_cabinetry_start_point,
                cabinetry_page_id: cabinetry_page_id,
                live_cabinetry_lm_m: live_cabinetry_lm_m,
                project_id: project_id.clone(),
                project_email: project_email(),
                focus_floor_plan_request: focus_floor_plan_request(),
            }
            if !loading() {
                div {
                    class: "estimate-mobile-project-meta",
                    h3 {
                        class: "estimate-mobile-project-name",
                        "{project_name}"
                    }
                    p {
                        class: "estimate-mobile-project-email",
                        "{project_email}"
                    }
                }
            }
            if !loading() {
                div {
                    class: "estimate-mobile-bottom-bar",
                    if mobile_show_scale_input {
                        div {
                            class: "estimate-mobile-bottom-copy",
                            h3 {
                                class: "estimate-mobile-bottom-title",
                                "{mobile_step_title}"
                            }
                            input {
                                class: "estimate-mobile-bottom-scale-input",
                                r#type: "number",
                                step: "1",
                                min: "0",
                                placeholder: "distance (mm)",
                                value: "{scale_distance_input}",
                                oninput: move |evt| scale_distance_input.set(evt.value()),
                            }
                        }
                    } else {
                        div {
                            class: "estimate-mobile-bottom-copy",
                            h3 {
                                class: "estimate-mobile-bottom-title",
                                "{mobile_step_title}"
                            }
                            p {
                                class: "estimate-mobile-bottom-subtitle",
                                "{mobile_step_subtitle}"
                            }
                        }
                    }
                    if mobile_show_next {
                        button {
                            class: "estimate-mobile-bottom-next-btn",
                            onclick: move |_| {
                                let current = step();
                                if current == 1 && scale_points().len() == 2 {
                                    let Ok(distance_mm) = scale_distance_input().trim().parse::<f64>() else {
                                        return;
                                    };
                                    if distance_mm <= 0.0 {
                                        return;
                                    }
                                    let distance_m = distance_mm / 1000.0;
                                    if distance_m <= 0.0 {
                                        return;
                                    }

                                    let pts = scale_points();
                                    if pts.len() != 2 {
                                        return;
                                    }
                                    let Some(page_id) = active_scale_page_id() else {
                                        return;
                                    };

                                    let (x1, y1) = pts[0];
                                    let (x2, y2) = pts[1];
                                    let px_distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
                                    if px_distance <= 0.0 {
                                        return;
                                    }

                                    let submit = ScaleData {
                                        page_id: page_id.clone(),
                                        point1: (x1, y1),
                                        point2: (x2, y2),
                                        real_distance_m: distance_m,
                                        px_per_meter: px_distance / distance_m,
                                    };
                                    page_scales.write().insert(page_id.clone(), submit);

                                    let project_id_for_save = project_id_for_mobile_bar.clone();
                                    spawn(async move {
                                        let payload = SaveScaleRequest {
                                            reference_start: ScalePoint { x: x1, y: y1 },
                                            reference_end: ScalePoint { x: x2, y: y2 },
                                            real_distance_m: distance_m,
                                            page_id,
                                        };
                                        let save_url = format!("{}/public/projects/{}/scale", API_BASE_URL, project_id_for_save);
                                        if let Ok(request) = Request::post(&save_url)
                                            .header("Content-Type", "application/json")
                                            .json(&payload)
                                        {
                                            let _ = request.send().await;
                                        }
                                    });

                                    scale_points.set(Vec::new());
                                    active_scale_page_id.set(None);
                                    scale_distance_input.set(String::new());
                                }

                                if current < 6 {
                                    let next = current + 1;
                                    step.set(next);
                                    if (2..=5).contains(&next) {
                                        focus_floor_plan_request.set(focus_floor_plan_request() + 1);
                                    }
                                }
                            },
                            "→"
                        }
                    }
                }
            }


            if panel_open() {
                div {
                    class: "estimate-panel-divider",
                    onmousedown: move |evt| {
                        evt.prevent_default();
                        is_resizing_panel.set(true);
                    },
                }
            }

            div {
                class: "estimate-panel",
                style: "{panel_style}",

                h3 {
                    class: "estimate-panel-project-name",
                    "{project_name}"
                }
                p {
                    class: "estimate-panel-email",
                    "{project_email}"
                }

                div {
                    class: "estimate-panel-step-indicator",
                    "Step {step()} of 6"
                }

                h2 {
                    class: "estimate-panel-title",
                    "{step_title}"
                }

                if step() == 1 {
                    ScalePanelSection {
                        project_id: project_id.clone(),
                        page_scales: page_scales,
                        scale_points: scale_points,
                        active_scale_page_id: active_scale_page_id,
                        scale_distance_input: scale_distance_input,
                    }
                } else if (2..=5).contains(&step()) {
                    {
                        let step_now = step();
                        let scales_now = page_scales();
                        let footprints_now = footprints();
                        let ewalls_now = ewalls();
                        let iwalls_now = iwalls();
                        let cabinetry_now = cabinetry();

                        let saved_footprint_m2 = footprints_now
                            .iter()
                            .filter_map(|(page_id, footprint)| {
                                scales_now.get(page_id).and_then(|scale| {
                                    let px_per_meter = scale_px_per_meter_for_area(scale);
                                    if px_per_meter > 0.0 {
                                        Some(polygon_area_px(&footprint.points) / (px_per_meter * px_per_meter))
                                    } else {
                                        None
                                    }
                                })
                            })
                            .fold(0.0_f64, |sum, value| sum + value);
                        let saved_ewalls_m = compute_saved_ewalls_lm_m(&ewalls_now, &scales_now);
                        let saved_iwalls_m = total_iwalls_lm_m(&iwalls_now, &scales_now);
                        let saved_cabinetry_m = total_cabinetry_lm_m(&cabinetry_now, &scales_now);

                        let footprint_display_m2 = if step_now == 2 && !active_footprint_drawing().is_empty() {
                            live_footprint_area_m2().unwrap_or(saved_footprint_m2)
                        } else {
                            saved_footprint_m2
                        };
                        let ewalls_display_m = if step_now == 3 && !active_ewall_drawing().is_empty() {
                            live_ewalls_lm_m().unwrap_or(saved_ewalls_m)
                        } else {
                            saved_ewalls_m
                        };
                        let iwalls_display_m = if step_now == 4 && active_iwall_start_point().is_some() {
                            live_iwalls_lm_m().unwrap_or(saved_iwalls_m)
                        } else {
                            saved_iwalls_m
                        };
                        let cabinetry_display_m = if step_now == 5 && active_cabinetry_start_point().is_some() {
                            live_cabinetry_lm_m().unwrap_or(saved_cabinetry_m)
                        } else {
                            saved_cabinetry_m
                        };

                        let draw_hint = match step_now {
                            2 => "Trace the outline of the ground floor area.",
                            3 => "Trace external walls over the footprint.",
                            4 => "Add internal wall segments.",
                            5 => "Add cabinetry segments.",
                            _ => "",
                        };

                        rsx! {
                            p {
                                class: "estimate-panel-description",
                                "{draw_hint}"
                            }
                            div {
                                class: "estimate-measurement-brick-list",
                                if step_now >= 2 {
                                    div {
                                        class: "estimate-measurement-brick estimate-measurement-brick-footprint",
                                        div {
                                            class: "estimate-measurement-brick-left",
                                            span { class: "estimate-measurement-brick-swatch" }
                                            span { class: "estimate-measurement-brick-label", "Footprint" }
                                        }
                                        span { class: "estimate-measurement-brick-value", "{footprint_display_m2:.5} m2" }
                                    }
                                }
                                if step_now >= 3 {
                                    div {
                                        class: "estimate-measurement-brick estimate-measurement-brick-ewalls",
                                        div {
                                            class: "estimate-measurement-brick-left",
                                            span { class: "estimate-measurement-brick-swatch" }
                                            span { class: "estimate-measurement-brick-label", "External walls" }
                                        }
                                        span { class: "estimate-measurement-brick-value", "{ewalls_display_m:.5} m" }
                                    }
                                }
                                if step_now >= 4 {
                                    div {
                                        class: "estimate-measurement-brick estimate-measurement-brick-iwalls",
                                        div {
                                            class: "estimate-measurement-brick-left",
                                            span { class: "estimate-measurement-brick-swatch" }
                                            span { class: "estimate-measurement-brick-label", "Internal walls" }
                                        }
                                        span { class: "estimate-measurement-brick-value", "{iwalls_display_m:.5} m" }
                                    }
                                }
                                if step_now >= 5 {
                                    div {
                                        class: "estimate-measurement-brick estimate-measurement-brick-cabinetry",
                                        div {
                                            class: "estimate-measurement-brick-left",
                                            span { class: "estimate-measurement-brick-swatch" }
                                            span { class: "estimate-measurement-brick-label", "Cabinetry" }
                                        }
                                        span { class: "estimate-measurement-brick-value", "{cabinetry_display_m:.5} m" }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    p {
                        class: "estimate-panel-description",
                        "{step_description}"
                    }
                }

                if !(step() == 1 && scale_points().len() == 2) {
                    button {
                        class: "estimate-panel-btn estimate-panel-next-btn",
                        onclick: move |_| {
                            let current = step();
                            if current < 6 {
                                let next = current + 1;
                                step.set(next);
                                if (2..=5).contains(&next) {
                                    focus_floor_plan_request.set(focus_floor_plan_request() + 1);
                                }
                            }
                        },
                        if step() >= 6 { "Generate Budget" } else { "Next" }
                    }
                }
            }
        }
    }
}
