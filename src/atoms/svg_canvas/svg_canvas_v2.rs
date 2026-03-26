


/**
 
| element_coordinates() | The element that caught the event (SVG) | Canvas operations |
| client_coordinates() | Browser viewport (0,0 = top-left of window) | Positioning popups/menus |
| screen_coordinates() | Physical monitor | Rarely used |
 
 ┌─────────────────────────────────────┐ ← Browser window
│  Navbar                             │
│  ┌───────────────────────────────┐  │
│  │ SVG canvas                    │  │
│  │   * click here                │  │
│  │                               │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘


element_coordinates: (100, 50)  ← relative to SVG top-left
client_coordinates:  (100, 90)  ← includes navbar height

Zoom = ommousewheel
Pan = onmousedown -> onmousemove -> onmouseup | onmouseleave - stop dragging if cursor leaves the canvas - edge case



# Convert between screen and world
screen = world * zoom + pan
world = (screen - pan) / zoom

# Cursor-based zoom
new_pan = cursor - (cursor - old_pan) * (new_zoom / old_zoom)

(topmost, receives clicks first)
┌───────────────────────────┐
│ 1 - Annotations (g)           │ pointer-events: auto   | in pan/zoom <g>
└───────────────────────────┘
┌───────────────────────────┐
│ 2 - Draw area (rect)          │ pointer-events: auto   | in pan/zoom <g>
└───────────────────────────┘
┌───────────────────────────┐
│ 3 - Image (image)             │ pointer-events: none   | in pan/zoom <g>
└───────────────────────────┘
┌───────────────────────────┐
│ 4 - Grid fill (rect + pattern)│ pointer-events: none   | outside <g>, pattern uses pan/zoom
└───────────────────────────┘
┌───────────────────────────┐
│ 5- Background (rect)         │ pointer-events: auto   | outside <g>, good for pan start
└───────────────────────────┘
(bottom)

**/

use std::rc::Rc;
use crate::blocks::annotations::state::state_create_annotation;
use crate::blocks::block_list::state::LABELS;
use std::collections::HashSet;
use web_time::Instant;
use std::time::Duration;
use gloo_timers::future::TimeoutFuture;
use dioxus::prelude::*;
use dioxus::logger::tracing;
use wasm_bindgen::JsCast;
use super::state::Tool;
use super::crosshair_overlay::{CrosshairOverlay, BboxCrosshairOverlay};
use super:: {Geometry, Point};
use crate::blocks::annotations::models::{Annotation, CommentThread};
use crate::shell::{THEME, Theme};
use crate::blocks::annotations::state::{state_update_annotation_geometry, state_delete_annotation};

const SVG_CANVAS_V2_CSS: &str = include_str!("svg_canvas_v2.css");
const COMMENT_CURSOR_BLUE: Asset = asset!("/assets/icons/comment-blue.svg");



#[component]
pub fn SvgCanvasV2(
	block_id:String,
	image_id:String,
	image_url: String, 
	selected_tool: Tool,
	active_drawing: Signal<Vec<(f64, f64)>>,
	annotations: Signal<Vec<Annotation>>,
	comment_threads: Signal<Vec<CommentThread>>,
	selected_label_id:String,
	on_annotation_context_menu: EventHandler<(String, String, f64, f64)>,
	on_comment_click: EventHandler<(f64, f64, f64, f64)>,
	on_marker_click: EventHandler<(String, f64, f64)>,
	active_thread_id: Option<String>,
	scroll_to_thread: Signal<Option<String>>,
	hidden_label_ids:HashSet<String>,
	hovered_label_id: Signal<Option<String>>,
	show_comments: bool,
	#[props(default = 0.2)] annotation_opacity: f64,
	clipboard: Signal<Option<(Geometry, String)>>,
	paste_mode: Signal<bool>,
	copy_requested: Signal<u32>,

) -> Element {
	let mut pan_x = use_signal(|| 0.0_f64);
	let mut pan_y = use_signal(|| 0.0_f64);
	let mut zoom = use_signal(|| 1.0_f64);
	let mut cursor_pos = use_signal(|| (0.0_f64, 0.0_f64));
	let mut cursor_world = use_signal(|| (0.0_f64, 0.0_f64));
	let mut cursor_inside = use_signal(|| false);
	let mut image_size = use_signal(|| (0.0_f64, 0.0_f64));
	let mut is_dragging = use_signal(|| false);
	let mut last_mouse = use_signal(|| (0.0_f64, 0.0_f64));
	let mut space_held = use_signal(|| false);
	let mut click_start = use_signal(|| (0.0_f64, 0.0_f64));
	let mut is_panning = use_signal(|| false);
	let mut undo_last_point = use_signal(|| false);
	let mut selected_ann_id:Signal<String> = use_signal(|| String::new()); // Select Annnotation
	let mut dragging_node:Signal<i32> = use_signal(|| -1); // Adjusting nodes
	let mut node_move_happened = use_signal(|| false); // This is used to del nodes is no movement happened
	let mut dirty_ann_ids:Signal<HashSet<String>> = use_signal(HashSet::new); // Hashset is instant O(1) where vec is O(n) its a[1] or like a house no rather than loooking up every house on the street
	let mut last_dirty_tick = use_signal(Instant::now);
    let mut touch_pointer_id: Signal<Option<i32>> = use_signal(|| None);
    let mut touch_drag_active = use_signal(|| false);
    let mut skip_polygon_insert = use_signal(|| false);
    let mut bbox_start: Signal<Option<(f64, f64)>> = use_signal(|| None);

	// Clear bbox first-click when leaving BBox tool (e.g. Escape)
	if selected_tool != Tool::BBox { bbox_start.set(None); }

	let dot_spacing = 30.0; // Dot spacing in world units
	let dot_min_px = 15.0;  // minimum on-screen spacing so dots stay visible
	let dot_world = {
	    let z = zoom();
	    let base = dot_spacing;
	    if base * z < dot_min_px { dot_min_px / z } else { base }
	};
	let dot_center = dot_world / 2.0;
	let selected_label_id_for_mouse_move = selected_label_id.clone();
	let selected_label_id_for_draw_area = selected_label_id.clone();
	

	// Auto save feature for annotators
	let block_id_for_save = block_id.clone();
	let image_id_for_save = image_id.clone();
	let last_dirty_tick_save = last_dirty_tick.clone();
	let annotations_save = annotations.clone();
	let mut dirty_ann_ids_save = dirty_ann_ids.clone();

	use_future (move || {
		let blk_id = block_id_for_save.clone();
		let img_id = image_id_for_save.clone();
		async move{
			loop{
				TimeoutFuture::new(400).await;
				
				// Noting to save
				if dirty_ann_ids_save().is_empty() { continue ;} // Go to the next iteration which is next time poll at 1 sec etc

				// This is the debounce, waits for idle time once user is finished
				if last_dirty_tick_save().elapsed() < Duration::from_millis(400) { continue; }

				let dirty_ids:Vec<String> = dirty_ann_ids_save.write().drain().collect();

				for aid in dirty_ids {
					let geom = {
						annotations_save.read().iter().find(|a| a.id == aid).map(|a| a.geometry.clone())
					};
					if let Some(g) = geom {
						state_update_annotation_geometry(&blk_id, &img_id, &aid, g, annotations_save);
					} 
					
				}

			}
		}
	});

	// Copy selected annotation to clipboard when Ctrl+C is pressed
	let hovered_ann_id_for_copy: Signal<Option<String>> = use_signal(|| None);
	use_effect(move || {
		let tick = copy_requested();
		if tick == 0 { return; }
		let aid = {
			let sel = selected_ann_id();
			if !sel.is_empty() { sel } else { hovered_ann_id_for_copy().unwrap_or_default() }
		};
		if aid.is_empty() { copy_requested.set(0); return; }
		if let Some(ann) = annotations().iter().find(|a| a.id == aid) {
			clipboard.set(Some((ann.geometry.clone(), ann.label_id.clone())));
			tracing::info!("📋 Annotation copied to clipboard");
		}
		copy_requested.set(0);
	});

	// Scroll to thread when sidebar triggers it (animated)
	use_effect(move || {
		if let Some(tid) = scroll_to_thread() {
			scroll_to_thread.set(None);
			if let Some(thread) = comment_threads().iter().find(|t| t.id == tid).cloned() {
				let (canvas_cx, canvas_cy, svg_left, svg_top) = {
					#[cfg(target_arch = "wasm32")]
					{
						web_sys::window()
							.and_then(|w| w.document())
							.and_then(|d| d.get_element_by_id("svg-canvas-v2"))
							.map(|el| {
								let r = el.get_bounding_client_rect();
								(r.width() / 2.0, r.height() / 2.0, r.left(), r.top())
							})
							.unwrap_or((400.0, 300.0, 0.0, 0.0))
					}
					#[cfg(not(target_arch = "wasm32"))]
					{ (400.0, 300.0, 0.0, 0.0) }
				};
				let target_px = canvas_cx - thread.world_x * zoom();
				let target_py = canvas_cy - thread.world_y * zoom();
				let start_px = pan_x();
				let start_py = pan_y();
				// Animate pan over ~300ms with ease-out cubic
				spawn(async move {
					let steps = 20;
					let step_ms = 16; // ~60fps
					for i in 1..=steps {
						let t = i as f64 / steps as f64;
						let ease = 1.0 - (1.0 - t).powi(3); // ease-out cubic
						pan_x.set(start_px + (target_px - start_px) * ease);
						pan_y.set(start_py + (target_py - start_py) * ease);
						TimeoutFuture::new(step_ms).await;
					}
					// Final exact position + open dialog
					pan_x.set(target_px);
					pan_y.set(target_py);
					let sx = thread.world_x * zoom() + pan_x() + svg_left;
					let sy = thread.world_y * zoom() + pan_y() + svg_top;
					on_marker_click.call((tid, sx, sy));
				});
			}
		}
	});

	rsx!{
		style { {SVG_CANVAS_V2_CSS} }
		svg{
			id: "svg-canvas-v2",
			tabindex: "0",
			class:"svg-canvas-v2",
			width:"100%",
			height:"100%",
			style: {
				let comment_cursor = COMMENT_CURSOR_BLUE;
				let comment_cursor_style = format!("cursor:url('{}') 0 16, auto;", comment_cursor);
				if paste_mode() {
					"cursor:crosshair;".to_string()
				} else if selected_tool == Tool::Polygon {
					"cursor:none;".to_string()
				} else if selected_tool == Tool::BBox {
					"cursor:none;".to_string()
				} else if selected_tool == Tool::Comment {
					comment_cursor_style
				} else if is_dragging() {
					"cursor:grabbing;".to_string()
				} else {
					"cursor:default;".to_string()
				}
			},
			oncontextmenu: move |evt| {
				
				if selected_tool == Tool::Polygon { 		// Only when we are drawing pop the last node, or we need context menu
					let mut pts = active_drawing.write();	// to change classes or del annotations
					if !pts.is_empty() {	
						evt.prevent_default();
						evt.stop_propagation();
						pts.pop();
						undo_last_point.set(true);
					}
				}
			},

			onmouseenter: move|_| cursor_inside.set(true),

			onmouseleave: move |_| {
				is_dragging.set(false);
				is_panning.set(false);
				cursor_inside.set(false);
				
			},

            // This is all for one finger down CVAT style panning
            onpointerdown: move |evt| {
               let pointer_type = format!("{:?}", evt.data.pointer_type());
                tracing::info!("👆 pointerdown: type={}", pointer_type);
                
                // Only handle touch - mouse handled by onmousedown
                if !pointer_type.contains("Touch") {
                    return;
                }
                evt.prevent_default();
                touch_pointer_id.set(Some(evt.data.pointer_id() as i32));
                touch_drag_active.set(true);
                is_dragging.set(true);
                is_panning.set(false);
                let p = evt.element_coordinates();
                last_mouse.set((p.x, p.y));
                click_start.set((p.x, p.y));
            },

            onpointermove: move |evt| {
                 let pointer_type = format!("{:?}", evt.data.pointer_type());
                if !pointer_type.contains("Touch") {
                    return;
                }
                if touch_pointer_id() != Some(evt.data.pointer_id() as i32) || !touch_drag_active() {
                    return;
                }
                let p = evt.element_coordinates();
                cursor_pos.set((p.x, p.y));
                let wx = (p.x - pan_x()) / zoom();
                let wy = (p.y - pan_y()) / zoom();
                cursor_world.set((wx, wy));
                
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

            onpointerup: move |evt| {
                 let pointer_type = format!("{:?}", evt.data.pointer_type());
                if !pointer_type.contains("Touch") {
                    return;
                }
                if touch_pointer_id() == Some(evt.data.pointer_id() as i32) {
                    tracing::info!("👆 touch pointerup");
                    touch_pointer_id.set(None);
                    touch_drag_active.set(false);
                    is_dragging.set(false);
                    is_panning.set(false);
                }
            },

            onpointercancel: move |_| {
                touch_pointer_id.set(None);
                touch_drag_active.set(false);
                is_dragging.set(false);
                is_panning.set(false);
            },

			onmousedown: move |evt| {
				tracing::info!("🖱️ mousedown: button={:?}, mods={:?}", 
		        evt.data.trigger_button(), 
		        evt.data.modifiers());
				if dragging_node() >= 0 { return; } // Node drag in progress, don't start pan
				is_dragging.set(true);
				is_panning.set(false);
				undo_last_point.set(false);
				let p = evt.element_coordinates();
				last_mouse.set((p.x,p.y)); // needed for panning
				click_start.set((p.x,p.y)); // needed for calculating travel
			},

			onmousemove: move |evt| {
				let p = evt.element_coordinates();
				cursor_pos.set((p.x,p.y));
				let wx = (p.x - pan_x()) /zoom();
				let wy = (p.y - pan_y())/zoom();
				cursor_world.set((wx, wy)); //set world coords

			
			    // ══════════════════════════════════════════════════════════
			    // NODE DRAGGING: If a node is being dragged, update its position
			    // This runs at SVG level so it works even when mouse moves fast
			    // ══════════════════════════════════════════════════════════
			    if dragging_node() >= 0 { // Node has been moved
			        node_move_happened.set(true);
			        let idx = dragging_node() as usize;
			        let aid = selected_ann_id();
			        tracing::info!("🔄 Dragging node {} to ({}, {})", idx, wx, wy);
				    
				    {	
				    	let mut anns = annotations.write();
				        if let Some(a) = anns.iter_mut().find(|a| a.id == aid) {
				            match &mut a.geometry {
				                Geometry::Polygon { ref mut points } => {
				                    if idx < points.len() {
				                        points[idx] = Point { x: wx, y: wy };
				                    }
				                }
				                Geometry::BBox { ref mut start, ref mut end } => {
				                    // Corners: 0=TL(start.x,start.y) 1=TR(end.x,start.y)
				                    //          2=BR(end.x,end.y)     3=BL(start.x,end.y)
				                    match idx {
				                        0 => { start.x = wx; start.y = wy; }
				                        1 => { end.x = wx; start.y = wy; }
				                        2 => { end.x = wx; end.y = wy; }
				                        3 => { start.x = wx; end.y = wy; }
				                        _ => {}
				                    }
				                }
				            }
				        }
					}
			        return; // Don't do panning while dragging a node
			    }



				if !is_dragging() {return;} // no panning logic needed if mouse not down and moving
				
				// if is_dragging() {
			    //     tracing::info!("🖱️ mousemove (dragging): pos={:?}", evt.element_coordinates());
			    // }
				let (sx,sy) = click_start();
				let travel = ((p.x-sx).powi(2) + (p.y-sy).powi(2)).sqrt(); // All calcs in screen coords
				if travel > 5.0 { is_panning.set(true); }
				
				if is_panning(){
					let (lx,ly) = last_mouse();
					let travel_x = p.x - lx; // calculate how much pan has travelled
					let travel_y = p.y - ly; // and adjust for pan
					pan_x.set(pan_x() + travel_x);
					pan_y.set(pan_y() + travel_y);
					last_mouse.set((p.x, p.y));  
				}	
			},

			onmouseup: move |evt| {
				tracing::info!("🖱️ mouseup: button={:?}", evt.data.trigger_button());
 
				
				// ══════════════════════════════════════════════════════════
			    // NODE DRAG END: Save geometry to backend
			    // ══════════════════════════════════════════════════════════
			    if dragging_node() >= 0 {
			        let aid = selected_ann_id();
			        let img_id = image_id.clone();
			        // let moved = node_move_happened();
			       
			        
			        let geometry = annotations().iter()
			            .find(|a| a.id == aid)
			            .map(|a| a.geometry.clone());

			       

			        if !node_move_happened(){
			        	info!("node move didnt happen");
			        	info!("the index to be deleted is {}", dragging_node );
			        }
			        // Node move happened now we can send to Server for saving ...
			        else {
			        	// Mark Dirty
			        	 tracing::info!("💾 Saving node drag for annotation {}", aid);
				         dirty_ann_ids.write().insert(aid.clone());
				         last_dirty_tick.set(Instant::now());	
				        
			        }
			        
			        
			        dragging_node.set(-1);
			        node_move_happened.set(false); // reset node_move_happened
			        return;
			    }

				let selected_label_id_for_mouse_move1 = selected_label_id_for_mouse_move.clone();
				if !is_dragging() { return; }

			    let p = evt.element_coordinates();
			    let (sx, sy) = click_start();
			    let travel = ((p.x - sx).powi(2) + (p.y - sy).powi(2)).sqrt();
			    let was_pan = travel > 5.0 || is_panning(); // Just be conservative
			    is_dragging.set(false);
    			is_panning.set(false);

    			// if it was a pan, stop here
			    if was_pan { return; }

			    // ══════════════════════════════════════════════════════════
			    // PASTE MODE: click to place copied annotation
			    // ══════════════════════════════════════════════════════════
    if paste_mode() {
			        if let Some((ref geom, ref label_id)) = clipboard() {
			            let wx = (p.x - pan_x()) / zoom();
			            let wy = (p.y - pan_y()) / zoom();
			            let new_geom = offset_geometry_to(geom, wx, wy);
			            let ann_id = uuid::Uuid::new_v4().to_string();
                    selected_ann_id.set(ann_id.clone());
                    let label_id = label_id.clone();
                    let label_name = crate::blocks::block_list::state::LABELS.read()
                        .iter()
                        .find(|l| l.label_id == label_id)
                        .map(|l| l.label_name.clone())
                        .unwrap_or_default();
                        let img_id = image_id.clone();
			            let block_id1 = block_id.clone();
			            state_create_annotation(&block_id1, &img_id, &ann_id, &label_id, &label_name, new_geom, annotations);
			        }
			        paste_mode.set(false);
			        return;
			    }

			    // Check if click hit an existing comment marker (only when not drawing and comments visible)
			    if show_comments && active_drawing().is_empty() {
			        let wx = (p.x - pan_x()) / zoom();
			        let wy = (p.y - pan_y()) / zoom();
			        let marker_size = (18.0 / zoom()).clamp(10.0, 28.0);
			        let hit_radius = marker_size;
			        if let Some(thread) = comment_threads().iter().find(|t| {
			            let dx = wx - t.world_x;
			            let dy = wy - (t.world_y - marker_size / 2.0);
			            (dx * dx + dy * dy).sqrt() < hit_radius
			        }) {
			            let tid = thread.id.clone();
			            on_marker_click.call((tid, evt.client_coordinates().x, evt.client_coordinates().y));
			            return;
			        }
			    }

			    // Comment tool: click to place comment pin
			    if selected_tool == Tool::Comment {
			        let wx = (p.x - pan_x()) / zoom();
			        let wy = (p.y - pan_y()) / zoom();
			        on_comment_click.call((wx, wy, evt.client_coordinates().x, evt.client_coordinates().y));
			        return;
			    }

			    // BBox tool: 2-click to create bbox
			    if selected_tool == Tool::BBox {
			        let wx = (p.x - pan_x()) / zoom();
			        let wy = (p.y - pan_y()) / zoom();
			        if let Some((sx, sy)) = bbox_start() {
			            // Second click: create bbox annotation
			            let width = (wx - sx).abs();
			            let height = (wy - sy).abs();
            if width > 1.0 && height > 1.0 {
			                let geometry = Geometry::BBox {
			                    start: Point { x: sx.min(wx), y: sy.min(wy) },
			                    end: Point { x: sx.max(wx), y: sy.max(wy) },
			                };
                    let ann_id = uuid::Uuid::new_v4().to_string();
                    selected_ann_id.set(ann_id.clone()); // Auto-select so nodes appear immediately
                    let label_id = selected_label_id_for_mouse_move1;
                    let label_name = crate::blocks::block_list::state::LABELS.read()
                        .iter()
                        .find(|l| l.label_id == label_id)
                        .map(|l| l.label_name.clone())
                        .unwrap_or_default();
                        let img_id = image_id.clone();
			                let block_id1 = block_id.clone();
			                state_create_annotation(&block_id1, &img_id, &ann_id, &label_id, &label_name, geometry, annotations);
			            }
			            bbox_start.set(None);
			        } else {
			            // First click: store start point
			            bbox_start.set(Some((wx, wy)));
			        }
			        return;
			    }

			    if selected_tool != Tool::Polygon { return; }


			    // treat as polygon click
			    evt.prevent_default();
			    evt.stop_propagation();

			    let wx = (p.x - pan_x()) / zoom();
			    let wy = (p.y - pan_y()) / zoom();
			    let snap_threshold = 15.0 / zoom();
			    let mut pts = active_drawing.write();

			    
			    // CREATING NEW POLYGON ANNOTATIONS IF  GREATER THAN 3 CL
			    if pts.len() >= 3 {
			        if let Some(&(fx, fy)) = pts.first() {
			            let dist = ((wx - fx).powi(2) + (wy - fy).powi(2)).sqrt();
			            if dist < snap_threshold {
                let points: Vec<Point> = pts.iter().copied().map(|(x,y)| Point { x, y }).collect();
                let geometry = Geometry::Polygon {points};
                    let ann_id = uuid::Uuid::new_v4().to_string();
                    selected_ann_id.set(ann_id.clone()); // Auto-select so nodes appear immediately
                    let label_id = selected_label_id_for_mouse_move1;
                    let label_name = crate::blocks::block_list::state::LABELS.read()
                        .iter()
                        .find(|l| l.label_id == label_id)
                        .map(|l| l.label_name.clone())
                        .unwrap_or_default();
			                let img_id = image_id.clone();
			            	
                        let block_id1 = block_id.clone();
			                state_create_annotation(&block_id1, &img_id, &ann_id, &label_id, &label_name, geometry, annotations);
			                
			                tracing::info!("Polygon save intiated{}", annotations.read().len());
			                pts.clear();
			                return;
			            }
			        }
			    }
			    if !undo_last_point() { 
				    pts.push((wx, wy));
				    tracing::info!("adding point");
				}
			},

			// Zoom on scroll
			onwheel: move |evt| {
				let delta = evt.delta().strip_units();
			    tracing::info!("🎡 wheel: delta=({}, {}), ctrl={}, shift={}", 
			        delta.x, delta.y,
			        evt.data.modifiers().ctrl(),
			        evt.data.modifiers().shift());
				evt.prevent_default();
				let dy = evt.delta().strip_units().y; // scroll wheel up and down
let factor = (-dy * 0.009).exp().clamp(0.7,1.4); // zoom speed
				let new_zoom = (zoom() * factor).clamp(0.1, 10.0);

				// Cursor-based zoom: adjust pan so point under cursor stays the same
				// new_pan = cursor - (cursor - old_pan) * (new_zoom / old_zoom)
				let p = evt.element_coordinates();
				let scale_change = new_zoom / zoom();
				let new_pan_x = p.x - (p.x - pan_x()) * scale_change;
				let new_pan_y = p.y - (p.y - pan_y()) * scale_change;
				pan_x.set(new_pan_x);
				pan_y.set(new_pan_y);


				zoom.set(new_zoom);
				tracing::info!("factor: {}", factor);
				tracing::info!("zoom: {}", zoom);

			},



			// Dot pattern in fixed world units (zooms with canvas)
			// 60 world units ≈ 30px on screen at typical initial zoom (~0.5)
			defs {
				pattern {
					id: "dot-pattern",
					width: "60",
					height: "60",
					pattern_units: "userSpaceOnUse",
					circle {
						cx: "30",
						cy: "30",
						r: "2",
						fill: "var(--dot-color)",
						pointer_events: "none",
					}
				}
			}
			
			// 5. Background: bottom layer, catches pan/zoom (pointer-events: auto by default)
			rect { 
					width:"100%", 
					height:"100%", 
					fill:"var(--bg-primary)",
				}

			// World group: everything that pans/zooms together
			g{
				transform:"translate({pan_x()}, {pan_y()}) scale({zoom()})",
			// Dot grid behind image (hidden until image loads to prevent zoomed-in flash)
				if image_size() != (0.0, 0.0) {
					rect { 
						x: "-10000", 
						y: "-10000", 
						width: "20000", 
						height: "20000", 
						fill: "url(#dot-pattern)", 
						pointer_events: "none",
					}
				}
				//Image (non-interactive) - renders on top of dots
				r#image {
					class: "canvas-image",
					href: "{image_url}",
					x: "0",
					y: "0",
					pointer_events:"none",
					// width: "{image_size().0}",
					// height: "{image_size().1}",
					onload: move |_| {
						#[cfg(target_arch = "wasm32")]
						{
							// Read old zoom BEFORE resetting (DOM still has old transform)
							let old_zoom = zoom();
							
							if let Some(win) = web_sys::window() {
								if let Some(doc) = win.document(){
									if let Some(svg_el) = doc.get_element_by_id("svg-canvas-v2") {
										
										if let Ok(Some(img_el)) = svg_el.query_selector(".canvas-image") {
											let img_rect = img_el.get_bounding_client_rect();
											// Divide by old_zoom to get natural dimensions
											let iw = img_rect.width() / old_zoom;
											let ih = img_rect.height() / old_zoom;
											image_size.set((iw, ih));

											let svg_rect = svg_el.get_bounding_client_rect();
											let vw = svg_rect.width();
											let vh = svg_rect.height();

											let fit_zoom = (vw/iw).min(vh/ih) * 0.75;
											zoom.set(fit_zoom);
											pan_x.set((vw - iw * fit_zoom)/2.0);
											pan_y.set((vh - ih * fit_zoom)/2.0);
										}
									}	
								}
							}
						}
					}
				}
				if image_size() != (0.0, 0.0) {
					// CURRENT DRAWING
					DrawArea {
						image_size: image_size(), 
						selected_tool: selected_tool, 
						zoom: zoom, 
						pan_x: pan_x, 
						pan_y: pan_y, 
						active_drawing:active_drawing, 
						annotations: annotations, 
						selected_label_id:selected_label_id_for_draw_area,
						selected_ann_id: selected_ann_id,
						is_panning:is_panning,
					},

					// PREVIEW DRAWING
					PolygonPreview { points: active_drawing(), cursor_world_pos: cursor_world(), zoom: zoom(), selected_label_id: selected_label_id.clone() },
					BboxPreview { bbox_start: bbox_start(), cursor_world_pos: cursor_world(), zoom: zoom(), selected_label_id: selected_label_id.clone() },

					// PASTE PREVIEW (ghost following cursor)
					if paste_mode() {
						PastePreview { clipboard: clipboard(), cursor_world_pos: cursor_world(), zoom: zoom() }
					}

					// SAVED DRAWING
					SavedAnnotation {
						block_id:block_id.clone(),
						annotations:annotations,
						on_annotation_context_menu:on_annotation_context_menu,
						selected_ann_id: selected_ann_id,
						dragging_node: dragging_node,
						cursor_world: cursor_world,
						zoom:zoom,
						image_id: image_id.clone(),
						dirty_ann_ids:dirty_ann_ids,
						last_dirty_tick:last_dirty_tick,
						skip_polygon_insert: skip_polygon_insert,
						hidden_label_ids:hidden_label_ids.clone(),
						hovered_label_id: hovered_label_id,
						hovered_ann_id_out: hovered_ann_id_for_copy,
						annotation_opacity: annotation_opacity,
						selected_tool: selected_tool,
					}
					if show_comments {
						CommentMarkers { threads: comment_threads(), zoom: zoom(), active_thread_id: active_thread_id.clone() }
					}
				}

			}
		}
		if image_size() == (0.0, 0.0) {
			div {
				style: "position:absolute; top:50%; left:50%; transform:translate(-50%, -50%); font-family:'Helvetica Neue', Helvetica, Arial, sans-serif; font-size:12px; font-weight:300; color:var(--text-secondary); pointer-events:none;",
				"Loading image..."
			}
		}
		CrosshairOverlay {		// Uses div has to be outsidfe svg{}
				x:cursor_pos().0,
				y:cursor_pos().1,
				visible: (selected_tool == Tool::Polygon || dragging_node() >=0) && cursor_inside(),
			}
		BboxCrosshairOverlay {
				x: cursor_pos().0,
				y: cursor_pos().1,
				visible: selected_tool == Tool::BBox && cursor_inside(),
			}
	}
}

/**
 All drawing happens here
 Click Measurements, Snap Threshold, 
 onmousedown()
 */
#[component]
fn DrawArea(
	image_size: (f64,f64),
	selected_tool:Tool,
	zoom:Signal<f64>,
	pan_x:Signal<f64>,
	pan_y:Signal<f64>,
	active_drawing:Signal<Vec<(f64,f64)>>,
	annotations:Signal<Vec<Annotation>>,
	selected_label_id:String,
	mut selected_ann_id: Signal<String>,
	is_panning:Signal<bool>,

	)->Element{
	

	rsx!{
		// Draw area (interactive)
		rect {
			class:"draw-area",
			x:"0",
			y:"0",
			width: "{image_size.0}",
			height: "{image_size.1}",
			fill: "transparent",
			pointer_events:"auto",
			onclick: move|_| {
				// Only deselect with Arrow tool — drawing tools fire click on
				// DrawArea right after creating an annotation (before the new
				// polygon is in the DOM), which would undo the auto-select.
				if selected_tool == Tool::Select {
					selected_ann_id.set(String::new());
				}
			}
		}
	}
}

#[component]
fn CommentMarkers(
	threads: Vec<CommentThread>,
	zoom: f64,
	active_thread_id: Option<String>,
) -> Element {
	if threads.is_empty() { return rsx!{}; }

	rsx! {
		for (i, thread) in threads.iter().enumerate() {
			{
				let is_active = active_thread_id.as_ref() == Some(&thread.id);
                let size = if is_active { 14.0 / zoom } else { 24.0 / zoom };
                let font_size = if is_active { 7.0 / zoom } else { 12.0 / zoom };
				let scale = size / 16.0;
				let tx = thread.world_x;
				let ty = thread.world_y - size;
				let cx = thread.world_x + size / 2.0;
				let cy = thread.world_y - size / 2.0;
				rsx! {
					g {
						key: "{thread.id}",
						pointer_events: "none",
					path {
						d: "M0 8C0 3.58172 3.58172 0 8 0C12.4183 0 16 3.58172 16 8C16 12.4183 12.4183 16 8 16H0V8Z",
						fill: if thread.resolved { "#34C759" } else { "#009CFF" },
							transform: "translate({tx}, {ty}) scale({scale})",
						}
						text {
							x: "{cx}",
							y: "{cy}",
							text_anchor: "middle",
							dominant_baseline: "central",
							fill: "white",
							font_size: "{font_size}",
							font_family: "Helvetica Neue Light, Helvetica Light, Helvetica, Arial, sans-serif",
                            font_weight: "400",
							"{i + 1}"
						}
					}
				}
			}
		}
	}
}

#[component]
fn PolygonPreview(
	points:Vec<(f64,f64)>,
	cursor_world_pos: (f64,f64),
	zoom:f64,
	selected_label_id: String,
	) -> Element {

	if points.is_empty() { return rsx!{};}

	let is_dark = THEME() == Theme::Dark;
	let stroke_w = (0.9/zoom).clamp(0.1, 2.7);
	let circle_r = (6.0/zoom).clamp(0.75, 18.0);
	let snap_thresh = 15.0 / zoom;
	
	// Get label color
	let labels = LABELS();
	let label_color = labels.iter()
		.find(|l| l.label_id == selected_label_id)
		.map(|l| l.label_color.clone())
		.unwrap_or_else(|| "#22C55E".to_string());
	
	// Convert to rgba for fill (handles both #hex and rgb() formats)
	let fill_rgba = if label_color.starts_with("rgb(") {
		label_color.replace("rgb(", "rgba(").replace(")", ",0.15)")
	} else if label_color.starts_with("#") && label_color.len() == 7 {
		let r = u8::from_str_radix(&label_color[1..3], 16).unwrap_or(0);
		let g = u8::from_str_radix(&label_color[3..5], 16).unwrap_or(0);
		let b = u8::from_str_radix(&label_color[5..7], 16).unwrap_or(0);
		format!("rgba({},{},{},0.15)", r, g, b)
	} else {
		"rgba(0,0,0,0.15)".to_string()
	};

	let fmt = |pts: &[(f64, f64)]| {
	    pts.iter().fold(String::new(), |mut s, (x, y)| {
	        if !s.is_empty() { s.push(' '); }
	        s.push_str(&format!("{x},{y}"));
	        s
	    })
	};

	let points_str = fmt(&points);

	let fill_points = {
        let mut s = points_str.clone();
        s.push(' ');
        s.push_str(&format!("{},{}", cursor_world_pos.0, cursor_world_pos.1));
        s
    };

    let can_close = points.len() >= 3 && points.first().map_or(false, |(fx, fy)| { // Calculate distance via pythagores <=15
        let dx = cursor_world_pos.0 - fx;
        let dy = cursor_world_pos.1 - fy;
        (dx*dx + dy*dy).sqrt() < snap_thresh
    });


    rsx!{   
        // Fill preview
        polygon {
            points: "{fill_points}",
            fill: "{fill_rgba}",
            stroke: "none",
            pointer_events: "none",
        }

        // Stroke preview
        polyline {
            points: "{points_str}",
            fill: "none",
            stroke: "{label_color}",
            stroke_width: "1.5",
            vector_effect: "non-scaling-stroke",
            pointer_events: "none",
        }

        // Vertex circles
        for (i, (x, y)) in points.iter().enumerate() {
            circle {
                key: "node-{i}",
                cx: "{x}", cy: "{y}", r: "{circle_r}",
                fill: "{label_color}",
                stroke: "white",
                stroke_width: "{stroke_w}",
                pointer_events: "none",
            }
        }

        // Preview line to cursor
        if let Some(&(lx, ly)) = points.last() {
            line {
                x1: "{lx}", y1: "{ly}",
                x2: "{cursor_world_pos.0}", y2: "{cursor_world_pos.1}",
                stroke: "{label_color}",
                stroke_width: "1.5",
                vector_effect: "non-scaling-stroke",
                pointer_events: "none",
            }
        }

        // Snap indicator
        if can_close {
            if let Some(&(fx, fy)) = points.first() {
                circle {
                    cx: "{fx}", cy: "{fy}",
                    r: "{circle_r * 2.0}",
                    fill: "{fill_rgba}",
                    stroke: "{label_color}",
                    stroke_width: "{stroke_w * 2.0}",
                    pointer_events: "none",
                }
            }
        }
    }
}

#[component]
fn BboxPreview(
	bbox_start: Option<(f64, f64)>,
	cursor_world_pos: (f64, f64),
	zoom: f64,
	selected_label_id: String,
) -> Element {
	let Some((sx, sy)) = bbox_start else { return rsx!{}; };

	let stroke_w = (0.9 / zoom).clamp(0.1, 2.7);
	let circle_r = (6.0 / zoom).clamp(0.75, 18.0);

	// Get label color (same as PolygonPreview)
	let labels = LABELS();
	let label_color = labels.iter()
		.find(|l| l.label_id == selected_label_id)
		.map(|l| l.label_color.clone())
		.unwrap_or_else(|| "#22C55E".to_string());

	// Convert to rgba for fill (handles both #hex and rgb() formats)
	let fill_rgba = if label_color.starts_with("rgb(") {
		label_color.replace("rgb(", "rgba(").replace(")", ",0.15)")
	} else if label_color.starts_with("#") && label_color.len() == 7 {
		let r = u8::from_str_radix(&label_color[1..3], 16).unwrap_or(0);
		let g = u8::from_str_radix(&label_color[3..5], 16).unwrap_or(0);
		let b = u8::from_str_radix(&label_color[5..7], 16).unwrap_or(0);
		format!("rgba({},{},{},0.15)", r, g, b)
	} else {
		"rgba(0,0,0,0.15)".to_string()
	};

	let (cx, cy) = cursor_world_pos;
	let x = sx.min(cx);
	let y = sy.min(cy);
	let w = (cx - sx).abs();
	let h = (cy - sy).abs();

	// 4 corner points
	let corners = [(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
	let points_str = format!("{},{} {},{} {},{} {},{}", x, y, x + w, y, x + w, y + h, x, y + h);

	rsx! {
		// Fill preview
		polygon {
			points: "{points_str}",
			fill: "{fill_rgba}",
			stroke: "none",
			pointer_events: "none",
		}

		// Stroke preview
		polygon {
			points: "{points_str}",
			fill: "none",
			stroke: "{label_color}",
			stroke_width: "1.5",
			vector_effect: "non-scaling-stroke",
			pointer_events: "none",
		}

		// Corner circles
		for (i, (cx, cy)) in corners.iter().enumerate() {
			circle {
				key: "bbox-node-{i}",
				cx: "{cx}", cy: "{cy}", r: "{circle_r}",
				fill: "{label_color}",
				stroke: "white",
				stroke_width: "{stroke_w}",
				pointer_events: "none",
			}
		}
	}
}

// ---------- Offset geometry centroid to target position (for copy-paste) ----------
fn offset_geometry_to(geom: &Geometry, target_x: f64, target_y: f64) -> Geometry {
	match geom {
		Geometry::Polygon { points } => {
			let n = points.len() as f64;
			if n == 0.0 { return geom.clone(); }
			let cx = points.iter().map(|p| p.x).sum::<f64>() / n;
			let cy = points.iter().map(|p| p.y).sum::<f64>() / n;
			let dx = target_x - cx;
			let dy = target_y - cy;
			Geometry::Polygon {
				points: points.iter().map(|p| Point { x: p.x + dx, y: p.y + dy }).collect()
			}
		}
		Geometry::BBox { start, end } => {
			let cx = (start.x + end.x) / 2.0;
			let cy = (start.y + end.y) / 2.0;
			let dx = target_x - cx;
			let dy = target_y - cy;
			Geometry::BBox {
				start: Point { x: start.x + dx, y: start.y + dy },
				end: Point { x: end.x + dx, y: end.y + dy },
			}
		}
	}
}

#[component]
fn PastePreview(
	clipboard: Option<(Geometry, String)>,
	cursor_world_pos: (f64, f64),
	zoom: f64,
) -> Element {
	let Some((ref geom, ref label_id)) = clipboard else { return rsx!{}; };

	// Get label color
	let labels = LABELS();
	let label_color = labels.iter()
		.find(|l| l.label_id == *label_id)
		.map(|l| l.label_color.clone())
		.unwrap_or_else(|| "#22C55E".to_string());

	// Semi-transparent fill
	let fill_rgba = if label_color.starts_with("#") && label_color.len() == 7 {
		let r = u8::from_str_radix(&label_color[1..3], 16).unwrap_or(0);
		let g = u8::from_str_radix(&label_color[3..5], 16).unwrap_or(0);
		let b = u8::from_str_radix(&label_color[5..7], 16).unwrap_or(0);
		format!("rgba({},{},{},0.25)", r, g, b)
	} else {
		"rgba(0,0,0,0.25)".to_string()
	};

	// Offset geometry so centroid follows cursor
	let offset_geom = offset_geometry_to(geom, cursor_world_pos.0, cursor_world_pos.1);

	match offset_geom {
		Geometry::Polygon { ref points } => {
			let points_str = points.iter()
				.map(|p| format!("{},{}", p.x, p.y))
				.collect::<Vec<_>>()
				.join(" ");
			rsx! {
				polygon {
					points: "{points_str}",
					fill: "{fill_rgba}",
					stroke: "{label_color}",
					stroke_width: "1.5",
					stroke_dasharray: "6 3",
					vector_effect: "non-scaling-stroke",
					pointer_events: "none",
				}
			}
		}
		Geometry::BBox { ref start, ref end } => {
			let x = start.x.min(end.x);
			let y = start.y.min(end.y);
			let w = (end.x - start.x).abs();
			let h = (end.y - start.y).abs();
			rsx! {
				rect {
					x: "{x}",
					y: "{y}",
					width: "{w}",
					height: "{h}",
					fill: "{fill_rgba}",
					stroke: "{label_color}",
					stroke_width: "1.5",
					stroke_dasharray: "6 3",
					vector_effect: "non-scaling-stroke",
					pointer_events: "none",
				}
			}
		}
	}
}

// ---------- Simple geometry helpers for inserting a point ----------
// Returns squared distance from click P to segment AB (avoids expensive sqrt).
//      B (4, 3)
   //     /|
   //    / |
   //   /  | 3 units (vertical)
   //  /   |
   // A----+
   // (1,0)
   
// 4 units (horizontal)
//    Vector AB = (bx - ax, by - ay)
//           = (4 - 1, 3 - 0)  
//           = (3, 3)

// So: abx = 3, aby = 3

// Length² = 3² + 3² = 9 + 9 = 18
// Length = √18 ≈ 4.24 units
fn dist_sq_to_segment(ax:f64, ay:f64, bx:f64, by:f64, px:f64, py:f64)-> f64{
	
	// Vector from A to B (the segment)
	let abx = bx - ax;
	let aby = by - ay;

	// Vector from A to P (the click point)
	let apx = px - ax;
	let apy = py - ay;

	// How long is the segment
	let segment_length_sq = abx * abx + aby * aby;  // Pythagorean

	// Edge case: A and B are the same point
    if segment_length_sq == 0.0 {
        return apx * apx + apy * apy;
    }

    // How far along the segment (0 to 1) is the closest point?
    // 0 = at point A, 0.5 = middle, 1 = at point B
    let how_far_along = (apx * abx + apy * aby) / segment_length_sq;

    // Keep it on the segment (not before A or after B)
    let how_far_along = how_far_along.clamp(0.0, 1.0);

    // Find the actual closest point on the segment
    let closest_x = ax + abx * how_far_along;
    let closest_y = ay + aby * how_far_along;

    // Return squared distance from P to closest point
    let dx = px - closest_x;
    let dy = py - closest_y;
    dx * dx + dy * dy
}

// Find which edge of the polygon is closest to the click
fn closest_edge(points: &[Point], px: f64, py: f64) -> Option<usize> {
    if points.len() < 2 {
        return None; // Need at least 2 points to make an edge
    }
    let mut closest_edge_index = 0;
    let mut smallest_distance = f64::INFINITY; // Start with a large number first and then fine tune

    // Check each edge of the polygon
    for i in 0..points.len() {
        let point_a = &points[i];
        let point_b = &points[(i + 1) % points.len()]; // Wrap around to close polygon

        let distance = dist_sq_to_segment(point_a.x, point_a.y, point_b.x, point_b.y, px, py);
        
        if distance < smallest_distance {
            smallest_distance = distance;
            closest_edge_index = i;
        }
    }
    Some(closest_edge_index)
}


// Add a new point to the polygon at the closest edge
fn insert_point_on_polygon(points: &mut Vec<Point>, px: f64, py: f64) {
    if let Some(edge_start) = closest_edge(points, px, py) {
        // Insert the new point right after the edge's start
        points.insert(edge_start + 1, Point { x: px, y: py });
    } else {
        // No edges found, just add to the end
        points.push(Point { x: px, y: py });
    }
}



#[component]
fn SavedAnnotation(
	block_id:String,
	annotations:Signal<Vec<Annotation>>,
	on_annotation_context_menu:EventHandler<(String,String,f64,f64)>,
	selected_ann_id: Signal<String>,
	dragging_node: Signal<i32>,
	cursor_world: Signal<(f64, f64)>,
	zoom: Signal<f64>,
	image_id: String,
	dirty_ann_ids: Signal<HashSet<String>>,
	last_dirty_tick: Signal<Instant>,
	skip_polygon_insert: Signal<bool>,
	hidden_label_ids:HashSet<String>,
	hovered_label_id: Signal<Option<String>>,
	hovered_ann_id_out: Signal<Option<String>>,
	#[props(default = 0.2)] annotation_opacity: f64,
	selected_tool: Tool,
	)->Element{
	let is_dark = THEME() == Theme::Dark;
	// tracing::info!("What is the theme: {}", is_dark);

	let to_svg_points = |pts: &[Point]| {
        pts.iter()
            .map(|p| format!("{},{}", p.x, p.y))
            .collect::<Vec<_>>()
            .join(" ")
    };

    let labels = LABELS();
    let z = zoom();

    // Scale node radius inversely with zoom so nodes stay same visual size when zoomed
    // clamp() ensures nodes don't get too small or too big
    let node_r = (6.0 / z).clamp(0.75, 18.0);
    let stroke_w = (1.0 / z).clamp(0.3, 2.0);
    let selected_stroke_w = (1.5 / z).clamp(0.5, 2.0); // Thicker stroke when selected
    let node_stroke_color = if is_dark { "white" } else { "black" };


    // Get color from label_id
    let get_color = |label_id: &str| -> String {
        labels.iter()
            .find(|l| l.label_id == label_id)
            .map(|l| l.label_color.clone())
            .unwrap_or_else(|| "#22C55E".to_string())
    };

    // Convert color to rgba with opacity (handles both hex and rgb formats)
    let color_with_opacity = |color: &str, opacity: f64| -> String {
        if color.starts_with("rgb(") {
            // rgb(0,122,255) -> rgba(0,122,255,0.2)
            color.replace("rgb(", "rgba(").replace(")", &format!(",{})" , opacity))
        } else if color.starts_with("#") && color.len() == 7 {
            // #RRGGBB -> rgba(r,g,b,opacity)
            let r = u8::from_str_radix(&color[1..3], 16).unwrap_or(0);
            let g = u8::from_str_radix(&color[3..5], 16).unwrap_or(0);
            let b = u8::from_str_radix(&color[5..7], 16).unwrap_or(0);
            format!("rgba({},{},{},{})" , r, g, b, opacity)
        } else {
            format!("rgba(0,0,0,{})" , opacity)
        }
    };


	// Calculate polygon area using shoelace formula
	let polygon_area = |points: &[Point]| -> f64 {
		if points.len() < 3 { return 0.0; }
		let mut area = 0.0;
		for i in 0..points.len() {
			let j = (i + 1) % points.len();
			area += points[i].x * points[j].y;
			area -= points[j].x * points[i].y;
		}
		(area / 2.0).abs()
	};

	// Sort: largest first (renders below), smallest last (renders on top)
	let mut sorted_anns: Vec<_> = annotations().into_iter()
		.filter(|a| !hidden_label_ids.contains(&a.label_id))
		.collect();
	sorted_anns.sort_by(|a, b| {
		let area_a = match &a.geometry { Geometry::Polygon { points } => polygon_area(points), _ => 0.0 };
		let area_b = match &b.geometry { Geometry::Polygon { points } => polygon_area(points), _ => 0.0 };
		area_b.partial_cmp(&area_a).unwrap_or(std::cmp::Ordering::Equal)
	});

	let mut hovered_ann_id: Signal<Option<String>> = use_signal(|| None);

	rsx!{
		// Loop through sorted annotations (largest first, smallest on top)
	    for ann in sorted_anns.into_iter() {{

	    	// Clone IDs because we need them in multiple closures below
			let ann_id = ann.id.clone();
			let label_id = ann.label_id.clone();
			let color = get_color(&ann.label_id);

			// Check if THIS annotation is the selected or hovered one
            let is_selected = ann_id == selected_ann_id();
            let is_hovered = hovered_ann_id() == Some(ann_id.clone()) && !is_selected;

            // Extract points from geometry (we only handle Polygon for now)
            let points: Vec<Point> = match &ann.geometry {
                Geometry::Polygon { points } => points.clone(),
                Geometry::BBox { start, end } => {
                    vec![
                        Point { x: start.x, y: start.y },
                        Point { x: end.x, y: start.y },
                        Point { x: end.x, y: end.y },
                        Point { x: start.x, y: end.y },
                    ]
                }
            };

             // Convert points to SVG format: "x1,y1 x2,y2 x3,y3"
            let svg_pts = points.iter()
                .map(|p| format!("{},{}", p.x, p.y))
                .collect::<Vec<_>>()
                .join(" ");


			rsx!{
				// Wrap annotation in <g> so hover is stable across polygon + nodes
				g {
					onmouseenter: {
						let lid = label_id.clone();
						let aid = ann_id.clone();
						move |_| {
							hovered_label_id.set(Some(lid.clone()));
							hovered_ann_id.set(Some(aid.clone()));
							hovered_ann_id_out.set(Some(aid.clone()));
						}
					},
					onmouseleave: move |_| {
						hovered_label_id.set(None);
						hovered_ann_id.set(None);
						// Keep hovered_ann_id_out so copy retains last-hovered annotation
					},

				// ═══════════════════════════════════════════════════════════
                // POLYGON SHAPE (the filled annotation)
                // ═══════════════════════════════════════════════════════════
				{{
					let selected_opacity = (annotation_opacity + 0.2).min(1.0);
					let hovered_opacity = (annotation_opacity + 0.15).min(1.0);
					let fill_color = if is_selected { color_with_opacity(&color, selected_opacity) } else if is_hovered { color_with_opacity(&color, hovered_opacity) } else { color_with_opacity(&color, annotation_opacity) };
					// Wider invisible hit area for edges (3px buffer for adding nodes)
					let hit_stroke_w = (6.0 / z).clamp(3.0, 12.0);
					let aid_hit = ann_id.clone();
					let mut annotations_hit = annotations;
					rsx! {
						// Visible polygon first (bottom)
						polygon {
							key:"{ann_id}",
							points: "{svg_pts}",
							fill: "{fill_color}",
					// Selected = label color border
                    stroke: "{color}",
					// Selected/hovered = thicker stroke
                    stroke_width: if is_selected || is_hovered { "2.5" } else { "1.5" },
					vector_effect:"non-scaling-stroke",
					pointer_events: if selected_tool == Tool::Comment { "none" } else { "auto" },
					cursor: "default",


                    // CLICK: Select this annotation
                    onclick: {
                        let aid = ann_id.clone();
                        move |evt: MouseEvent| {
                            evt.stop_propagation(); // Don't let click bubble to background
                            selected_ann_id.set(aid.clone()); // Mark this annotation as selected
                        }
                    },

					// Bubble up to parent SVG selection is handled by onclick
                    onmousedown: move |evt: MouseEvent| { 
                    	let mods = evt.data.modifiers();
                    	let is_middle = evt.data.trigger_button() == Some(dioxus::html::input_data::MouseButton::Auxiliary);
                    	           	 
					    // Cmd/Ctrl+click = adding node, don't start pan
					    if mods.meta() || mods.ctrl() {
					        evt.prevent_default();
					        evt.stop_propagation();
					        return;
					    }

					    // Shift or middle mouse = let it bubble for panning
    					// Regular click = also bubble (selection handled by onclick)
                    },

                	onmouseup:  { 
                		let aid = ann_id.clone();
                		let img_id = image_id.clone();
                		let mut annotations_sig = annotations;
                		

                		move |evt:MouseEvent| {
                			
                			if skip_polygon_insert() { // We need to skip polygon adding nodes in svg mouse up cmd+click
                                skip_polygon_insert.set(false);
                                return;
                            }
					        // Require Cmd (macOS) or Ctrl (Windows/Linux)
					        let mods = evt.data.modifiers();
					        if !(mods.meta() || mods.ctrl()) {
					            return; // let the SVG see mouseup so it can exit pan mode
					        }

					        evt.prevent_default();
                            evt.stop_propagation();

					        // Only add to the currently selected annotation
					        if selected_ann_id() != aid {
					            return;
					        }

					        // World-space click position
					        let (wx, wy) = cursor_world();

					        // Insert point into nearest edge, then persist
        					let mut anns = annotations_sig.write();
        					if let Some(ann) = anns.iter_mut().find(|a| a.id == aid) {
					            if let Geometry::Polygon { points } = &mut ann.geometry {
					                insert_point_on_polygon(points, wx, wy);
					                dirty_ann_ids().insert(aid.to_string());
					                last_dirty_tick.set(Instant::now());
					            }
        					};

                		}
                	},
					
					// RIGHT-CLICK: Show context menu (delete, change label)
                    oncontextmenu: {
                        let aid = ann_id.clone();
                        let lid = label_id.clone();
                        move |evt: MouseEvent| {
                            evt.prevent_default();  // Prevent browser context menu
                            evt.stop_propagation();
                            let p = evt.client_coordinates(); // Screen position for menu
                            on_annotation_context_menu.call((aid.clone(), lid.clone(), p.x, p.y));
                        }
                    }
							}
						// Invisible wider stroke for edge hit detection (on top for clicks)
						if is_selected {
							polygon {
								key: "hit-{ann_id}",
								points: "{svg_pts}",
								fill: "none",
								stroke: "transparent",
								stroke_width: "{hit_stroke_w}",
								pointer_events: "stroke",
								cursor: "crosshair",
								onmousedown: move |evt: MouseEvent| {
									let mods = evt.data.modifiers();
									if mods.meta() || mods.ctrl() {
										evt.prevent_default();
										evt.stop_propagation();
									}
								},
								onmouseup: move |evt: MouseEvent| {
									if skip_polygon_insert() {
										skip_polygon_insert.set(false);
										return;
									}
									let mods = evt.data.modifiers();
									if !(mods.meta() || mods.ctrl()) { return; }
									evt.prevent_default();
									evt.stop_propagation();
									if selected_ann_id() != aid_hit { return; }
									let (wx, wy) = cursor_world();
									let mut anns = annotations_hit.write();
									if let Some(ann) = anns.iter_mut().find(|a| a.id == aid_hit) {
										if let Geometry::Polygon { points } = &mut ann.geometry {
											insert_point_on_polygon(points, wx, wy);
											dirty_ann_ids.write().insert(aid_hit.to_string());
											last_dirty_tick.set(Instant::now());
										}
									}
								},
							}
						}
					}
				}}

				 // Label name tooltip on hover
                if is_hovered {
                    {{
                        // Position tooltip to the right of the bounding box, vertically centered
                        let max_x = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
                        let min_y = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
                        let max_y = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
                        let mid_y = (min_y + max_y) / 2.0;
                        let gap = 6.0 / z;
                        let tx = max_x + gap;
                        let label_name = labels.iter()
                            .find(|l| l.label_id == label_id)
                            .map(|l| l.label_name.clone())
                            .unwrap_or_default();
                        let font_size = 16.0 / z;
                        let pad_x = 4.0 / z;
                        let pad_y = 3.0 / z;
                        let bg_rx = 3.0 / z;
                        let text_w = label_name.len() as f64 * font_size * 0.6;
                        let bg_w = text_w + pad_x * 2.0;
                        let bg_h = font_size + pad_y * 2.0;
                        rsx! {
                            rect {
                                x: "{tx}",
                                y: "{mid_y - bg_h / 2.0}",
                                width: "{bg_w}",
                                height: "{bg_h}",
                                rx: "{bg_rx}",
                                fill: "rgba(0,0,0,0.75)",
                                pointer_events: "none",
                            }
                            text {
                                x: "{tx + bg_w / 2.0}",
                                y: "{mid_y}",
                                text_anchor: "middle",
                                dominant_baseline: "central",
                                fill: "white",
                                font_size: "{font_size}",
                                font_family: "Helvetica Neue, Helvetica, Arial, sans-serif",
                                font_weight: "500",
                                pointer_events: "none",
                                "{label_name}"
                            }
                        }
                    }}
                }

				// Draggable nodes (selected or hovered)
                if is_selected || is_hovered {
                    for (i, pt) in points.iter().enumerate() {
                        DraggableNode {
                        	block_id: block_id.clone(),
                            ann_id: ann_id.clone(),
                            image_id: image_id.clone(),
                            node_index: i,
                            x: pt.x,
                            y: pt.y,
                            radius: node_r,
                            stroke_width: stroke_w,
                            annotations: annotations,
                            dragging_node: dragging_node,
                            cursor_world: cursor_world,
                            node_color: color.clone(),
                            dirty_ann_ids: dirty_ann_ids,
    						last_dirty_tick: last_dirty_tick,  
    						skip_polygon_insert: skip_polygon_insert,
                            selected_ann_id: selected_ann_id,
                        }
                    }
                }


			} // close g
			} // close rsx!

		}}
	}
}


// ═══════════════════════════════════════════════════════════════════════════
// DRAGGABLE NODE - A single vertex circle that can be dragged to reshape polygon
// | Click on | Handler fires | Action |
// |----------|---------------|--------|
// | Node circle (●) | DraggableNode.onmousedown | Cmd+click = DELETE node |
// | Polygon edge/fill | SavedAnnotation polygon.onmouseup | Cmd+click = ADD node |
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn DraggableNode(
	block_id:String,
    ann_id: String,           // Which annotation this node belongs to
    image_id: String,         // For saving to backend
    node_index: usize,        // Current node index - Which point in the polygon (0, 1, 2, ...)
    x: f64,                   // Current x position
    y: f64,                   // Current y position
    radius: f64,              // Circle radius (scaled by zoom)
    stroke_width: f64,        // Border thickness (scaled by zoom)
    annotations: Signal<Vec<Annotation>>,
    dragging_node: Signal<i32>,
    cursor_world: Signal<(f64, f64)>,
    node_color: String,
    dirty_ann_ids: Signal<HashSet<String>>,  
    last_dirty_tick: Signal<Instant>, 
    skip_polygon_insert: Signal<bool>,
    selected_ann_id: Signal<String>,
) -> Element {
		let mut is_hovered = use_signal(||false);
		
		// Node fill is solid label color, stroke is always white
		let node_stroke = "white";
	
		 rsx! {
	        circle {
	            key: "node-{ann_id}-{node_index}",
	            cx: "{x}",
	            cy: "{y}",
	            r: "{radius}",
	            fill: "{node_color}",
	            stroke: "{node_stroke}",
	            stroke_width: "{stroke_width}",
	            cursor: "default",
	            pointer_events: "auto",

	            // Only set which node we're dragging - actual movement handled at SVG level
	            onmousedown: {
	                let idx = node_index as i32;
	                let ann_id = ann_id.clone();
	                move |evt: MouseEvent| {
	                    evt.prevent_default();
                    evt.stop_propagation();
                    // Ensure this annotation is the active selection before dragging
                    selected_ann_id.set(ann_id.clone());

	                let mods = evt.data.modifiers();
                    if mods.meta() || mods.ctrl() {
                        let mut anns = annotations.write();
                        let mut delete_ann_id: Option<String> = None;
                        if let Some(ann) = anns.iter_mut().find(|a| a.id == ann_id) {
                            if let Geometry::Polygon { points } = &mut ann.geometry {
                                if (idx as usize) < points.len() {
                                    tracing::info!("🗑️ Deleting node {}", idx);
                                    points.remove(idx as usize);
                                    skip_polygon_insert.set(true);

                                    if points.len() < 3 {
                                    	delete_ann_id = Some(ann_id.clone());
                                    	info!("dele_ann_id = {:?}", Some(&delete_ann_id));
                                    } else {
                                        dirty_ann_ids.write().insert(ann_id.clone());
                                        last_dirty_tick.set(Instant::now());
                                    }
                                }
                            }
                        }
                        drop(anns); //** release borrow before touching annotations again ** crucial ** 

                        if let Some(aid) = delete_ann_id {
                        	// info!("should be coming here now : {}", aid);
                        	// annotations.write().retain(|a| a.id != aid);
                        	let img_id = image_id.clone();
                        	let block_id1 = block_id.clone();
                        	state_delete_annotation(&block_id1, &img_id, &aid, annotations);
                        }

                        return;
                    }

	                    dragging_node.set(idx); // Sets the current node to index and is used by parent SVG mouse handlers
	                }
	            },

	            onmouseenter:move |_| {
	            	is_hovered.set(true);
	            },
	            
	            onmouseleave:move |_| {
	            	is_hovered.set(false);
	            }


	        }
	    }
	}



