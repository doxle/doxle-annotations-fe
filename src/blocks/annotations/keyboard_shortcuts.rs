use std::collections::HashSet;
use std::cell::RefCell;
use std::rc::Rc;
use dioxus::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast};
use crate::atoms::svg_canvas::state::Tool;
use crate::blocks::dashboard::state::LABELS;


pub fn setup_keyboard_shortcuts(
	mut sidebar_open: Signal<bool>,
	mut grid_visible: Signal<bool>,
	mut selected_tool: Signal<Tool>,
	mut active_drawing: Signal<Vec<(f64,f64)>>,
	mut hidden_label_ids: Signal<HashSet<String>>,
	hovered_label_id: Signal<Option<String>>,
	mut show_shortcuts: Signal<bool>,
)
{
	use_effect(move || {
		let Some(win) = web_sys::window() else {return};
		let last_key: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
		let lk = last_key.clone();
		
		let closure = Closure::wrap(Box::new(move|evt:web_sys::KeyboardEvent|{
			let key = evt.key();
			let prev = lk.borrow().clone();
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
			// "ha" - toggle hide/show ALL labels
			if (key == "a" || key == "A") && (prev == "h" || prev == "H") {
				let mut h = hidden_label_ids.write();
				if h.is_empty() {
					for l in LABELS().iter() { h.insert(l.label_id.clone()); }
				} else {
					h.clear();
				}
				*lk.borrow_mut() = String::new();
				return;
			}
			// h - toggle hide/show for hovered label
			if key == "h" || key == "H" {
				if let Some(label_id) = hovered_label_id() {
					let mut h = hidden_label_ids.write();
					if h.contains(&label_id) { h.remove(&label_id); }
					else { h.insert(label_id); }
				}
				*lk.borrow_mut() = key;
				return;
			}
			// Space - toggle shortcuts help
			if key == " " {
				evt.prevent_default();
				show_shortcuts.set(!show_shortcuts());
			}
			*lk.borrow_mut() = key;
		}) as Box<dyn FnMut(_)>);
		let _ = win.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
		closure.forget()
	});
}

