use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlElement};

#[component]
pub fn MoreMenu(is_open: Signal<bool>) -> Element {
    let toggle_theme = move |_| {
        if let Some(window) = window() {
            if let Some(document) = window.document() {
                if let Some(html) = document.document_element() {
                    if let Ok(html_el) = html.dyn_into::<HtmlElement>() {
                        // Get current class name
                        let current_class = html_el.class_name();

                        // Toggle between dark and light
                        if current_class.contains("dark") {
                            html_el.set_class_name("light");
                        } else {
                            html_el.set_class_name("dark");
                        }
                    }
                }
            }
        }
        is_open.set(false);
    };

    rsx! {
        if is_open() {
        div {
            class: "more-dropdown-menu",
            onclick: move |e| {
                e.stop_propagation();
            },

            button {
                class: "more-dropdown-item",
                onclick: toggle_theme,
                "Change Theme"
            }

            button {
                class: "more-dropdown-item",
                onclick: move |_| {
                    // TODO: Button 1 action
                    is_open.set(false);
                },
                "Button 1"
            }

            button {
                class: "more-dropdown-item",
                onclick: move |_| {
                    // TODO: Button 2 action
                    is_open.set(false);
                },
                "Button 2"
            }

            button {
                class: "more-dropdown-item",
                onclick: move |_| {
                    // TODO: Button 3 action
                    is_open.set(false);
                },
                "Button 3"
            }
        }
        }
    }
}
