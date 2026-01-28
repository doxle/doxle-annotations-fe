use dioxus::prelude::*;
use std::time::Duration;

#[derive(Clone, PartialEq, Debug)]
pub enum StatusType {
    Success,
    Error,
    Info,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Status {
    pub message: String,
    pub status_type: StatusType,
}

// Global Status Signal
pub static STATUS: GlobalSignal<Option<Status>> = Signal::global(|| None);

// Helper to show success (Green)
pub fn show_success(msg: &str) {
    set_status(msg, StatusType::Success, 3000);
}

// Helper to show success (Green)
pub fn show_success_for(msg: &str,seconds:u32) {
    set_status(msg, StatusType::Success, seconds*1000);
}

// Helper to show error (Red)
pub fn show_error(msg: &str) {
    set_status(msg, StatusType::Error, 3000);
}

// New helper: Show info for X seconds
pub fn show_error_for(msg: &str, seconds: u32) {
    set_status(msg, StatusType::Error, seconds * 1000);
}


// Helper to show info (White)
pub fn show_info(msg: &str) {
    set_status(msg, StatusType::Info, 3000);
}

// New helper: Show info for X seconds
pub fn show_info_for(msg: &str, seconds: u32) {
    set_status(msg, StatusType::Info, seconds * 1000);
}



// Internal setter with auto-clear
fn set_status(msg: &str, status_type: StatusType, duration:u32) {
    *STATUS.write() = Some(Status {
        message: msg.to_string(),
        status_type,
    });

    // Auto-clear after 3 seconds
    spawn(async move {
        gloo_timers::future::TimeoutFuture::new(duration).await;
        // Only clear if the status hasn't changed in the meantime
        // Note: A perfect implementation might use IDs to track specific messages,
        // but for a solo dev/simple app, clearing unconditionally after 3s is usually fine UX.
        *STATUS.write() = None;
    });
}
