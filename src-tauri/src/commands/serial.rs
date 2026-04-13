use crate::{
    services::serial,
    state::{self},
};

use tauri::{AppHandle, State};
use tokio_util::sync::CancellationToken;

#[tauri::command]
#[allow(unused)]
async fn start_rx(
    path: String,
    baud_rate: u32,
    serial_state: State<'_, state::SerialState>,
    telemetry_state: State<'_, state::TelemetryState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let failed_count = telemetry_state.packet_verify_failed_count.clone();

    let cancellation_token = {
        let cancellation_token_guard = serial_state.cancellation_token.lock().unwrap();

        if cancellation_token_guard.is_some() {
            return Err("serial monitoring task has already started".to_string());
        }

        let new_cancellation_token = CancellationToken::new();
        *serial_state.cancellation_token.lock().unwrap() = Some(new_cancellation_token.clone());

        new_cancellation_token
    };

    tokio::spawn(async move {
        _ = serial::reader::rx_loop(
            path,
            baud_rate,
            cancellation_token,
            failed_count,
            app_handle,
        )
        .await
    });

    Ok(())
}

#[tauri::command]
#[allow(unused)]
async fn stop_rx(
    serial_state: State<'_, state::SerialState>,
    app_handle: AppHandle,
) -> Result<String, ()> {
    let mut cancellation_token_guard = serial_state.cancellation_token.lock().unwrap();

    if let Some(cancellation_token) = cancellation_token_guard.take() {
        cancellation_token.cancel();
    }

    Ok("serial monitoring task paused gracefully".to_string())
}
