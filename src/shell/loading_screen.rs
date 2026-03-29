use dioxus::prelude::*;
use super::status_dialog::LiveTimer;
const LOADING_SCREEN_CSS: &str = include_str!("loading_screen.css");
const LOTTIE_DOG: Asset = asset!("/assets/animations/doxle-the-dog.json");

#[component]
pub fn LoadingScreen(text: String) -> Element {
    let lottie_src = LOTTIE_DOG;

    use_effect(move || {
        let src = lottie_src.to_string();
        let js = format!(
            r#"(function(){{var c=document.getElementById('loading-lottie');if(!c||!window.lottie)return;c.innerHTML='';window.lottie.loadAnimation({{container:c,renderer:'svg',loop:true,autoplay:true,path:'{}'}});}})();"#,
            src
        );
        document::eval(&js);
    });

    rsx! {
        style { {LOADING_SCREEN_CSS} }
        div { class: "loading-screen",
            div { class: "loading-screen-content",
                div { id: "loading-lottie", class: "loading-lottie" }
                LiveTimer {}
                span { "{text}" }
            }
        }
    }
}
