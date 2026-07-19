mod commands;
mod config;
mod llm;
mod persona;
mod state;

use state::AppState;
use std::time::Duration;
use tauri::Manager;

/// Shuts down llama-server (bounded by a timeout so a stuck lock or hung
/// process can never block the app from quitting).
async fn shutdown_llm(app_handle: &tauri::AppHandle) {
    let state = app_handle.state::<AppState>();
    let shutdown = async { state.llm.lock().await.shutdown().await };
    let _ = tokio::time::timeout(Duration::from_secs(6), shutdown).await;
}

/// In `tauri dev`, the CLI's file watcher restarts the app binary directly
/// (SIGTERM) whenever source files change — that bypasses `on_window_event`
/// entirely, so without this, editing code while llama-server is running
/// orphans it every time. Only relevant in dev/when something signals the
/// process directly; a normal window close still goes through
/// `on_window_event` below.
#[cfg(unix)]
fn spawn_signal_shutdown_handler(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to register SIGTERM handler");
        let mut sigint =
            signal(SignalKind::interrupt()).expect("failed to register SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => {},
            _ = sigint.recv() => {},
        }

        shutdown_llm(&app_handle).await;
        std::process::exit(0);
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            app.manage(AppState::new(app_data_dir));

            #[cfg(unix)]
            spawn_signal_shutdown_handler(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::send_chat_message,
            commands::get_llm_status,
            commands::get_settings,
            commands::save_settings,
            commands::list_personas,
            commands::save_persona,
            commands::delete_persona,
            commands::set_active_persona,
            commands::reset_persona,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Closing the webview window alone doesn't reliably terminate the
                // process on Linux once the request has been intercepted here, so
                // we shut down llama-server first and then force the whole app to
                // exit rather than just closing the window.
                api.prevent_close();
                let app_handle = window.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    shutdown_llm(&app_handle).await;
                    app_handle.exit(0);
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
