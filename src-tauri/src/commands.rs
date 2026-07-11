use crate::config::Settings;
use crate::llm::chat_loop::ChatPhase;
use crate::llm::process::LlmStatus;
use crate::state::AppState;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn send_chat_message(
    app: AppHandle,
    state: State<'_, AppState>,
    message: String,
) -> Result<String, String> {
    let settings = state.settings.lock().await.clone();

    let already_ready = { state.llm.lock().await.status == LlmStatus::Ready };
    if !already_ready {
        let _ = app.emit("chat-status", "waking_up");
    }

    let port = {
        let mut llm = state.llm.lock().await;
        llm.ensure_running(&settings)
            .await
            .map_err(|e| e.to_string())?
    };

    let mut history = state.history.lock().await;
    let app_for_delta = app.clone();
    let app_for_phase = app.clone();
    let reply = crate::llm::chat_loop::run_chat_turn(
        port,
        &mut history,
        &state.tools,
        message,
        move |delta| {
            let _ = app_for_delta.emit("chat-delta", delta);
        },
        move |phase| {
            let status = match phase {
                ChatPhase::Thinking => "thinking".to_string(),
                ChatPhase::CallingTool { name } => format!("calling_tool:{name}"),
            };
            let _ = app_for_phase.emit("chat-status", status);
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(reply)
}

#[tauri::command]
pub async fn get_llm_status(state: State<'_, AppState>) -> Result<LlmStatus, String> {
    let mut llm = state.llm.lock().await;
    llm.poll_crashed();
    Ok(llm.status.clone())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    Ok(state.settings.lock().await.clone())
}

#[tauri::command]
pub async fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    settings
        .save(&state.app_data_dir)
        .map_err(|e| e.to_string())?;
    *state.settings.lock().await = settings;
    Ok(())
}
