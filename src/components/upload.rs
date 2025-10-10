use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn UploadPage() -> Element {
    rsx! {
        div { class: "min-h-screen bg-gray-900 flex items-center justify-center px-4",
            h1 {"Upload Page"}
        }
    }
}
