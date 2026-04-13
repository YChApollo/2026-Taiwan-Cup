use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub struct SerialState {
    pub cancellation_token: Mutex<Option<CancellationToken>>,
}

impl SerialState {
    pub fn new() -> Self {
        Self {
            cancellation_token: Mutex::new(None),
        }
    }
}
