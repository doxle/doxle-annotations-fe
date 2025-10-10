use crate::Route;
use dioxus::prelude::*;

const LOGO: Asset = asset!("/assets/floorplan-dark.svg");
const DOTS_JS: &str = include_str!("../../js/dot-animation.js");

#[component]
pub fn HomePage() -> Element {
    let mut button_hover = use_signal(|| false);

    // Button style with hover effect
    let button_opacity = if button_hover() { 0.8 } else { 1.0 };
    let button_style = format!(
        "width: min(215px, 100%); height: 70px; display: flex;
        align-items: center; justify-content: space-between; padding: 0 20px;
        background-color: rgba(0, 0, 0, {}); color: white; border-radius: 0px;
        border: none; transition: background-color 0.05s; font-size: 18px;
        font-weight: 300 !important; font-family: HelveticaNeue, Helvetica, Arial, sans-serif;
        text-decoration: none; cursor: pointer;",
        button_opacity
    );

    // Inject dot animation JS on mount (include_str) so the browser executes it
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            if document.get_element_by_id("bg-dots-script").is_none() {
                let script = document.create_element("script").unwrap();
                script.set_id("bg-dots-script");
                script.set_attribute("type", "text/javascript").ok();
                script.set_text_content(Some(DOTS_JS));
                let _ = document.head().unwrap().append_child(&script);
            }
        }
    });

    rsx! {
        document::Style {
            r".dot {{
                position: absolute;
                width: 3px;
                height: 3px;
                background-color: rgba(149, 128, 255, 0.3);
                border-radius: 50%;
                will-change: transform;
                transform: translate3d(0,0,0);
                pointer-events: none;
                backface-visibility: hidden;
            }}"
        }
        div {
            style: "position: relative; display: flex; align-items: center; justify-content: center;
            width: 100%; min-height: 100vh; font-family: HelveticaNeue, Helvetica, Arial, sans-serif;
            background-color: rgb(247, 247, 247); overflow: hidden;",

            // Dots layer
            div {
                id: "dots-container",
                style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%;
                pointer-events: none; contain: layout paint style; transform: translateZ(0);",
            }

            // Content layer
            div {
                style: "position: relative; z-index: 10; text-align: left; max-width: 800px;
                margin: 0 auto; padding: 0 20px;",
                h1 {
                    style: "font-size: clamp(60px, 8vw, 90px); font-weight: 400; line-height: 1;
                    color:rgb(79,91,248); margin: 0;",
                    "BUILT WITH AI"
                }
                h2 {
                    style: "font-size: clamp(60px, 8vw, 90px); font-weight: 400; line-height: 1;
                    color:rgb(79,91,248); margin: 0;",
                    "SO YOU "
                    span {
                        style: "font-weight: 400; text-decoration: none;",
                        "BUILD"
                    }
                }
                h2 {
                    style: "font-size: clamp(60px, 8vw, 90px); font-weight: 400; line-height: 1;
                    color: rgb(79,91,248); margin: 0 0 24px 0;",
                    "WITH AI"
                }
                p{
                    style:"display:block; max-width: 600px; font-weight:300; line-height:20px;
                    font-size: clamp(16px, 2.5vw, 18px); margin: 0px 0 32px 0; color: black; text-align: left;",
                    "We're teaching the computer to read plans so you don't have to have to babysit the paperwork. It's still learning like any good apprentice --and every plan you upload teaches it something new. Give it a crack, see what it can do, and help us build the future of building."
                }
                button {
                    onclick: move |_| { navigator().push(Route::LoginPage {}); },
                    onmouseenter: move |_| button_hover.set(true),
                    onmouseleave: move |_| button_hover.set(false),
                    style: "{button_style}",
                    span {
                        style: "text-align:left;",
                        "Upload Plans"
                    }
                    img {
                        src: "{LOGO}",
                        alt: "Floorplan Logo",
                        style: "height: 27px; width: auto; display: block;",
                    }
                }
            }
        }
    }
}
