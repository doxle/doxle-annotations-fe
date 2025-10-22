use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlElement};

#[component]
pub fn AvatarDropdown(is_open: Signal<bool>, show_grid_lines: Signal<bool>) -> Element {
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
                        } else if current_class.contains("light") {
                            html_el.set_class_name("dark");
                        } else {
                            // If no theme class yet, initialize to the opposite of system preference
                            let prefers_dark = window.match_media("(prefers-color-scheme: dark)")
                                .ok()
                                .flatten()
                                .map(|m| m.matches())
                                .unwrap_or(false);
                            if prefers_dark {
                                html_el.set_class_name("light");
                            } else {
                                html_el.set_class_name("dark");
                            }
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
            class: "avatar-dropdown-menu",
            onclick: move |e| {
                e.stop_propagation();
            },

            button {
                class: "avatar-dropdown-item",
                onclick: toggle_theme,
                "Change Theme"
            }

            button {
                class: "avatar-dropdown-item",
                onclick: move |_| {
                    show_grid_lines.set(!show_grid_lines());
                    is_open.set(false);
                },
                if show_grid_lines() {
                    "Hide Grid Lines"
                } else {
                    "Show Grid Lines"
                }
            }

            button {
                class: "avatar-dropdown-item",
                onclick: move |_| {
                    // TODO: Button 2 action
                    is_open.set(false);
                },
                "Button 2"
            }

            button {
                class: "avatar-dropdown-item",
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
