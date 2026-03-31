use dioxus::prelude::*;

#[component]
pub fn BottomBar() -> Element {
    rsx! {
        div { class: "app-bottom-bar",
            "Bottom Bar Placeholder"
        }
    }
}
