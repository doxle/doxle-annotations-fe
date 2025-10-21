use dioxus::prelude::*;

#[component]
pub fn SidebarHeader(block_label: String, images_count: u32) -> Element {
    rsx! {
        div { class: "sidebar-header",
            div { class: "sidebar-block-title", "{block_label} - {images_count} Images" }
        }
    }
}
