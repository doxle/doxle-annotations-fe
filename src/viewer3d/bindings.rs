use js_sys::{Function, Reflect};
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;

fn get_viewer_fn(name: &str) -> Option<Function> {
    let window = web_sys::window()?;
    let viewer = Reflect::get(&window, &JsValue::from_str("doxleViewer")).ok()?;
    let func = Reflect::get(&viewer, &JsValue::from_str(name)).ok()?;
    func.dyn_into::<Function>().ok()
}

pub fn viewer_init(container_id: &str) {
    if let Some(func) = get_viewer_fn("initViewer") {
        let _ = func.call1(&JsValue::NULL, &JsValue::from_str(container_id));
    }
}

pub fn viewer_add_wall(json: &str) {
    if let Some(func) = get_viewer_fn("addWall") {
        let _ = func.call1(&JsValue::NULL, &JsValue::from_str(json));
    }
}

pub fn viewer_clear_scene() {
    if let Some(func) = get_viewer_fn("clearScene") {
        let _ = func.call0(&JsValue::NULL);
    }
}

pub fn viewer_dispose() {
    if let Some(func) = get_viewer_fn("disposeViewer") {
        let _ = func.call0(&JsValue::NULL);
    }
}
