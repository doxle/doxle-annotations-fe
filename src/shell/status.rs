use dioxus::prelude::*;

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

// Show info that stays until manually cleared or replaced
pub fn show_info_persistent(msg: &str) {
    *STATUS.write() = Some(Status {
        message: msg.to_string(),
        status_type: StatusType::Info,
    });
}

// Clear status immediately
pub fn clear_status() {
    *STATUS.write() = None;
}

// Internal setter with auto-clear
fn set_status(msg: &str, status_type: StatusType, duration:u32) {
    *STATUS.write() = Some(Status {
        message: msg.to_string(),
        status_type,
    });

    // Use gloo Timeout callback (not spawn) so the timer survives component unmounts/navigation
    gloo_timers::callback::Timeout::new(duration, || {
        *STATUS.write() = None;
    }).forget();
}
