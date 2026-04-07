use dioxus::prelude::*;

/// Reactive signal that tracks whether the viewport is mobile-sized.
/// Updates on window resize. Use `is_mobile()` to read the current value.
pub static IS_MOBILE: GlobalSignal<bool> = Signal::global(|| {
    check_mobile()
});

/// One-shot check: viewport width <= 768px
pub fn check_mobile() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(width) = window.inner_width() {
                if let Some(w) = width.as_f64() {
                    return w <= 768.0;
                }
            }
        }
    }
    false
}

/// Read the current mobile state from the reactive signal.
pub fn is_mobile() -> bool {
    *IS_MOBILE.read()
}

/// Call once from App to keep IS_MOBILE in sync with viewport resizes.
pub fn use_mobile_listener() {
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::prelude::*;
            use wasm_bindgen::JsCast;

            if let Some(window) = web_sys::window() {
                // Set initial value
                *IS_MOBILE.write() = check_mobile();

                // Listen for resize
                let closure = Closure::wrap(Box::new(move || {
                    *IS_MOBILE.write() = check_mobile();
                }) as Box<dyn Fn()>);

                let _ = window.add_event_listener_with_callback(
                    "resize",
                    closure.as_ref().unchecked_ref(),
                );
                closure.forget(); // keep alive for app lifetime
            }
        }
    });
}
