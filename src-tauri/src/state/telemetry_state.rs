use std::sync::{Arc, Mutex};

pub struct TelemetryState {
    pub packet_verify_failed_count: Arc<Mutex<u32>>,
}
