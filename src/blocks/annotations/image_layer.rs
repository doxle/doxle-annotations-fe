use dioxus::prelude::*;

#[component]
pub fn ImageLayer(src: String, image_size:Signal<(f64,f64)>) -> Element {
    rsx! {
        img {
            class: "canvas-bg-image",
            src: "{src}",
            style: "position: absolute; top: 0; left: 0; pointer-events: none; user-select: none; -webkit-user-select: none;",
            draggable: "false",
            onload: move|_|{
                #[cfg(target_arch="wasm32")]{
                    use wasm_bindgen::JsCast;
                    if let Some(win) = web_sys::window(){
                        if let Some(doc) = win.document() {
                            if let Ok(Some(img)) = doc.query_selector(".canvas-bg-image"){
                                if let Ok(img) = img.dyn_into::<web_sys::HtmlImageElement>(){
                                    // Use rendered size (getBoundingClientRect), not natural size
                                    let rect = img.get_bounding_client_rect();
                                    let w = rect.width();
                                    let h = rect.height();
                                    image_size.set((w,h));
                                }
                            }
                        }
                    }

                }
            }
        }
    }
}
