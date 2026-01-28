use dioxus::prelude::*;

#[component]
pub fn StackingBricksPage() -> Element {
    let mut brick_state = use_signal(|| "hidden");
    let mut brick_visible = use_signal(|| [false; 5]);
    let mut show_text = use_signal(|| false);

    use_future(move || async move {
        loop {
            // Reset all - hidden state
            brick_state.set("hidden");
            brick_visible.set([false; 5]);
            show_text.set(false);
            gloo_timers::future::TimeoutFuture::new(300).await;
            
            // Start showing bricks
            brick_state.set("visible");
            
            // Row 1: brick 1 (180px left)
            brick_visible.set([true, false, false, false, false]);
            gloo_timers::future::TimeoutFuture::new(400).await;
            
            // Row 1: brick 2 (120px right)
            brick_visible.set([true, true, false, false, false]);
            gloo_timers::future::TimeoutFuture::new(400).await;
            
            // Row 2: brick 3 (120px right-aligned to base)
            brick_visible.set([true, true, true, false, false]);
            gloo_timers::future::TimeoutFuture::new(400).await;
            
            // Row 2: brick 4 (120px right)
            brick_visible.set([true, true, true, true, false]);
            gloo_timers::future::TimeoutFuture::new(400).await;
            
            // Row 3: brick 5 (120px centered)
            brick_visible.set([true, true, true, true, true]);
            gloo_timers::future::TimeoutFuture::new(800).await;
            
            // Logo phase - fade out brick 2, 4, 5 only, keep 1 and 3
            brick_state.set("logo");
            gloo_timers::future::TimeoutFuture::new(600).await;
            
            // Show text
            show_text.set(true);
            gloo_timers::future::TimeoutFuture::new(3000).await; // stay on screen
            
            // Fade out everything
            brick_state.set("fadeout-all");
            show_text.set(false);
            gloo_timers::future::TimeoutFuture::new(600).await;
        }
    });

    let bricks = brick_visible();
    let state = brick_state();
    let text_visible = show_text();

    rsx! {
        style { {
            r#"
            .bricks-page {
                display: flex;
                justify-content: center;
                align-items: center;
                min-height: 100vh;
                width: 100vw;
                background-color: #000;
            }
            .bricks-container {
                position: relative;
                width: 304px;
                height: 192px;
            }
            .brick {
                position: absolute;
                height: 60px;
                background-color: rgb(211, 0, 0);
                opacity: 0;
                transform: translateY(-30px);
                filter: blur(0px);
                transition: opacity 200ms ease-out, transform 200ms ease-out, filter 200ms ease-out;
            }
            .brick.visible {
                opacity: 1;
                transform: translateY(0);
                filter: blur(0px);
            }
            .brick.fadeout {
                opacity: 0;
                transform: translateY(0);
                transition: opacity 600ms ease-out;
            }
            .logo-text {
                position: absolute;
                font-family: 'Lexend', sans-serif;
                font-weight: 300;
                font-size: 15px;
                color: white;
                bottom: -30px;
                left: 90px;
                transform: translateX(-50%);
                opacity: 0;
            }
            .logo-text.visible {
                opacity: 1;
            }
            /* Row 1: 180px + 4px gap + 120px */
            .brick-1 {
                width: 180px;
                bottom: 0;
                left: 0;
            }
            .brick-2 {
                width: 120px;
                bottom: 0;
                left: 184px;
            }
            /* Row 2: first brick right-aligned to base brick */
            .brick-3 {
                width: 120px;
                bottom: 64px;
                left: 60px;
            }
            .brick-4 {
                width: 120px;
                bottom: 64px;
                left: 184px;
            }
            /* Row 3: one 120px brick centered */
            .brick-5 {
                width: 120px;
                bottom: 128px;
                left: 92px;
            }
            "#
        } }
        div { class: "bricks-page",
            div { class: "bricks-container",
                // Brick 1 and 3 stay visible during logo phase
                div { class: format!("brick brick-1 {}", 
                    if state == "fadeout-all" { "fadeout" } 
                    else if bricks[0] { "visible" } 
                    else { "" }) 
                }
                div { class: format!("brick brick-2 {}", 
                    if state == "logo" || state == "fadeout-all" { "fadeout" } 
                    else if bricks[1] { "visible" } 
                    else { "" }) 
                }
                div { class: format!("brick brick-3 {}", 
                    if state == "fadeout-all" { "fadeout" } 
                    else if bricks[2] { "visible" } 
                    else { "" }) 
                }
                div { class: format!("brick brick-4 {}", 
                    if state == "logo" || state == "fadeout-all" { "fadeout" } 
                    else if bricks[3] { "visible" } 
                    else { "" }) 
                }
                div { class: format!("brick brick-5 {}", 
                    if state == "logo" || state == "fadeout-all" { "fadeout" } 
                    else if bricks[4] { "visible" } 
                    else { "" }) 
                }
                span { class: format!("logo-text {}", if text_visible { "visible" } else { "" }), "doxle" }
            }
        }
    }
}
