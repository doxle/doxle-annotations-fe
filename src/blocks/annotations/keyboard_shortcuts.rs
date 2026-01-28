use dioxus::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast};
use crate::atoms::svg_canvas::state::Tool;


pub fn setup_keyboard_shortcuts(
	mut sidebar_open: Signal<bool>,
	mut grid_visible: Signal<bool>,
	mut selected_tool: Signal<Tool>,
	mut active_drawing: Signal<Vec<(f64,f64)>>,
)
{
	use_effect(move || {
		let Some(win) = web_sys::window() else {return};
		let closure = Closure::wrap(Box::new(move|evt:web_sys::KeyboardEvent|{
			let key = evt.key();
			// Cmd+\ (Mac) or Ctrl+\ (Windows/Linux) - toggle sidebar
			if key == "\\" && (evt.meta_key() || evt.ctrl_key()){
				evt.prevent_default();
				sidebar_open.set(!sidebar_open());
			}
			// g - toggle grid/dots
			if key == "g" || key == "G" {
				grid_visible.set(!grid_visible());
			}
			// p - enable polygon mode
			if key == "p" || key == "P" {
				selected_tool.set(Tool::Polygon);
			}
			// Escape - disable polygon mode, clear drawing
			if key == "Escape" {
				selected_tool.set(Tool::Pan);
				active_drawing.write().clear();
			}
			if key == " " {
				tracing::info!("Shift key is pressed");
			}
		}) as Box<dyn FnMut(_)>);
		let _ = win.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
		closure.forget()
	});
}
