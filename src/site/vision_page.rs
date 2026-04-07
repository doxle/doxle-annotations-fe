use dioxus::prelude::*;

#[component]
pub fn VisionPage() -> Element {
    rsx! {
        div {
            class: "vision-container",
            div {
                class: "vision-card",
                h1 {
                    class: "vision-title",
                    "Vision"
                }
            }
        }
    }
}
