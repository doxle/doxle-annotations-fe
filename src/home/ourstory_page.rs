use dioxus::prelude::*;

#[component]
pub fn OurStoryPage() -> Element {
    rsx! {
        div { 
            class: "ourstory-container",
            div { 
                class: "ourstory-content",
                h1 { 
                    class: "ourstory-title",
                    "Our Story" 
                }
                div {
                    class: "ourstory-text-container",
                    p {
                        class: "ourstory-paragraph",
                        "Welcome to Doxle. We're building something amazing."
                    }
                    p {
                        class: "ourstory-paragraph",
                        "Our mission is to revolutionize the way you work with documents and data."
                    }
                    p {
                        class: "ourstory-paragraph",
                        "Stay tuned for more updates as we continue to grow and evolve."
                    }
                }
            }
        }
    }
}
