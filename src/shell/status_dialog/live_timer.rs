use dioxus::prelude::*;

const LIVE_TIMER_CSS: &str = include_str!("live_timer.css");

#[component]
pub fn LiveTimer() -> Element {
    let mut elapsed = use_signal(|| 0u64);

    let _timer = use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(1_000).await;
            elapsed.set(elapsed() + 1);
        }
    });

    let mins = elapsed() / 60;
    let secs = elapsed() % 60;

    rsx! {
        style { {LIVE_TIMER_CSS} }
        span { class: "live-timer", "{mins:02}:{secs:02}" }
    }
}
