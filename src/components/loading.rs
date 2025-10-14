use dioxus::prelude::*;

#[component]
pub fn LoadingPage() -> Element {
    const LOADING_ANIMATION: Asset = asset!("/assets/images/dark-theme-animations.svg");

    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: #000000; display: flex; flex-direction: column; align-items: center; justify-content: center; z-index: 9999;",
            
            // SVG Animation
            img {
                src: "{LOADING_ANIMATION}",
                alt: "Loading",
                style: "width: 300px; height: auto;",
            }
            
            // Loading text
            div {
                style: "margin-top: 32px; color: #FFFFFF; font-size: 12px; font-weight: 400; font-family: 'Courier New', Courier, monospace;",
                "Loading..."
            }
        }
    }
}
