use dioxus::prelude::*;

const LETTER_D: Asset = asset!("/assets/animations/d.svg");
const LETTER_O: Asset = asset!("/assets/animations/o.svg");
const LETTER_X: Asset = asset!("/assets/animations/x.svg");
const LETTER_L: Asset = asset!("/assets/animations/l.svg");
const LETTER_E: Asset = asset!("/assets/animations/e.svg");

// 0-4 = letters D,O,X,L,E
// 5 = loading text phase
const LOADING_PHASE: usize = 5;

#[component]
pub fn LetterCyclePage() -> Element {
    let mut current_index = use_signal(|| 0usize);
    let mut loading_class = use_signal(|| "");
    
    let letters = [LETTER_D, LETTER_O, LETTER_X, LETTER_L, LETTER_E];
    
    use_future(move || async move {
        loop {
            // Cycle through letters D, O, X, L, E
            for i in 0..5 {
                current_index.set(i);
                gloo_timers::future::TimeoutFuture::new(800).await;
            }
            
            // Show loading text phase
            current_index.set(LOADING_PHASE);
            
            // Blink twice fast
            loading_class.set("visible");
            gloo_timers::future::TimeoutFuture::new(150).await;
            loading_class.set("");
            gloo_timers::future::TimeoutFuture::new(150).await;
            loading_class.set("visible");
            gloo_timers::future::TimeoutFuture::new(150).await;
            loading_class.set("");
            gloo_timers::future::TimeoutFuture::new(150).await;
            
            // Stay visible then slow fade out
            loading_class.set("visible");
            gloo_timers::future::TimeoutFuture::new(800).await;
            loading_class.set("fade-out");
            gloo_timers::future::TimeoutFuture::new(600).await;
            loading_class.set("");
            
            // Small pause before restarting
            gloo_timers::future::TimeoutFuture::new(200).await;
        }
    });

    let idx = current_index();
    let load_class = loading_class();

    rsx! {
        style { {
            r#"
            .letter-cycle-page {
                display: flex;
                flex-direction: column;
                justify-content: center;
                align-items: center;
                min-height: 100vh;
                width: 100vw;
                background-color: #000;
            }
            .letter-container {
                position: relative;
                width: 450px;
                height: 305px;
            }
            .letter-container img {
                position: absolute;
                top: 0;
                left: 0;
                width: 450px;
                height: 305px;
                object-fit: contain;
                opacity: 0;
                transition: opacity 150ms ease-in-out;
            }
            .letter-container img.active {
                opacity: 1;
            }
            .loading-text {
                position: absolute;
                font-family: 'Lexend', sans-serif;
                font-weight: 300;
                font-size: 18px;
                color: white;
                opacity: 0;
                transition: opacity 150ms ease-in-out;
            }
            .loading-text.visible {
                opacity: 1;
                transition: opacity 150ms ease-in-out;
            }
            .loading-text.fade-out {
                opacity: 0;
                transition: opacity 600ms ease-in-out;
            }
            "#
        } }
        div { class: "letter-cycle-page",
            div { class: "letter-container",
                for (i, letter) in letters.iter().enumerate() {
                    img {
                        src: "{letter}",
                        class: if i == idx && idx < LOADING_PHASE { "active" } else { "" }
                    }
                }
            }
            if idx == LOADING_PHASE {
                span { class: "loading-text {load_class}", "loading doxle..." }
            }
        }
    }
}
