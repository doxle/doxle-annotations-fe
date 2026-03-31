use std::collections::HashSet;
use std::cell::RefCell;
use std::rc::Rc;
use dioxus::prelude::*;
use dioxus::dioxus_core::use_drop;
use wasm_bindgen::{closure::Closure, JsCast};
use crate::core::svg_canvas::state::Tool;
use crate::blocks::state::LABELS;
use crate::core::app_sidebar::SidebarTab;


pub fn setup_keyboard_shortcuts(
	mut sidebar_open: Signal<bool>,
	mut grid_visible: Signal<bool>,
	mut selected_tool: Signal<Tool>,
	mut active_drawing: Signal<Vec<(f64,f64)>>,
	mut hidden_label_ids: Signal<HashSet<String>>,
	hovered_label_id: Signal<Option<String>>,
	mut show_shortcuts: Signal<bool>,
	mut sidebar_tab: Signal<SidebarTab>,
	mut nav_direction: Signal<Option<i32>>,
	mut copy_requested: Signal<u32>,
	mut paste_mode: Signal<bool>,
)
{
	let keydown_listener: Rc<RefCell<Option<Closure<dyn FnMut(web_sys::KeyboardEvent)>>>> =
		use_hook(|| Rc::new(RefCell::new(None)));

	use_effect({
		let keydown_listener = keydown_listener.clone();
		move || {
		let Some(win) = web_sys::window() else {return};
		let last_key: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
		let lk = last_key.clone();
		
		let closure = Closure::wrap(Box::new(move|evt:web_sys::KeyboardEvent|{
			// Ignore shortcuts when typing in input fields
			if let Some(target) = evt.target() {
				if let Ok(element) = target.dyn_into::<web_sys::HtmlElement>() {
					let tag_name = element.tag_name().to_lowercase();
					if tag_name == "input" || tag_name == "textarea" || tag_name == "select" || element.is_content_editable() {
						return;
					}
				}
			}
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
				sidebar_tab.set(SidebarTab::Labels);
			}
			// b - enable bbox mode
			if key == "b" || key == "B" {
				selected_tool.set(Tool::BBox);
				sidebar_tab.set(SidebarTab::Labels);
			}
			// Ctrl+C - copy selected annotation
			if (key == "c" || key == "C") && (evt.ctrl_key() || evt.meta_key()) {
				evt.prevent_default();
				copy_requested.set(copy_requested() + 1);
			}
			// Ctrl+V - paste copied annotation
			else if (key == "v" || key == "V") && (evt.ctrl_key() || evt.meta_key()) {
				evt.prevent_default();
				paste_mode.set(true);
			}
			// c - enable comment mode + switch sidebar to Comments (skip if Ctrl/Cmd held)
			else if key == "c" || key == "C" {
				selected_tool.set(Tool::Comment);
				sidebar_open.set(true);
				sidebar_tab.set(SidebarTab::Comments);
			}
			// Escape - return to arrow/select mode, clear drawing, exit paste mode
			if key == "Escape" {
				selected_tool.set(Tool::Select);
				active_drawing.write().clear();
				sidebar_tab.set(SidebarTab::Labels);
				paste_mode.set(false);
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
			// f - forward (next image) — skip if Ctrl/Cmd held (browser find)
			if (key == "f" || key == "F") && !evt.ctrl_key() && !evt.meta_key() {
				nav_direction.set(Some(1));
			}
			// d - backward (previous image)
			if key == "d" || key == "D" {
				nav_direction.set(Some(-1));
			}
			// Space - toggle shortcuts help
			if key == " " {
				evt.prevent_default();
				show_shortcuts.set(!show_shortcuts());
			}
			*lk.borrow_mut() = key;
		}) as Box<dyn FnMut(_)>);

		if let Some(old) = keydown_listener.borrow_mut().take() {
			let _ = win.remove_event_listener_with_callback("keydown", old.as_ref().unchecked_ref());
		}
		let _ = win.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
		*keydown_listener.borrow_mut() = Some(closure);
	}});

	use_drop({
		let keydown_listener = keydown_listener.clone();
		move || {
			if let Some(win) = web_sys::window() {
				if let Some(closure) = keydown_listener.borrow().as_ref() {
					let _ = win.remove_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
				}
			}
			keydown_listener.borrow_mut().take();
		}
	});
}

