use crate::services::serial::parser::TelemetryPayload;

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

pub fn on_error(app_handle: &AppHandle, event_name: &str, error_hint: String) {
    app_handle.emit(event_name, error_hint).unwrap();
}

pub fn on_payload(app_handle: &AppHandle, event_name: &str, payload: TelemetryPayload) {
    app_handle.emit(event_name, payload).unwrap();
}

pub fn on_packet_validation_error(failed_count_guard: Arc<Mutex<u32>>) {
    *failed_count_guard.lock().unwrap() += 1;
}
