use crate::core::svg_canvas::crosshair_overlay::CrosshairOverlay;
use crate::core::{is_mobile, LoadingScreen, Theme, THEME};
use crate::public::estimate_cabinetry::{
    compute_live_cabinetry_lm_m, handle_cabinetry_click, CabinetryData,
};
use crate::public::estimate_ewalls::{compute_live_ewalls_lm_m, handle_ewalls_click, EWallsData};
use crate::public::estimate_footprint::{
    compute_preview_area_m2, first_floor_plan_page, handle_footprint_click, polygon_points_to_svg,
    save_footprint_geometry, FootprintData,
};
use crate::public::estimate_iwalls::{compute_live_iwalls_lm_m, handle_iwalls_click, IWallsData};
use crate::public::estimate_scale::{
    handle_scale_canvas_click, render_active_scale_drawing, render_saved_scale_line,
    render_scale_badge,
};
use crate::public::estimate_types::{
    canvas_content_bounds, detect_page_id_from_world_y, PageLayout, ScaleData, ESTIMATE_CANVAS_ID,
};
use dioxus::prelude::*;
use std::collections::HashMap;

const LOGO_LIGHT: Asset = asset!("/assets/icons/dog-light.svg");
const LOGO_DARK: Asset = asset!("/assets/icons/dog-dark.svg");

#[component]
pub fn EstimateCanvas(
    pages: Vec<PageLayout>,
    loading: bool,
    loading_message: String,
    error: Option<String>,
    scale_tool_active: bool,
    footprint_tool_active: bool,
    ewalls_tool_active: bool,
    iwalls_tool_active: bool,
    cabinetry_tool_active: bool,
    scales: HashMap<String, ScaleData>,
    footprints: Signal<HashMap<String, FootprintData>>,
    ewalls: Signal<HashMap<String, EWallsData>>,
    iwalls: Signal<HashMap<String, IWallsData>>,
    cabinetry: Signal<HashMap<String, CabinetryData>>,
    scale_points: Signal<Vec<(f64, f64)>>,
    active_scale_page_id: Signal<Option<String>>,
    scale_distance_input: Signal<String>,
    active_footprint_drawing: Signal<Vec<(f64, f64)>>,
    footprint_page_id: Signal<Option<String>>,
    live_footprint_area_m2: Signal<Option<f64>>,
    active_ewall_drawing: Signal<Vec<(f64, f64)>>,
    ewall_page_id: Signal<Option<String>>,
    live_ewalls_lm_m: Signal<Option<f64>>,
    active_iwall_start_point: Signal<Option<(f64, f64)>>,
    iwall_page_id: Signal<Option<String>>,
    live_iwalls_lm_m: Signal<Option<f64>>,
    active_cabinetry_start_point: Signal<Option<(f64, f64)>>,
    cabinetry_page_id: Signal<Option<String>>,
    live_cabinetry_lm_m: Signal<Option<f64>>,
    project_id: String,
    project_email: String,
    focus_floor_plan_request: u64,
) -> Element {
    let is_dark = THEME() == Theme::Dark;
    let logo = if is_dark { LOGO_DARK } else { LOGO_LIGHT };
    let dot_fill = if is_dark {
        "var(--estimate-dot-fill-dark, rgba(255, 255, 255, 0.15))"
    } else {
        "var(--estimate-dot-fill-light, rgb(180, 180, 180))"
    };
    let dot_r = "var(--estimate-dot-r, 3)";
    let footprint_stroke = if is_dark { "#60A5FA" } else { "#2563EB" };
    let ewalls_stroke = "#16A34A";
    let iwalls_stroke = "#DB2777";
    let cabinetry_stroke = "#CA8A04";
    let scale_text_stroke = if is_dark {
        "var(--bg-primary)".to_string()
    } else {
        "rgb(245,245,245)".to_string()
    };

    let mut pan_x = use_signal(|| 0.0_f64);
    let mut pan_y = use_signal(|| 0.0_f64);
    let mut zoom = use_signal(|| 1.0_f64);
    let mut did_auto_fit = use_signal(|| false);
    let mut svg_mounted = use_signal(|| false);
    let mut last_focus_request = use_signal(|| 0_u64);

    let mut cursor_pos = use_signal(|| (0.0_f64, 0.0_f64));
    let mut cursor_world = use_signal(|| (0.0_f64, 0.0_f64));
    let mut cursor_inside = use_signal(|| false);

    let mut is_dragging = use_signal(|| false);
    let mut is_panning = use_signal(|| false);
    let mut last_mouse = use_signal(|| (0.0_f64, 0.0_f64));
    let mut click_start = use_signal(|| (0.0_f64, 0.0_f64));

    let mut touch_pointer_id = use_signal(|| None::<i32>);
    let mut touch_drag_active = use_signal(|| false);
    let mut touch_points = use_signal(HashMap::<i32, (f64, f64)>::new);
    let mut touch_had_multi = use_signal(|| false);
    let mut pinch_start_distance = use_signal(|| None::<f64>);
    let mut pinch_start_zoom = use_signal(|| None::<f64>);
    let mut pinch_world_anchor = use_signal(|| None::<(f64, f64)>);
    let mut mobile_scale_hold_token = use_signal(|| 0_u64);
    let mut mobile_scale_hold_active = use_signal(|| false);
    let mut mobile_scale_magnifier_visible = use_signal(|| false);
    let mut mobile_scale_magnifier_world = use_signal(|| (0.0_f64, 0.0_f64));

    let mut scale_points = scale_points;
    let mut active_scale_page_id = active_scale_page_id;
    let mut scale_distance_input = scale_distance_input;
    let mut active_footprint_drawing = active_footprint_drawing;
    let mut footprint_page_id = footprint_page_id;
    let mut live_footprint_area_m2 = live_footprint_area_m2;
    let mut active_ewall_drawing = active_ewall_drawing;
    let mut ewall_page_id = ewall_page_id;
    let mut live_ewalls_lm_m = live_ewalls_lm_m;
    let mut active_iwall_start_point = active_iwall_start_point;
    let mut iwall_page_id = iwall_page_id;
    let mut live_iwalls_lm_m = live_iwalls_lm_m;
    let mut active_cabinetry_start_point = active_cabinetry_start_point;
    let mut cabinetry_page_id = cabinetry_page_id;
    let mut live_cabinetry_lm_m = live_cabinetry_lm_m;
    let mut footprints = footprints;
    let mut ewalls = ewalls;
    let mut iwalls = iwalls;
    let mut cabinetry = cabinetry;

    if svg_mounted() && !did_auto_fit() && !pages.is_empty() {
        let pages_for_fit = pages.clone();
        did_auto_fit.set(true);
        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(100).await;
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Some(svg_el) = document.get_element_by_id(ESTIMATE_CANVAS_ID) {
                            let rect = svg_el.get_bounding_client_rect();
                            let viewport_width = rect.width();
                            let (content_width, _content_height) = canvas_content_bounds(&pages_for_fit);
                            if viewport_width > 0.0 && content_width > 0.0 {
                                let width_zoom = viewport_width / content_width * 0.50;
                                let fit_zoom = width_zoom.clamp(0.05, 1.0);
                                zoom.set(fit_zoom);
                                pan_x.set((viewport_width - content_width * fit_zoom) / 2.0);
                                pan_y.set(20.0);
                            }
                        }
                    }
                }
            }
        });
    }

    if svg_mounted()
        && focus_floor_plan_request > 0
        && focus_floor_plan_request != last_focus_request()
        && !pages.is_empty()
    {
        let request_id = focus_floor_plan_request;
        last_focus_request.set(request_id);
        let pages_for_focus = pages.clone();

        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(30).await;
            let Some(target_page) = first_floor_plan_page(&pages_for_focus) else {
                return;
            };

            #[cfg(target_arch = "wasm32")]
            {
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Some(svg_el) = document.get_element_by_id(ESTIMATE_CANVAS_ID) {
                            let rect = svg_el.get_bounding_client_rect();
                            let viewport_width = rect.width();
                            let viewport_height = rect.height();
                            if viewport_width <= 0.0
                                || viewport_height <= 0.0
                                || target_page.display_width == 0
                            {
                                return;
                            }

                            let start_zoom = zoom();
                            let start_pan_x = pan_x();
                            let start_pan_y = pan_y();

                            let target_zoom =
                                (viewport_width * 0.90 / target_page.display_width as f64).clamp(0.1, 10.0);
                            let target_pan_x =
                                (viewport_width - target_page.display_width as f64 * target_zoom) / 2.0;
                            let page_center_y =
                                target_page.y_offset + target_page.display_height as f64 / 2.0;
                            let target_pan_y = viewport_height / 2.0 - page_center_y * target_zoom;

                            let steps = 20;
                            let step_ms = 16;
                            for i in 1..=steps {
                                let t = i as f64 / steps as f64;
                                let ease = 1.0 - (1.0 - t).powi(3);
                                zoom.set(start_zoom + (target_zoom - start_zoom) * ease);
                                pan_x.set(start_pan_x + (target_pan_x - start_pan_x) * ease);
                                pan_y.set(start_pan_y + (target_pan_y - start_pan_y) * ease);
                                gloo_timers::future::TimeoutFuture::new(step_ms).await;
                            }

                            zoom.set(target_zoom);
                            pan_x.set(target_pan_x);
                            pan_y.set(target_pan_y);
                        }
                    }
                }
            }
        });
    }

    use_effect(move || {
        if !scale_tool_active && (!scale_points().is_empty() || active_scale_page_id().is_some()) {
            scale_points.set(Vec::new());
            active_scale_page_id.set(None);
        }
        if !scale_tool_active || !is_mobile() {
            mobile_scale_hold_active.set(false);
            mobile_scale_magnifier_visible.set(false);
        }
    });

    use_effect(move || {
        if !footprint_tool_active
            && (!active_footprint_drawing().is_empty() || footprint_page_id().is_some())
        {
            active_footprint_drawing.set(Vec::new());
            footprint_page_id.set(None);
        }
    });

    use_effect(move || {
        if !ewalls_tool_active && (!active_ewall_drawing().is_empty() || ewall_page_id().is_some()) {
            active_ewall_drawing.set(Vec::new());
            ewall_page_id.set(None);
        }
    });

    use_effect(move || {
        if !iwalls_tool_active && (active_iwall_start_point().is_some() || iwall_page_id().is_some()) {
            active_iwall_start_point.set(None);
            iwall_page_id.set(None);
        }
    });

    use_effect(move || {
        if !cabinetry_tool_active
            && (active_cabinetry_start_point().is_some() || cabinetry_page_id().is_some())
        {
            active_cabinetry_start_point.set(None);
            cabinetry_page_id.set(None);
        }
    });
    let scales_for_live_preview = scales.clone();

    use_effect(move || {
        if !footprint_tool_active {
            live_footprint_area_m2.set(Some(0.0));
            return;
        }
        let area_m2 = compute_preview_area_m2(
            &active_footprint_drawing(),
            if cursor_inside() {
                Some(cursor_world())
            } else {
                None
            },
            footprint_page_id().as_deref(),
            &scales_for_live_preview,
        );

        live_footprint_area_m2.set(Some(area_m2));
    });

    let scales_for_ewalls_preview = scales.clone();
    use_effect(move || {
        if !ewalls_tool_active {
            live_ewalls_lm_m.set(Some(0.0));
            return;
        }
        let lm_m = compute_live_ewalls_lm_m(
            &active_ewall_drawing(),
            if cursor_inside() {
                Some(cursor_world())
            } else {
                None
            },
            ewall_page_id().as_deref(),
            &scales_for_ewalls_preview,
        );
        live_ewalls_lm_m.set(Some(lm_m));
    });

    let scales_for_iwalls_preview = scales.clone();
    use_effect(move || {
        if !iwalls_tool_active {
            live_iwalls_lm_m.set(Some(0.0));
            return;
        }
        let lm_m = compute_live_iwalls_lm_m(
            &iwalls(),
            active_iwall_start_point(),
            if cursor_inside() {
                Some(cursor_world())
            } else {
                None
            },
            iwall_page_id().as_deref(),
            &scales_for_iwalls_preview,
        );
        live_iwalls_lm_m.set(Some(lm_m));
    });

    let scales_for_cabinetry_preview = scales.clone();
    use_effect(move || {
        if !cabinetry_tool_active {
            live_cabinetry_lm_m.set(Some(0.0));
            return;
        }
        let lm_m = compute_live_cabinetry_lm_m(
            &cabinetry(),
            active_cabinetry_start_point(),
            if cursor_inside() {
                Some(cursor_world())
            } else {
                None
            },
            cabinetry_page_id().as_deref(),
            &scales_for_cabinetry_preview,
        );
        live_cabinetry_lm_m.set(Some(lm_m));
    });

    let points = scale_points();
    let footprint_points = active_footprint_drawing();
    let saved_scales: Vec<ScaleData> = scales.values().cloned().collect();
    let mut saved_footprints: Vec<FootprintData> = footprints().values().cloned().collect();
    let mut saved_ewalls: Vec<EWallsData> = ewalls().values().cloned().collect();
    let mut saved_iwalls: Vec<IWallsData> = iwalls().values().cloned().collect();
    let mut saved_cabinetry: Vec<CabinetryData> = cabinetry().values().cloned().collect();
    saved_footprints.sort_by_key(|item| item.page_id.parse::<usize>().unwrap_or(999));
    saved_ewalls.sort_by_key(|item| item.page_id.parse::<usize>().unwrap_or(999));
    saved_iwalls.sort_by_key(|item| item.page_id.parse::<usize>().unwrap_or(999));
    saved_cabinetry.sort_by_key(|item| item.page_id.parse::<usize>().unwrap_or(999));

    let pages_for_pointer_up = pages.clone();
    let pages_for_mouse_up = pages.clone();
    let project_id_for_pointer_up = project_id.clone();
    let project_email_for_pointer_up = project_email.clone();
    let project_id_for_mouse_up = project_id.clone();
    let project_email_for_mouse_up = project_email.clone();
    let scales_for_mouse_leave = scales.clone();
    let scales_for_pointer_move = scales.clone();
    let scales_for_mouse_move = scales.clone();
    let scales_for_pointer_move_2 = scales.clone();
    let scales_for_mouse_move_2 = scales.clone();

    let drawing_tool_active = scale_tool_active
        || footprint_tool_active
        || ewalls_tool_active
        || iwalls_tool_active
        || cabinetry_tool_active;
    let cursor_style = if is_panning() || (is_dragging() && !drawing_tool_active) {
        "cursor: grabbing;"
    } else if drawing_tool_active {
        "cursor: none;"
    } else {
        "cursor: grab;"
    };
    let overlay_stroke_w = (3.0 / zoom().max(0.0001).powf(1.35)).clamp(1.5, 10.0);
    let overlay_point_stroke_w = (2.2 / zoom().max(0.0001).powf(1.25)).clamp(1.0, 7.0);
    let overlay_point_r = (9.5 / zoom().max(0.0001).powf(1.20)).clamp(3.0, 15.0);
    let footprint_stroke_w = (2.8 / zoom().max(0.0001).powf(1.35)).clamp(1.2, 10.0);
    let footprint_point_stroke_w = (1.8 / zoom().max(0.0001).powf(1.35)).clamp(0.9, 8.0);
    let footprint_point_r = (8.5 / zoom().max(0.0001).powf(1.25)).clamp(2.8, 16.0);
    let overlay_dash = format!(
        "{:.3} {:.3}",
        (6.0 / zoom().max(0.0001).powf(1.15)).clamp(4.0, 22.0),
        (6.0 / zoom().max(0.0001).powf(1.15)).clamp(4.0, 22.0)
    );
    let mobile_scale_mode = scale_tool_active && is_mobile();
    let scale_magnifier_size = 160.0_f64;
    let scale_magnifier_half = scale_magnifier_size / 2.0;
    let scale_magnifier_zoom = 3.0_f64;
    let (scale_magnifier_wx, scale_magnifier_wy) = mobile_scale_magnifier_world();
    let scale_magnifier_tx = scale_magnifier_half - scale_magnifier_wx * scale_magnifier_zoom;
    let scale_magnifier_ty = scale_magnifier_half - scale_magnifier_wy * scale_magnifier_zoom;

    rsx! {
        div {
            class: "estimate-pages",

            if loading {
                LoadingScreen {
                    text: loading_message.clone()
                }
            } else if let Some(err) = &error {
                div {
                    class: "estimate-error",
                    img { class: "estimate-loading-logo", src: logo, alt: "Error" }
                    p { "{err}" }
                }
            } else {
                svg {
                    id: "{ESTIMATE_CANVAS_ID}",
                    width: "100%",
                    height: "100%",
                    style: "{cursor_style} touch-action: none; user-select: none; -webkit-user-select: none;",
                    onmounted: move |_| {
                        svg_mounted.set(true);
                    },

                    oncontextmenu: move |evt| {
                        if footprint_tool_active {
                            let mut pts = active_footprint_drawing.write();
                            if !pts.is_empty() {
                                evt.prevent_default();
                                evt.stop_propagation();
                                pts.pop();
                            }
                        } else if ewalls_tool_active {
                            let mut pts = active_ewall_drawing.write();
                            if !pts.is_empty() {
                                evt.prevent_default();
                                evt.stop_propagation();
                                pts.pop();
                            }
                        } else if iwalls_tool_active {
                            if active_iwall_start_point().is_some() {
                                evt.prevent_default();
                                evt.stop_propagation();
                                active_iwall_start_point.set(None);
                            }
                        } else if cabinetry_tool_active {
                            if active_cabinetry_start_point().is_some() {
                                evt.prevent_default();
                                evt.stop_propagation();
                                active_cabinetry_start_point.set(None);
                            }
                        }
                    },

                    onmouseenter: move |_| {
                        cursor_inside.set(true);
                    },
                    onmouseleave: move |_| {
                        cursor_inside.set(false);
                        is_dragging.set(false);
                        is_panning.set(false);
                        touch_pointer_id.set(None);
                        touch_drag_active.set(false);
                        touch_points.write().clear();
                        touch_had_multi.set(false);
                        pinch_start_distance.set(None);
                        pinch_start_zoom.set(None);
                        pinch_world_anchor.set(None);
                        mobile_scale_hold_token.set(mobile_scale_hold_token() + 1);
                        mobile_scale_hold_active.set(false);
                        mobile_scale_magnifier_visible.set(false);
                        if footprint_tool_active {
                            let area_m2 = compute_preview_area_m2(
                                &active_footprint_drawing(),
                                None,
                                footprint_page_id().as_deref(),
                                &scales_for_mouse_leave,
                            );
                            live_footprint_area_m2.set(Some(area_m2));
                        }
                        if ewalls_tool_active {
                            let lm_m = compute_live_ewalls_lm_m(
                                &active_ewall_drawing(),
                                None,
                                ewall_page_id().as_deref(),
                                &scales_for_mouse_leave,
                            );
                            live_ewalls_lm_m.set(Some(lm_m));
                        }
                        if iwalls_tool_active {
                            let lm_m = compute_live_iwalls_lm_m(
                                &iwalls(),
                                active_iwall_start_point(),
                                None,
                                iwall_page_id().as_deref(),
                                &scales_for_mouse_leave,
                            );
                            live_iwalls_lm_m.set(Some(lm_m));
                        }
                        if cabinetry_tool_active {
                            let lm_m = compute_live_cabinetry_lm_m(
                                &cabinetry(),
                                active_cabinetry_start_point(),
                                None,
                                cabinetry_page_id().as_deref(),
                                &scales_for_mouse_leave,
                            );
                            live_cabinetry_lm_m.set(Some(lm_m));
                        }
                    },

                    onpointerdown: move |evt| {
                        let pointer_type = format!("{:?}", evt.data.pointer_type()).to_ascii_lowercase();
                        if !pointer_type.contains("touch") {
                            return;
                        }
                        evt.prevent_default();
                        let pointer_id = evt.data.pointer_id() as i32;
                        let p = evt.element_coordinates();
                        let wx = (p.x - pan_x()) / zoom();
                        let wy = (p.y - pan_y()) / zoom();
                        cursor_pos.set((p.x, p.y));
                        cursor_world.set((wx, wy));
                        cursor_inside.set(true);
                        if mobile_scale_mode {
                            mobile_scale_magnifier_world.set((wx, wy));
                            mobile_scale_hold_active.set(false);
                            mobile_scale_magnifier_visible.set(false);
                        }

                        let (touch_count, pinch_pair) = {
                            let mut pts = touch_points.write();
                            pts.insert(pointer_id, (p.x, p.y));
                            let touch_count = pts.len();
                            let pinch_pair = if touch_count >= 2 {
                                let mut iter = pts.values();
                                if let (Some(&(ax, ay)), Some(&(bx, by))) = (iter.next(), iter.next()) {
                                    Some((ax, ay, bx, by))
                                } else {
                                    None
                                }
                            } else {
                                None
                            };
                            (touch_count, pinch_pair)
                        };

                        if touch_count >= 2 {
                            touch_had_multi.set(true);
                            touch_pointer_id.set(None);
                            touch_drag_active.set(false);
                            is_dragging.set(false);
                            is_panning.set(true);
                            mobile_scale_hold_token.set(mobile_scale_hold_token() + 1);
                            mobile_scale_hold_active.set(false);
                            mobile_scale_magnifier_visible.set(false);
                            if let Some((ax, ay, bx, by)) = pinch_pair {
                                let center_x = (ax + bx) / 2.0;
                                let center_y = (ay + by) / 2.0;
                                let dist = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt().max(1.0);
                                pinch_start_distance.set(Some(dist));
                                pinch_start_zoom.set(Some(zoom()));
                                pinch_world_anchor.set(Some((
                                    (center_x - pan_x()) / zoom().max(0.0001),
                                    (center_y - pan_y()) / zoom().max(0.0001),
                                )));
                                last_mouse.set((center_x, center_y));
                            }
                        } else {
                            touch_pointer_id.set(Some(pointer_id));
                            touch_drag_active.set(true);
                            is_dragging.set(true);
                            is_panning.set(false);
                            click_start.set((p.x, p.y));
                            last_mouse.set((p.x, p.y));
                            if mobile_scale_mode {
                                let hold_token = mobile_scale_hold_token() + 1;
                                mobile_scale_hold_token.set(hold_token);
                                spawn(async move {
                                    gloo_timers::future::TimeoutFuture::new(420).await;
                                    if mobile_scale_hold_token() == hold_token
                                        && touch_pointer_id() == Some(pointer_id)
                                        && touch_drag_active()
                                        && mobile_scale_mode
                                        && !is_panning()
                                        && !touch_had_multi()
                                    {
                                        mobile_scale_hold_active.set(true);
                                        mobile_scale_magnifier_visible.set(true);
                                    }
                                });
                            }
                        }

                        if footprint_tool_active {
                            let area_m2 = compute_preview_area_m2(
                                &active_footprint_drawing(),
                                Some((wx, wy)),
                                footprint_page_id().as_deref(),
                                &scales_for_pointer_move,
                            );
                            live_footprint_area_m2.set(Some(area_m2));
                        }
                        if ewalls_tool_active {
                            let lm_m = compute_live_ewalls_lm_m(
                                &active_ewall_drawing(),
                                Some((wx, wy)),
                                ewall_page_id().as_deref(),
                                &scales_for_pointer_move,
                            );
                            live_ewalls_lm_m.set(Some(lm_m));
                        }
                        if iwalls_tool_active {
                            let lm_m = compute_live_iwalls_lm_m(
                                &iwalls(),
                                active_iwall_start_point(),
                                Some((wx, wy)),
                                iwall_page_id().as_deref(),
                                &scales_for_pointer_move,
                            );
                            live_iwalls_lm_m.set(Some(lm_m));
                        }
                        if cabinetry_tool_active {
                            let lm_m = compute_live_cabinetry_lm_m(
                                &cabinetry(),
                                active_cabinetry_start_point(),
                                Some((wx, wy)),
                                cabinetry_page_id().as_deref(),
                                &scales_for_pointer_move,
                            );
                            live_cabinetry_lm_m.set(Some(lm_m));
                        }
                    },
                    onpointermove: move |evt| {
                        let pointer_type = format!("{:?}", evt.data.pointer_type()).to_ascii_lowercase();
                        if !pointer_type.contains("touch") {
                            return;
                        }
                        let pointer_id = evt.data.pointer_id() as i32;
                        let p = evt.element_coordinates();
                        let (is_known_touch, touch_count, pinch_pair) = {
                            let mut pts = touch_points.write();
                            if !pts.contains_key(&pointer_id) {
                                (false, pts.len(), None)
                            } else {
                                pts.insert(pointer_id, (p.x, p.y));
                                let touch_count = pts.len();
                                let pinch_pair = if touch_count >= 2 {
                                    let mut iter = pts.values();
                                    if let (Some(&(ax, ay)), Some(&(bx, by))) = (iter.next(), iter.next()) {
                                        Some((ax, ay, bx, by))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };
                                (true, touch_count, pinch_pair)
                            }
                        };
                        if !is_known_touch {
                            return;
                        }

                        let wx = (p.x - pan_x()) / zoom();
                        let wy = (p.y - pan_y()) / zoom();
                        cursor_pos.set((p.x, p.y));
                        cursor_world.set((wx, wy));
                        if mobile_scale_mode && touch_pointer_id() == Some(pointer_id) {
                            mobile_scale_magnifier_world.set((wx, wy));
                        }

                        if touch_count >= 2 {
                            touch_had_multi.set(true);
                            touch_pointer_id.set(None);
                            touch_drag_active.set(false);
                            is_dragging.set(false);
                            is_panning.set(true);
                            mobile_scale_hold_token.set(mobile_scale_hold_token() + 1);
                            mobile_scale_hold_active.set(false);
                            mobile_scale_magnifier_visible.set(false);

                            if let Some((ax, ay, bx, by)) = pinch_pair {
                                if pinch_start_distance().is_none()
                                    || pinch_start_zoom().is_none()
                                    || pinch_world_anchor().is_none()
                                {
                                    let center_x = (ax + bx) / 2.0;
                                    let center_y = (ay + by) / 2.0;
                                    let dist = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt().max(1.0);
                                    pinch_start_distance.set(Some(dist));
                                    pinch_start_zoom.set(Some(zoom()));
                                    pinch_world_anchor.set(Some((
                                        (center_x - pan_x()) / zoom().max(0.0001),
                                        (center_y - pan_y()) / zoom().max(0.0001),
                                    )));
                                }

                                if let (Some(start_dist), Some(start_zoom), Some((anchor_x, anchor_y))) =
                                    (pinch_start_distance(), pinch_start_zoom(), pinch_world_anchor())
                                {
                                    let center_x = (ax + bx) / 2.0;
                                    let center_y = (ay + by) / 2.0;
                                    let dist = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt().max(1.0);
                                    let zoom_factor = dist / start_dist.max(1.0);
                                    let new_zoom = (start_zoom * zoom_factor).clamp(0.1, 10.0);
                                    let new_pan_x = center_x - anchor_x * new_zoom;
                                    let new_pan_y = center_y - anchor_y * new_zoom;
                                    zoom.set(new_zoom);
                                    pan_x.set(new_pan_x);
                                    pan_y.set(new_pan_y);
                                    last_mouse.set((center_x, center_y));
                                }
                            }
                            return;
                        }

                        if touch_pointer_id() != Some(pointer_id) || !touch_drag_active() {
                            touch_pointer_id.set(Some(pointer_id));
                            touch_drag_active.set(true);
                            click_start.set((p.x, p.y));
                            last_mouse.set((p.x, p.y));
                            is_dragging.set(true);
                            is_panning.set(false);
                            return;
                        }

                        if mobile_scale_mode && mobile_scale_hold_active() {
                            is_panning.set(false);
                            return;
                        }

                        if footprint_tool_active {
                            let area_m2 = compute_preview_area_m2(
                                &active_footprint_drawing(),
                                Some((wx, wy)),
                                footprint_page_id().as_deref(),
                                &scales_for_pointer_move_2,
                            );
                            live_footprint_area_m2.set(Some(area_m2));
                        }
                        if ewalls_tool_active {
                            let lm_m = compute_live_ewalls_lm_m(
                                &active_ewall_drawing(),
                                Some((wx, wy)),
                                ewall_page_id().as_deref(),
                                &scales_for_pointer_move_2,
                            );
                            live_ewalls_lm_m.set(Some(lm_m));
                        }
                        if iwalls_tool_active {
                            let lm_m = compute_live_iwalls_lm_m(
                                &iwalls(),
                                active_iwall_start_point(),
                                Some((wx, wy)),
                                iwall_page_id().as_deref(),
                                &scales_for_pointer_move_2,
                            );
                            live_iwalls_lm_m.set(Some(lm_m));
                        }
                        if cabinetry_tool_active {
                            let lm_m = compute_live_cabinetry_lm_m(
                                &cabinetry(),
                                active_cabinetry_start_point(),
                                Some((wx, wy)),
                                cabinetry_page_id().as_deref(),
                                &scales_for_pointer_move_2,
                            );
                            live_cabinetry_lm_m.set(Some(lm_m));
                        }

                        let (sx, sy) = click_start();
                        let travel = ((p.x - sx).powi(2) + (p.y - sy).powi(2)).sqrt();
                        if travel > 12.0 {
                            if mobile_scale_mode {
                                mobile_scale_hold_token.set(mobile_scale_hold_token() + 1);
                                mobile_scale_hold_active.set(false);
                                mobile_scale_magnifier_visible.set(false);
                            }
                            is_panning.set(true);
                        }
                        if is_panning() {
                            let (lx, ly) = last_mouse();
                            pan_x.set(pan_x() + (p.x - lx));
                            pan_y.set(pan_y() + (p.y - ly));
                            last_mouse.set((p.x, p.y));
                        }
                    },
                    onpointerup: move |evt| {
                        let pointer_type = format!("{:?}", evt.data.pointer_type()).to_ascii_lowercase();
                        if !pointer_type.contains("touch") {
                            return;
                        }
                        let pointer_id = evt.data.pointer_id() as i32;
                        let p = evt.element_coordinates();
                        let wx = (p.x - pan_x()) / zoom();
                        let wy = (p.y - pan_y()) / zoom();
                        cursor_pos.set((p.x, p.y));
                        cursor_world.set((wx, wy));

                        let (remaining_count, remaining_touch) = {
                            let mut pts = touch_points.write();
                            pts.remove(&pointer_id);
                            let remaining_count = pts.len();
                            let remaining_touch = if remaining_count == 1 {
                                pts.iter().next().map(|(id, point)| (*id, *point))
                            } else {
                                None
                            };
                            (remaining_count, remaining_touch)
                        };

                        let is_click_candidate =
                            touch_pointer_id() == Some(pointer_id) && touch_drag_active();
                        let had_multi = touch_had_multi();
                        let (sx, sy) = click_start();
                        let travel = ((p.x - sx).powi(2) + (p.y - sy).powi(2)).sqrt();
                        let scale_hold_was_active = mobile_scale_mode && mobile_scale_hold_active();
                        let (click_wx, click_wy) = if scale_hold_was_active {
                            mobile_scale_magnifier_world()
                        } else {
                            (wx, wy)
                        };
                        let was_pan = if scale_hold_was_active {
                            is_panning() || had_multi
                        } else {
                            travel > 12.0 || is_panning() || had_multi
                        };

                        if remaining_count >= 2 {
                            if let Some((ax, ay, bx, by)) = {
                                let pts = touch_points.read();
                                let mut iter = pts.values();
                                if let (Some(&(ax, ay)), Some(&(bx, by))) = (iter.next(), iter.next()) {
                                    Some((ax, ay, bx, by))
                                } else {
                                    None
                                }
                            } {
                                let center_x = (ax + bx) / 2.0;
                                let center_y = (ay + by) / 2.0;
                                let dist = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt().max(1.0);
                                pinch_start_distance.set(Some(dist));
                                pinch_start_zoom.set(Some(zoom()));
                                pinch_world_anchor.set(Some((
                                    (center_x - pan_x()) / zoom().max(0.0001),
                                    (center_y - pan_y()) / zoom().max(0.0001),
                                )));
                                last_mouse.set((center_x, center_y));
                            }
                            touch_pointer_id.set(None);
                            touch_drag_active.set(false);
                            is_dragging.set(false);
                            is_panning.set(true);
                            mobile_scale_hold_token.set(mobile_scale_hold_token() + 1);
                            mobile_scale_hold_active.set(false);
                            mobile_scale_magnifier_visible.set(false);
                            return;
                        }

                        if let Some((next_id, (nx, ny))) = remaining_touch {
                            touch_pointer_id.set(Some(next_id));
                            touch_drag_active.set(true);
                            is_dragging.set(true);
                            is_panning.set(false);
                            click_start.set((nx, ny));
                            last_mouse.set((nx, ny));
                            pinch_start_distance.set(None);
                            pinch_start_zoom.set(None);
                            pinch_world_anchor.set(None);
                            if mobile_scale_mode {
                                mobile_scale_magnifier_world.set((
                                    (nx - pan_x()) / zoom().max(0.0001),
                                    (ny - pan_y()) / zoom().max(0.0001),
                                ));
                            }
                            return;
                        }

                        touch_pointer_id.set(None);
                        touch_drag_active.set(false);
                        touch_points.write().clear();
                        touch_had_multi.set(false);
                        pinch_start_distance.set(None);
                        pinch_start_zoom.set(None);
                        pinch_world_anchor.set(None);
                        mobile_scale_hold_token.set(mobile_scale_hold_token() + 1);
                        mobile_scale_hold_active.set(false);
                        mobile_scale_magnifier_visible.set(false);
                        is_dragging.set(false);
                        is_panning.set(false);

                        if !is_click_candidate || was_pan {
                            return;
                        }

                        if scale_tool_active {
                            let Some(clicked_page_id) = detect_page_id_from_world_y(&pages_for_pointer_up, click_wy) else {
                                return;
                            };
                            if mobile_scale_mode && scale_hold_was_active {
                                let current_points = scale_points();
                                if current_points.len() == 2
                                    && active_scale_page_id().as_deref() == Some(clicked_page_id.as_str())
                                {
                                    scale_points.set(vec![current_points[0], (click_wx, click_wy)]);
                                    return;
                                }
                            }
                            handle_scale_canvas_click(
                                clicked_page_id,
                                (click_wx, click_wy),
                                scale_points,
                                active_scale_page_id,
                                scale_distance_input,
                            );
                            return;
                        }

                        if footprint_tool_active {
                            let result = handle_footprint_click(
                                &pages_for_pointer_up,
                                &active_footprint_drawing(),
                                footprint_page_id().as_deref(),
                                (wx, wy),
                                zoom(),
                            );
                            active_footprint_drawing.set(result.next_points);
                            footprint_page_id.set(result.next_page_id);

                            if let Some((page_id, closed_points)) = result.closed_polygon {
                                footprints.write().insert(
                                    page_id.clone(),
                                    FootprintData {
                                        page_id: page_id.clone(),
                                        points: closed_points.clone(),
                                    },
                                );

                                let project_id_for_save = project_id_for_pointer_up.clone();
                                let project_email_for_save = project_email_for_pointer_up.clone();
                                spawn(async move {
                                    save_footprint_geometry(
                                        project_id_for_save,
                                        project_email_for_save,
                                        page_id,
                                        closed_points,
                                    )
                                    .await;
                                });
                            }
                            return;
                        }

                        if ewalls_tool_active {
                            let result = handle_ewalls_click(
                                &pages_for_pointer_up,
                                &active_ewall_drawing(),
                                ewall_page_id().as_deref(),
                                (wx, wy),
                                zoom(),
                            );
                            active_ewall_drawing.set(result.next_points);
                            ewall_page_id.set(result.next_page_id);
                            if let Some((page_id, closed_points)) = result.closed_path {
                                ewalls.write().insert(
                                    page_id.clone(),
                                    EWallsData {
                                        page_id,
                                        points: closed_points,
                                        closed: true,
                                    },
                                );
                            }
                            return;
                        }

                        if iwalls_tool_active {
                            let result = handle_iwalls_click(
                                &pages_for_pointer_up,
                                active_iwall_start_point(),
                                iwall_page_id().as_deref(),
                                (wx, wy),
                            );
                            active_iwall_start_point.set(result.next_start_point);
                            iwall_page_id.set(result.next_page_id);
                            if let Some((page_id, segment)) = result.new_segment {
                                let mut all_iwalls = iwalls.write();
                                let entry = all_iwalls.entry(page_id.clone()).or_insert(IWallsData {
                                    page_id: page_id.clone(),
                                    segments: Vec::new(),
                                });
                                entry.segments.push(segment);
                            }
                            return;
                        }

                        if cabinetry_tool_active {
                            let result = handle_cabinetry_click(
                                &pages_for_pointer_up,
                                active_cabinetry_start_point(),
                                cabinetry_page_id().as_deref(),
                                (wx, wy),
                            );
                            active_cabinetry_start_point.set(result.next_start_point);
                            cabinetry_page_id.set(result.next_page_id);
                            if let Some((page_id, segment)) = result.new_segment {
                                let mut all_cabinetry = cabinetry.write();
                                let entry = all_cabinetry.entry(page_id.clone()).or_insert(CabinetryData {
                                    page_id: page_id.clone(),
                                    segments: Vec::new(),
                                });
                                entry.segments.push(segment);
                            }
                        }
                    },
                    onpointercancel: move |_| {
                        touch_pointer_id.set(None);
                        touch_drag_active.set(false);
                        touch_points.write().clear();
                        touch_had_multi.set(false);
                        pinch_start_distance.set(None);
                        pinch_start_zoom.set(None);
                        pinch_world_anchor.set(None);
                        mobile_scale_hold_token.set(mobile_scale_hold_token() + 1);
                        mobile_scale_hold_active.set(false);
                        mobile_scale_magnifier_visible.set(false);
                        is_dragging.set(false);
                        is_panning.set(false);
                    },

                    onmousedown: move |evt| {
                        let button = format!("{:?}", evt.data.trigger_button());
                        if !button.contains("Primary") {
                            return;
                        }
                        let p = evt.element_coordinates();
                        let wx = (p.x - pan_x()) / zoom();
                        let wy = (p.y - pan_y()) / zoom();
                        cursor_pos.set((p.x, p.y));
                        cursor_world.set((wx, wy));
                        click_start.set((p.x, p.y));
                        last_mouse.set((p.x, p.y));
                        is_dragging.set(true);
                        is_panning.set(false);
                        cursor_inside.set(true);
                    },
                    onmousemove: move |evt| {
                        let p = evt.element_coordinates();
                        let wx = (p.x - pan_x()) / zoom();
                        let wy = (p.y - pan_y()) / zoom();
                        cursor_pos.set((p.x, p.y));
                        cursor_world.set((wx, wy));
                        if footprint_tool_active {
                            let area_m2 = compute_preview_area_m2(
                                &active_footprint_drawing(),
                                Some((wx, wy)),
                                footprint_page_id().as_deref(),
                                &scales_for_mouse_move,
                            );
                            live_footprint_area_m2.set(Some(area_m2));
                        }
                        if ewalls_tool_active {
                            let lm_m = compute_live_ewalls_lm_m(
                                &active_ewall_drawing(),
                                Some((wx, wy)),
                                ewall_page_id().as_deref(),
                                &scales_for_mouse_move_2,
                            );
                            live_ewalls_lm_m.set(Some(lm_m));
                        }
                        if iwalls_tool_active {
                            let lm_m = compute_live_iwalls_lm_m(
                                &iwalls(),
                                active_iwall_start_point(),
                                Some((wx, wy)),
                                iwall_page_id().as_deref(),
                                &scales_for_mouse_move_2,
                            );
                            live_iwalls_lm_m.set(Some(lm_m));
                        }
                        if cabinetry_tool_active {
                            let lm_m = compute_live_cabinetry_lm_m(
                                &cabinetry(),
                                active_cabinetry_start_point(),
                                Some((wx, wy)),
                                cabinetry_page_id().as_deref(),
                                &scales_for_mouse_move_2,
                            );
                            live_cabinetry_lm_m.set(Some(lm_m));
                        }

                        if !is_dragging() {
                            return;
                        }

                        let (sx, sy) = click_start();
                        let travel = ((p.x - sx).powi(2) + (p.y - sy).powi(2)).sqrt();
                        if travel > 5.0 {
                            is_panning.set(true);
                        }
                        if is_panning() {
                            let (lx, ly) = last_mouse();
                            pan_x.set(pan_x() + (p.x - lx));
                            pan_y.set(pan_y() + (p.y - ly));
                            last_mouse.set((p.x, p.y));
                        }
                    },
                    onmouseup: move |evt| {
                        if !is_dragging() {
                            return;
                        }
                        let button = format!("{:?}", evt.data.trigger_button());
                        if !button.contains("Primary") {
                            is_dragging.set(false);
                            is_panning.set(false);
                            return;
                        }

                        let p = evt.element_coordinates();
                        let wx = (p.x - pan_x()) / zoom();
                        let wy = (p.y - pan_y()) / zoom();
                        cursor_pos.set((p.x, p.y));
                        cursor_world.set((wx, wy));

                        let (sx, sy) = click_start();
                        let travel = ((p.x - sx).powi(2) + (p.y - sy).powi(2)).sqrt();
                        let was_pan = travel > 5.0 || is_panning();

                        is_dragging.set(false);
                        is_panning.set(false);

                        if was_pan {
                            return;
                        }

                        if scale_tool_active {
                            let Some(clicked_page_id) = detect_page_id_from_world_y(&pages_for_mouse_up, wy) else {
                                return;
                            };
                            handle_scale_canvas_click(
                                clicked_page_id,
                                (wx, wy),
                                scale_points,
                                active_scale_page_id,
                                scale_distance_input,
                            );
                            return;
                        }

                        if footprint_tool_active {
                            let result = handle_footprint_click(
                                &pages_for_mouse_up,
                                &active_footprint_drawing(),
                                footprint_page_id().as_deref(),
                                (wx, wy),
                                zoom(),
                            );
                            active_footprint_drawing.set(result.next_points);
                            footprint_page_id.set(result.next_page_id);

                            if let Some((page_id, closed_points)) = result.closed_polygon {
                                footprints.write().insert(
                                    page_id.clone(),
                                    FootprintData {
                                        page_id: page_id.clone(),
                                        points: closed_points.clone(),
                                    },
                                );

                                let project_id_for_save = project_id_for_mouse_up.clone();
                                let project_email_for_save = project_email_for_mouse_up.clone();
                                spawn(async move {
                                    save_footprint_geometry(
                                        project_id_for_save,
                                        project_email_for_save,
                                        page_id,
                                        closed_points,
                                    )
                                    .await;
                                });
                            }
                            return;
                        }

                        if ewalls_tool_active {
                            let result = handle_ewalls_click(
                                &pages_for_mouse_up,
                                &active_ewall_drawing(),
                                ewall_page_id().as_deref(),
                                (wx, wy),
                                zoom(),
                            );
                            active_ewall_drawing.set(result.next_points);
                            ewall_page_id.set(result.next_page_id);
                            if let Some((page_id, closed_points)) = result.closed_path {
                                ewalls.write().insert(
                                    page_id.clone(),
                                    EWallsData {
                                        page_id,
                                        points: closed_points,
                                        closed: true,
                                    },
                                );
                            }
                            return;
                        }

                        if iwalls_tool_active {
                            let result = handle_iwalls_click(
                                &pages_for_mouse_up,
                                active_iwall_start_point(),
                                iwall_page_id().as_deref(),
                                (wx, wy),
                            );
                            active_iwall_start_point.set(result.next_start_point);
                            iwall_page_id.set(result.next_page_id);
                            if let Some((page_id, segment)) = result.new_segment {
                                let mut all_iwalls = iwalls.write();
                                let entry = all_iwalls.entry(page_id.clone()).or_insert(IWallsData {
                                    page_id: page_id.clone(),
                                    segments: Vec::new(),
                                });
                                entry.segments.push(segment);
                            }
                            return;
                        }

                        if cabinetry_tool_active {
                            let result = handle_cabinetry_click(
                                &pages_for_mouse_up,
                                active_cabinetry_start_point(),
                                cabinetry_page_id().as_deref(),
                                (wx, wy),
                            );
                            active_cabinetry_start_point.set(result.next_start_point);
                            cabinetry_page_id.set(result.next_page_id);
                            if let Some((page_id, segment)) = result.new_segment {
                                let mut all_cabinetry = cabinetry.write();
                                let entry = all_cabinetry.entry(page_id.clone()).or_insert(CabinetryData {
                                    page_id: page_id.clone(),
                                    segments: Vec::new(),
                                });
                                entry.segments.push(segment);
                            }
                        }
                    },

                    onwheel: move |evt| {
                        evt.prevent_default();
                        let dy = evt.delta().strip_units().y;
                        let factor = (-dy * 0.009).exp().clamp(0.7, 1.4);
                        let old_zoom = zoom();
                        let new_zoom = (old_zoom * factor).clamp(0.1, 10.0);
                        let p = evt.element_coordinates();
                        let new_pan_x = p.x - (p.x - pan_x()) * (new_zoom / old_zoom);
                        let new_pan_y = p.y - (p.y - pan_y()) * (new_zoom / old_zoom);
                        pan_x.set(new_pan_x);
                        pan_y.set(new_pan_y);
                        zoom.set(new_zoom);
                    },

                    defs {
                        pattern {
                            id: "estimate-dots",
                            width: "80",
                            height: "80",
                            pattern_units: "userSpaceOnUse",
                            circle {
                                cx: "40",
                                cy: "40",
                                r: "{dot_r}",
                                fill: "{dot_fill}",
                            }
                        }
                    }

                    rect {
                        class: "canvas-bg",
                        width: "100%",
                        height: "100%",
                        fill: "rgb(245,245,245)",
                    }

                    g {
                        transform: "translate({pan_x()}, {pan_y()}) scale({zoom()})",

                        rect {
                            x: "-10000",
                            y: "-10000",
                            width: "20000",
                            height: "20000",
                            fill: "url(#estimate-dots)",
                            pointer_events: "none",
                        }

                        for page in pages.iter() {
                            r#image {
                                key: "{page.page_num}",
                                class: "estimate-canvas-image",
                                href: "{page.image_url}",
                                x: "0",
                                y: "{page.y_offset}",
                                width: "{page.display_width}",
                                height: "{page.display_height}",
                                preserve_aspect_ratio: "xMinYMin meet",
                                pointer_events: "none",
                            }
                            {
                                let pid = page.page_num.to_string();
                                if let Some(scale) = scales.get(&pid) {
                                    render_scale_badge(scale.clone(), page.y_offset)
                                } else {
                                    rsx! {}
                                }
                            }
                        }

                        for scale in saved_scales.iter() {
                            { render_saved_scale_line(scale.clone(), scale_text_stroke.clone()) }
                        }

                        { render_active_scale_drawing(points.clone()) }

                        if footprint_tool_active {
                            for footprint in saved_footprints.iter() {
                                {
                                    let svg_points = polygon_points_to_svg(&footprint.points);
                                    rsx! {
                                        polygon {
                                            points: "{svg_points}",
                                            fill: "none",
                                            stroke: "{footprint_stroke}",
                                            stroke_width: "{footprint_stroke_w}",
                                            pointer_events: "none",
                                        }
                                        for (x, y) in footprint.points.iter() {
                                            circle {
                                                cx: "{x}",
                                                cy: "{y}",
                                                r: "{footprint_point_r}",
                                                fill: "{footprint_stroke}",
                                                stroke: "{footprint_stroke}",
                                                stroke_width: "{footprint_point_stroke_w}",
                                                pointer_events: "none",
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if ewalls_tool_active {
                            for ewall in saved_ewalls.iter() {
                                {
                                    let svg_points = polygon_points_to_svg(&ewall.points);
                                    if ewall.closed {
                                        rsx! {
                                            polygon {
                                                points: "{svg_points}",
                                                fill: "none",
                                                stroke: "{ewalls_stroke}",
                                                stroke_width: "{overlay_stroke_w}",
                                                pointer_events: "none",
                                            }
                                        }
                                    } else {
                                        rsx! {
                                            polyline {
                                                points: "{svg_points}",
                                                fill: "none",
                                                stroke: "{ewalls_stroke}",
                                                stroke_width: "{overlay_stroke_w}",
                                                pointer_events: "none",
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if iwalls_tool_active {
                            for iwall in saved_iwalls.iter() {
                                for segment in iwall.segments.iter() {
                                    line {
                                        x1: "{segment.0.0}",
                                        y1: "{segment.0.1}",
                                        x2: "{segment.1.0}",
                                        y2: "{segment.1.1}",
                                        stroke: "{iwalls_stroke}",
                                        stroke_width: "{overlay_stroke_w}",
                                        pointer_events: "none",
                                    }
                                }
                            }
                        }

                        if cabinetry_tool_active {
                            for cabinet in saved_cabinetry.iter() {
                                for segment in cabinet.segments.iter() {
                                    line {
                                        x1: "{segment.0.0}",
                                        y1: "{segment.0.1}",
                                        x2: "{segment.1.0}",
                                        y2: "{segment.1.1}",
                                        stroke: "{cabinetry_stroke}",
                                        stroke_width: "{overlay_stroke_w}",
                                        pointer_events: "none",
                                    }
                                }
                            }
                        }

                        if !active_ewall_drawing().is_empty() {
                            {
                                let points = active_ewall_drawing();
                                let outline = polygon_points_to_svg(&points);
                                let cursor_world_point = cursor_world();
                                let line_tail = points.last().copied();
                                let close_hint = if points.len() >= 3 && cursor_inside() {
                                    if let Some(&(fx, fy)) = points.first() {
                                        let snap_threshold = 15.0 / zoom().max(0.0001);
                                        let dist =
                                            ((cursor_world_point.0 - fx).powi(2) + (cursor_world_point.1 - fy).powi(2)).sqrt();
                                        Some((fx, fy, dist < snap_threshold))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };
                                rsx! {
                                    polyline {
                                        points: "{outline}",
                                        fill: "none",
                                        stroke: "{ewalls_stroke}",
                                        stroke_width: "{overlay_stroke_w}",
                                        pointer_events: "none",
                                    }
                                    for (idx, (px, py)) in points.iter().enumerate() {
                                        {
                                            let marker_r = if idx == 0 { overlay_point_r * 1.15 } else { overlay_point_r };
                                            let marker_fill = if idx == 0 { "rgba(22,163,74,0.35)" } else { "none" };
                                            let marker_stroke_w = if idx == 0 {
                                                overlay_point_stroke_w * 1.30
                                            } else {
                                                overlay_point_stroke_w
                                            };
                                            rsx! {
                                                circle {
                                                    cx: "{px}",
                                                    cy: "{py}",
                                                    r: "{marker_r}",
                                                    fill: "{marker_fill}",
                                                    stroke: "{ewalls_stroke}",
                                                    stroke_width: "{marker_stroke_w}",
                                                    pointer_events: "none",
                                                }
                                            }
                                        }
                                    }
                                    if let Some((fx, fy, show)) = close_hint {
                                        if show {
                                            circle {
                                                cx: "{fx}",
                                                cy: "{fy}",
                                                r: "{18.0 / zoom().max(0.0001)}",
                                                fill: "rgba(34,197,94,0.35)",
                                                stroke: "#22C55E",
                                                stroke_width: "{overlay_stroke_w}",
                                                pointer_events: "none",
                                            }
                                        }
                                    }
                                    if let Some((lx, ly)) = line_tail {
                                        if cursor_inside() {
                                            line {
                                                x1: "{lx}",
                                                y1: "{ly}",
                                                x2: "{cursor_world_point.0}",
                                                y2: "{cursor_world_point.1}",
                                                stroke: "{ewalls_stroke}",
                                                stroke_width: "{overlay_stroke_w}",
                                                stroke_dasharray: "{overlay_dash}",
                                                pointer_events: "none",
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if let Some((sx, sy)) = active_iwall_start_point() {
                            circle {
                                cx: "{sx}",
                                cy: "{sy}",
                                r: "{overlay_point_r}",
                                fill: "rgba(219,39,119,0.30)",
                                stroke: "{iwalls_stroke}",
                                stroke_width: "{overlay_point_stroke_w}",
                                pointer_events: "none",
                            }
                            if cursor_inside() {
                                {
                                    let cw = cursor_world();
                                    rsx! {
                                        line {
                                            x1: "{sx}",
                                            y1: "{sy}",
                                            x2: "{cw.0}",
                                            y2: "{cw.1}",
                                            stroke: "{iwalls_stroke}",
                                            stroke_width: "{overlay_stroke_w}",
                                            stroke_dasharray: "{overlay_dash}",
                                            pointer_events: "none",
                                        }
                                    }
                                }
                            }
                        }

                        if let Some((sx, sy)) = active_cabinetry_start_point() {
                            circle {
                                cx: "{sx}",
                                cy: "{sy}",
                                r: "{overlay_point_r}",
                                fill: "rgba(202,138,4,0.30)",
                                stroke: "{cabinetry_stroke}",
                                stroke_width: "{overlay_point_stroke_w}",
                                pointer_events: "none",
                            }
                            if cursor_inside() {
                                {
                                    let cw = cursor_world();
                                    rsx! {
                                        line {
                                            x1: "{sx}",
                                            y1: "{sy}",
                                            x2: "{cw.0}",
                                            y2: "{cw.1}",
                                            stroke: "{cabinetry_stroke}",
                                            stroke_width: "{overlay_stroke_w}",
                                            stroke_dasharray: "{overlay_dash}",
                                            pointer_events: "none",
                                        }
                                    }
                                }
                            }
                        }

                        if !footprint_points.is_empty() {
                            {
                                let outline_points = polygon_points_to_svg(&footprint_points);
                                let cursor_world_point = cursor_world();
                                let line_tail = footprint_points.last().copied();
                                let preview_stroke_w = footprint_stroke_w;
                                let point_stroke_w = footprint_point_stroke_w;
                                let point_r = footprint_point_r;
                                let preview_dash = format!(
                                    "{:.3} {:.3}",
                                    (6.0 / zoom().max(0.0001).powf(1.15)).clamp(4.0, 22.0),
                                    (6.0 / zoom().max(0.0001).powf(1.15)).clamp(4.0, 22.0)
                                );
                                let close_hint = if footprint_points.len() >= 3 && cursor_inside() {
                                    if let Some(&(fx, fy)) = footprint_points.first() {
                                        let snap_threshold = 15.0 / zoom().max(0.0001);
                                        let dist =
                                            ((cursor_world_point.0 - fx).powi(2) + (cursor_world_point.1 - fy).powi(2)).sqrt();
                                        Some((fx, fy, dist < snap_threshold))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };

                                rsx! {

                                    if let Some((fx, fy, show)) = close_hint {
                                        if show {
                                            circle {
                                                cx: "{fx}",
                                                cy: "{fy}",
                                                r: "{18.0 / zoom().max(0.0001)}",
                                                fill: "rgba(34,197,94,0.35)",
                                                stroke: "#22C55E",
                                                stroke_width: "{preview_stroke_w}",
                                                pointer_events: "none",
                                            }
                                        }
                                    }

                                    polyline {
                                        points: "{outline_points}",
                                        fill: "none",
                                        stroke: "{footprint_stroke}",
                                        stroke_width: "{preview_stroke_w}",
                                        pointer_events: "none",
                                    }

                                    for (idx, (x, y)) in footprint_points.iter().enumerate() {
                                        {
                                            let marker_r = if idx == 0 { point_r * 1.15 } else { point_r };
                                            let marker_fill = if idx == 0 { "rgba(37,99,235,0.30)" } else { "none" };
                                            let marker_stroke_w = if idx == 0 {
                                                point_stroke_w * 1.30
                                            } else {
                                                point_stroke_w
                                            };
                                            rsx! {
                                                circle {
                                                    cx: "{x}",
                                                    cy: "{y}",
                                                    r: "{marker_r}",
                                                    fill: "{marker_fill}",
                                                    stroke: "{footprint_stroke}",
                                                    stroke_width: "{marker_stroke_w}",
                                                    pointer_events: "none",
                                                }
                                            }
                                        }
                                    }

                                    if let Some((lx, ly)) = line_tail {
                                        if cursor_inside() {
                                            line {
                                                x1: "{lx}",
                                                y1: "{ly}",
                                                x2: "{cursor_world_point.0}",
                                                y2: "{cursor_world_point.1}",
                                                stroke: "{footprint_stroke}",
                                                stroke_width: "{preview_stroke_w}",
                                                stroke_dasharray: "{preview_dash}",
                                                pointer_events: "none",
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                CrosshairOverlay {
                    x: cursor_pos().0,
                    y: cursor_pos().1,
                    visible: drawing_tool_active && cursor_inside() && !mobile_scale_mode,
                }
                if mobile_scale_mode && mobile_scale_magnifier_visible() {
                    div { class: "estimate-scale-magnifier",
                        svg {
                            class: "estimate-scale-magnifier__canvas",
                            view_box: "0 0 {scale_magnifier_size} {scale_magnifier_size}",
                            rect {
                                x: "0",
                                y: "0",
                                width: "{scale_magnifier_size}",
                                height: "{scale_magnifier_size}",
                                fill: "var(--bg-primary)",
                            }
                            g {
                                transform: "translate({scale_magnifier_tx}, {scale_magnifier_ty}) scale({scale_magnifier_zoom})",
                                for page in pages.iter() {
                                    r#image {
                                        href: "{page.image_url}",
                                        x: "0",
                                        y: "{page.y_offset}",
                                        width: "{page.display_width}",
                                        height: "{page.display_height}",
                                        preserve_aspect_ratio: "xMinYMin meet",
                                        pointer_events: "none",
                                    }
                                }
                                for scale in saved_scales.iter() {
                                    { render_saved_scale_line(scale.clone(), scale_text_stroke.clone()) }
                                }
                                { render_active_scale_drawing(points.clone()) }
                            }
                        }
                        div { class: "estimate-scale-magnifier__crosshair-h" }
                        div { class: "estimate-scale-magnifier__crosshair-v" }
                    }
                }
            }
        }
    }
}
