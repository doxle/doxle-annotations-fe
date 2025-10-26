use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    pub fn to_class(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => {
                #[cfg(target_arch = "wasm32")]
                {
                    if system_prefers_dark() {
                        "dark"
                    } else {
                        "light"
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                "light"
            }
        }
    }

    pub fn cycle(&self) -> Theme {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::System,
            Theme::System => Theme::Light,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
            Theme::System => "System",
        }
    }
}

// Global theme signal
pub const THEME: GlobalSignal<Theme> = Signal::global(|| Theme::Light);

// Check system preference
#[cfg(target_arch = "wasm32")]
pub fn system_prefers_dark() -> bool {
    use wasm_bindgen::JsCast;
    use web_sys::window;

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

// localStorage helpers
#[cfg(target_arch = "wasm32")]
pub fn save_theme_preference(theme: Theme) {
    use web_sys::window;

    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let value = match theme {
                Theme::Light => "light",
                Theme::Dark => "dark",
                Theme::System => "system",
            };
            let _ = storage.set_item("doxle_theme", value);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_theme_preference(_theme: Theme) {}

#[cfg(target_arch = "wasm32")]
pub fn load_theme_preference() -> Option<Theme> {
    use web_sys::window;

    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(value)) = storage.get_item("doxle_theme") {
                return match value.as_str() {
                    "light" => Some(Theme::Light),
                    "dark" => Some(Theme::Dark),
                    "system" => Some(Theme::System),
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

// Apply theme to HTML element
#[cfg(target_arch = "wasm32")]
pub fn apply_theme_class(theme: Theme) {
    use wasm_bindgen::JsCast;
    use web_sys::{window, HtmlElement};

    if let Some(window) = window() {
        if let Some(document) = window.document() {
            if let Some(html) = document.document_element() {
                if let Ok(html_el) = html.dyn_into::<HtmlElement>() {
                    html_el.set_class_name(theme.to_class());
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn apply_theme_class(_theme: Theme) {}
