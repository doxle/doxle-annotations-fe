use dioxus::prelude::*;

#[component]
pub fn CommentsPanel() -> Element {
    rsx! {
        div { class: "sidebar-comments",
            p { class: "sidebar-muted", "Comments coming soon." }
        }
    }
}
