use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use { wasm_bindgen::{JsCast, JsValue},  web_sys::{window, HtmlElement} };


#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Theme {
    Light,
    Dark,
}



// Global theme signal - checks localStorage first, defaults to Light
pub const THEME: GlobalSignal<Theme> = Signal::global(|| {
    if let Some(saved) = load_theme_preference() {
        return saved;
    }
    Theme::Light
});


/// Check if current theme is dark
pub fn is_dark_theme() -> bool {
    *THEME.read() == Theme::Dark
}

#[cfg(target_arch = "wasm32")]
fn notify_native_theme(theme: Theme) {
    let Some(window) = window() else {
        return;
    };
    let theme_value = match theme {
        Theme::Light => "light",
        Theme::Dark => "dark",
    };

    let Ok(webkit) = js_sys::Reflect::get(&window, &JsValue::from_str("webkit")) else {
        return;
    };
    if webkit.is_null() || webkit.is_undefined() {
        return;
    }
    let Ok(message_handlers) = js_sys::Reflect::get(&webkit, &JsValue::from_str("messageHandlers")) else {
        return;
    };
    if message_handlers.is_null() || message_handlers.is_undefined() {
        return;
    }
    let Ok(theme_handler) = js_sys::Reflect::get(&message_handlers, &JsValue::from_str("themeChange")) else {
        return;
    };
    if theme_handler.is_null() || theme_handler.is_undefined() {
        return;
    }
    let Ok(post_message) = js_sys::Reflect::get(&theme_handler, &JsValue::from_str("postMessage")) else {
        return;
    };
    let Some(post_message_fn) = post_message.dyn_ref::<js_sys::Function>() else {
        return;
    };
    let _ = post_message_fn.call1(&theme_handler, &JsValue::from_str(theme_value));
}


/// Toggle between light and dark theme
pub fn toggle_theme(){
    let current = *THEME.read();

    *THEME.write() = match current {
        Theme::Light => Theme::Dark,
        Theme::Dark => Theme::Light,
    };
    apply_theme_class(*THEME.read());
    save_theme_preference(*THEME.read());
}


// Check system preference
#[cfg(target_arch = "wasm32")]
pub fn system_prefers_dark() -> bool {
    if let Some(window) = window() {
        if let Ok(media_query) = window.match_media("(prefers-color-scheme: dark)") {
            if let Some(mq) = media_query {
                return mq.matches();
            }
        }
    }
    false
}

#[cfg(not(target_arch = "wasm32"))]
pub fn system_prefers_dark() -> bool {
    false
}

/// Call once from App to sync theme with system prefers-color-scheme changes.
pub fn use_system_theme_listener() {
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::prelude::*;

            if let Some(window) = window() {
                if let Ok(Some(mq)) = window.match_media("(prefers-color-scheme: dark)") {
                    let closure = Closure::wrap(Box::new(move || {
                        let prefers_dark = system_prefers_dark();
                        let new_theme = if prefers_dark { Theme::Dark } else { Theme::Light };
                        *THEME.write() = new_theme;
                        apply_theme_class(new_theme);
                        save_theme_preference(new_theme);
                    }) as Box<dyn Fn()>);

                    let _ = mq.add_listener_with_opt_callback(
                        Some(closure.as_ref().unchecked_ref()),
                    );
                    closure.forget();
                }
            }
        }
    });
}

// localStorage helpers
#[cfg(target_arch = "wasm32")]
pub fn save_theme_preference(theme: Theme) {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let value = match theme {
                Theme::Light => "light",
                Theme::Dark => "dark",
            };
            let _ = storage.set_item("doxle_theme", value);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_theme_preference(_theme: Theme) {}


/// Load theme preference from localStorage
#[cfg(target_arch = "wasm32")]
pub fn load_theme_preference() -> Option<Theme> {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(value)) = storage.get_item("doxle_theme") {
                return match value.as_str() {
                    "light" => Some(Theme::Light),
                    "dark" => Some(Theme::Dark),
                    _ => None,
                };
            }
        }
    }
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_theme_preference() -> Option<Theme> {
    None
}


/// Apply theme class to HTML element
#[cfg(target_arch = "wasm32")]
pub fn apply_theme_class(theme: Theme) {
    if let Some(window) = window() {
        if let Some(document) = window.document() {
            if let Some(html) = document.document_element() {
                if let Ok(html_el) = html.dyn_into::<HtmlElement>() {
                    let class = match theme {
                        Theme::Light => "light",
                        Theme::Dark => "dark",
                    };
                    html_el.set_class_name(class);
                    notify_native_theme(theme);
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn apply_theme_class(_theme: Theme) {}
