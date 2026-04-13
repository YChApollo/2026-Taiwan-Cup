use crate::services::serial::parser::ParseResult;
use crate::services::serial::parser::Parser;

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncReadExt;
use tokio_serial::SerialPortBuilderExt;
use tokio_util::sync::CancellationToken;

#[allow(unused)]
pub async fn rx_loop(
    path: String,
    baud_rate: u32,
    cancellation_token: CancellationToken,
    failed_count: Arc<Mutex<u32>>,
    app_handle: AppHandle,
) -> Result<(), ()> {
    let mut serial_stream = match tokio_serial::new(path, baud_rate).open_native_async() {
        Ok(s) => s,
        Err(e) => {
            app_handle.emit("serial-error", format!("serial-error: {e}"));
            return Err(());
        }
    };

    let mut parser = Parser::new();

    loop {
        tokio::select! {
            biased; // check cancellation token priority to exit the loop A.S.A.P

            _ = cancellation_token.cancelled() => {  // cancellation requested, exit the loop
                return Ok(());
            }

            result = serial_stream.read_u8() => {
                let byte = match result {
                    Ok(b) => b,
                    Err(e) => {
                        app_handle.emit("serial-error", format!("serial-error: {e}")).ok();
                        return Err(());
                    }
                };

                match parser.feed(byte) {
                    ParseResult::Incomplete => {}
                    ParseResult::Ok(payload) => {
                        app_handle.emit("update-view", &payload).unwrap();
                    }
                    ParseResult::CrcError(e) => {
                        *failed_count.lock().unwrap() += 1;

                        app_handle.emit("packet-verify-failed", ()).unwrap();
                    }
                }
            }
        }
    }
}
