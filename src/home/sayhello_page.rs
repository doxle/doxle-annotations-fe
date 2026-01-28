use dioxus::prelude::*;

#[component]
pub fn SayHelloPage() -> Element {

    rsx! {
        div { 
            style: "min-height: 100vh; background: white; display: flex; align-items: center; justify-content: center; padding: 0 20px; font-family: Helvetica, Arial, sans-serif;",
            div { 
                style: "background: white; padding: 32px; width: 100%; max-width: 500px; text-align: center;",
               h1 {
                   style: "font-size: 32px; font-weight: 300; color: #333; font-family: Helvetica, Arial, sans-serif;",
                   "Say Hello"
               }
            }
        }
    }
}
