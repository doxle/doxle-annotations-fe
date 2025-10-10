use dioxus::prelude::*;

#[component]
pub fn OurStoryPage() -> Element {
    rsx! {
        div { 
            style: "min-height: 100vh; background: white; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 0 20px; font-family: Helvetica, Arial, sans-serif;",
            div { 
                style: "max-width: 900px; margin: 0 auto; padding: 64px 0;",
                h1 { 
                    style: "font-size: 48px; font-weight: 300; color: #333; margin-bottom: 32px; text-align: center; font-family: Helvetica, Arial, sans-serif;",
                    "Our Story" 
                }
                div {
                    style: "font-size: 18px; color: #666; font-weight: 300; text-align: center;",
                    p {
                        style: "margin-bottom: 24px;",
                        "Welcome to Doxle. We're building something amazing."
                    }
                    p {
                        style: "margin-bottom: 24px;",
                        "Our mission is to revolutionize the way you work with documents and data."
                    }
                    p {
                        style: "margin-bottom: 24px;",
                        "Stay tuned for more updates as we continue to grow and evolve."
                    }
                }
            }
        }
    }
}
