/*
Event flow (concise)
•  Mouse move
◦  EventLayer: p = element_coordinates() [screen]
◦  cursor_pos := (sx, sy)
◦  (wx, wy) = Transform.screen_to_world(sx, sy)
◦  cursor_world_pos := (wx, wy)
◦  AnnotationsLayer re-renders → draws dashed line last_point → cursor_world_pos
•  Mouse down (Polygon tool)
◦  EventLayer: (wx, wy) from screen_to_world
◦  active_drawing.push((wx, wy))
◦  AnnotationsLayer re-renders → polygon(points = active_drawing), circles at each vertex
•  Image load & sizing
◦  ImageLayer onload → image_size := (natural_w, natural_h)
◦  AnnotationsLayer uses width/height to set SVG size to match image
•  Center on first render
◦  SvgCanvas effect measures viewport & img DOM rects
◦  pan_x = (vw - iw)/2; pan_y = (vh - ih)/2; zoom = 1
◦  Transform set_view(zoom, pan_x, pan_y)
◦  world-container opacity toggles to avoid flicker

*/





use dioxus::prelude::*;
use super::state::{CanvasState, Tool};
use dioxus::html::input_data::MouseButton;

#[component]
pub fn EventLayer(
    mut is_dragging: Signal<bool>,
    mut last_mouse_pos: Signal<(f64, f64)>,
    mut cursor_pos: Signal<(f64, f64)>,
    mut cursor_inside: Signal<bool>,
    mut active_drawing:Signal<Vec<(f64,f64)>>,
    mut cursor_world_pos: Signal<(f64,f64)>,
    selected_tool: Tool,
) -> Element {
    let mut state: Signal<CanvasState> = use_context();


    rsx! {
        div {
            class: "event-layer",
            style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; z-index: 100;",

            onmousedown: move |evt| {
                let is_left = evt.data.trigger_button() == Some(MouseButton::Primary);
                let is_middle = evt.data.trigger_button() == Some(MouseButton::Auxiliary);
                if !is_left && !is_middle { return; } // Not left or middle return 

                let p = evt.element_coordinates();
                let shift_or_cmd = evt.data.modifiers().shift() || evt.data.modifiers().meta();

                // Shift/Cmd + drag OR middle click = pan (even in polygon mode)
                if shift_or_cmd ||is_middle {
                    is_dragging.set(true);
                    last_mouse_pos.set((p.x, p.y));
                    return;
                }

                // Polygon tool: add point
                if selected_tool == Tool::Polygon && is_left {                    
                     let transform = *state.read().transform.read();
                     let (wx,wy) = transform.screen_to_world(p.x, p.y);
                     tracing::info!("Adding point: ({}, {})", wx, wy);
                     active_drawing.write().push((wx,wy));

                }
                else{
                     // Start Pan drag
                    is_dragging.set(true);
                    last_mouse_pos.set((p.x, p.y));

                }              
            },

            onmousemove: move |evt| {
                let p = evt.element_coordinates();
                cursor_pos.set((p.x, p.y));
                cursor_inside.set(true);

                let transform = *state.read().transform.read();
                let (wx,wy) = transform.screen_to_world(p.x,p.y);
                cursor_world_pos.set((wx,wy));

                if !is_dragging() { return; }
                let (lx, ly) = last_mouse_pos();
                state.write().pan(p.x - lx, p.y - ly);
                last_mouse_pos.set((p.x, p.y));
            },

            onmouseup: move |_| { is_dragging.set(false); },
            onmouseleave: move |_| { is_dragging.set(false); cursor_inside.set(false); },

            onwheel: move |evt| {
                evt.prevent_default();
                evt.stop_propagation();

                let dy = evt.data.delta().strip_units().y;
                let p = evt.element_coordinates();
                let factor = (-dy * 0.001).exp().clamp(0.9, 1.1);
                state.write().zoom(factor, p.x, p.y);
                
                
            },
        }
    }
}
