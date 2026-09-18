#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arnes_host_broker::RuntimeStatus;

#[tauri::command]
fn runtime_status() -> RuntimeStatus {
    arnes_host_broker::runtime_status()
}

#[tauri::command]
fn runtime_start() -> Result<RuntimeStatus, String> {
    arnes_host_broker::start_runtime().map_err(|error| error.to_string())
}

#[tauri::command]
fn runtime_stop() -> Result<RuntimeStatus, String> {
    arnes_host_broker::stop_runtime().map_err(|error| error.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            runtime_status,
            runtime_start,
            runtime_stop
        ])
        .run(tauri::generate_context!())
        .expect("error while running RHODIZ Arnes Desktop");
}
