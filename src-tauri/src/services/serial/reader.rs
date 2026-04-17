use crate::services::serial::handler;
use crate::services::serial::parser::ParseResult;
use crate::services::serial::parser::Parser;

use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tokio::io::AsyncReadExt;
use tokio_serial::SerialPortBuilderExt;
use tokio_serial::SerialStream;
use tokio_util::sync::CancellationToken;

fn init_serial(
    app_handle: &AppHandle,
    path: String,
    baud_rate: u32,
) -> Result<SerialStream, String> {
    match tokio_serial::new(path, baud_rate).open_native_async() {
        Ok(s) => return Ok(s),
        Err(e) => {
            handler::on_error(&app_handle, "serial-error", format!("serial-error: {e}"));
            return Err(e.to_string());
        }
    };
}

#[allow(unused)]
pub async fn rx_loop(
    path: String,
    baud_rate: u32,
    cancellation_token: CancellationToken,
    failed_count: Arc<Mutex<u32>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut serial_stream = init_serial(&app_handle, path, baud_rate)?;

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
                        handler::on_error(&app_handle, "serial-error", format!("serial-error: {e}"));
                        return Err(e.to_string());
                    }
                };

                match parser.feed(byte) {
                    ParseResult::Incomplete => {}
                    ParseResult::Ok(payload) => {
                        handler::on_payload(&app_handle, "update-view", payload);
                    }
                    ParseResult::CrcError(e) => {
                        handler::on_packet_validation_error(failed_count.clone());
                    }
                }
            }
        }
    }
}
