use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum StatusType {
    Success,
    Error,
    Info,
    Danger,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Status {
    pub message: String,
    pub status_type: StatusType,
    pub progress: Option<Progress>,
    pub log: Vec<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Progress {
    pub current: usize,
    pub total: usize,
    pub elapsed_secs: u64,
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

// Show error that stays until user clicks close
pub fn show_error_persistent(msg: &str) {
    *STATUS.write() = Some(Status {
        message: msg.to_string(),
        status_type: StatusType::Error,
        progress: None,
        log: Vec::new(),
    });
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
        progress: None,
        log: Vec::new(),
    });
}

// Show progress status (persistent, with progress bar + timer)
pub fn show_progress(msg: &str, current: usize, total: usize, elapsed_secs: u64) {
    *STATUS.write() = Some(Status {
        message: msg.to_string(),
        status_type: StatusType::Info,
        progress: Some(Progress { current, total, elapsed_secs }),
        log: Vec::new(),
    });
}

// Show danger/delete progress (red bar)
pub fn show_progress_danger(msg: &str, current: usize, total: usize, elapsed_secs: u64) {
    *STATUS.write() = Some(Status {
        message: msg.to_string(),
        status_type: StatusType::Danger,
        progress: Some(Progress { current, total, elapsed_secs }),
        log: Vec::new(),
    });
}

// Push a completed log line and update progress message
pub fn push_log_danger(log_line: &str, msg: &str, current: usize, total: usize, elapsed_secs: u64) {
    let mut status = STATUS.write();
    if let Some(ref mut s) = *status {
        s.log.push(log_line.to_string());
        s.message = msg.to_string();
        s.progress = Some(Progress { current, total, elapsed_secs });
    } else {
        *status = Some(Status {
            message: msg.to_string(),
            status_type: StatusType::Danger,
            progress: Some(Progress { current, total, elapsed_secs }),
            log: vec![log_line.to_string()],
        });
    }
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
        progress: None,
        log: Vec::new(),
    });

    // Use gloo Timeout callback (not spawn) so the timer survives component unmounts/navigation
    gloo_timers::callback::Timeout::new(duration, || {
        *STATUS.write() = None;
    }).forget();
}
