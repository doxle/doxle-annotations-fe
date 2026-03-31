use dioxus::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{window, KeyboardEvent};
use crate::core::theme::toggle_theme;

/// Sets up global keyboard shortcuts that work on all pages
pub fn setup_global_keyboard_shortcuts() {
    use_effect(move || {
        let win = match window() {
            Some(w) => w,
            None => return,
        };

        let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            // Ignore shortcuts when typing in input fields
            if let Some(target) = event.target() {
                if let Ok(element) = target.dyn_into::<web_sys::HtmlElement>() {
                    let tag_name = element.tag_name().to_lowercase();
                    if tag_name == "input" || tag_name == "textarea" || tag_name == "select" {
                        return;
                    }
                    if element.get_attribute("contenteditable").as_deref() == Some("true") {
                        return;
                    }
                }
            }

            let key = event.key();

            // Handle 't' key - toggle theme via global signal
            if key == "t" {
                toggle_theme();
            }
        }) as Box<dyn FnMut(_)>);

        let _ = win.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());

        // Keep closure alive
        closure.forget();
    });
}
