mod commands;
mod config;
mod llm;
mod state;

use state::AppState;
use std::time::Duration;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            app.manage(AppState::new(app_data_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::send_chat_message,
            commands::get_llm_status,
            commands::get_settings,
            commands::save_settings,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Closing the webview window alone doesn't reliably terminate the
                // process on Linux once the request has been intercepted here, so
                // we shut down llama-server (bounded by a timeout so a stuck lock
                // or hung process can never block the app from quitting) and then
                // force the whole app to exit rather than just closing the window.
                api.prevent_close();
                let app_handle = window.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    let state = app_handle.state::<AppState>();
                    let shutdown = async { state.llm.lock().await.shutdown().await };
                    let _ = tokio::time::timeout(Duration::from_secs(6), shutdown).await;
                    app_handle.exit(0);
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
