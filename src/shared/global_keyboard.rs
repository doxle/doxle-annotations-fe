use dioxus::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{window, HtmlElement, KeyboardEvent};

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
                }
            }

            let key = event.key();

            // Handle 't' key - toggle theme
            if key == "t" {
                if let Some(win) = window() {
                    if let Some(document) = win.document() {
                        if let Some(html) = document.document_element() {
                            if let Ok(html_el) = html.dyn_into::<HtmlElement>() {
                                let current_class = html_el.class_name();
                                if current_class.contains("dark") {
                                    html_el.set_class_name("light");
                                } else if current_class.contains("light") {
                                    html_el.set_class_name("dark");
                                } else {
                                    // Default to dark if no class
                                    html_el.set_class_name("dark");
                                }
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        let _ = win.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());

        // Keep closure alive
        closure.forget();
    });
}
