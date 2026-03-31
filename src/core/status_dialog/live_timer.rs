use dioxus::prelude::*;

const LIVE_TIMER_CSS: &str = include_str!("live_timer.css");
fn format_elapsed(total_ms: u128) -> String {
    let hours = total_ms / 3_600_000;
    let mins = (total_ms % 3_600_000) / 60_000;
    let secs = (total_ms % 60_000) / 1_000;
    let centis = (total_ms % 1_000) / 10;
    format!("{hours:02}::{mins:02}::{secs:02}::{centis:02}")
}

#[component]
pub fn LiveTimer() -> Element {
    let mut elapsed_ms = use_signal(|| 0u128);

    let _timer = use_future(move || async move {
        let start = web_time::Instant::now();
        loop {
            gloo_timers::future::TimeoutFuture::new(10).await;
            elapsed_ms.set(start.elapsed().as_millis());
        }
    });

    let elapsed_display = format_elapsed(elapsed_ms());
    let secs = (elapsed_ms() / 1000) as u64;
    let bar_class = if secs >= 75 {
        "live-timer-loader bar-4"
    } else if secs >= 50 {
        "live-timer-loader bar-3"
    } else if secs >= 25 {
        "live-timer-loader bar-2"
    } else {
        "live-timer-loader bar-1"
    };

    rsx! {
        style { {LIVE_TIMER_CSS} }
        div { class: bar_class,
            span { class: "live-timer-value", "{elapsed_display}" }
        }
    }
}
