use crate::config::Settings;
use crate::llm::chat_loop::ChatPhase;
use crate::llm::process::LlmStatus;
use crate::persona::Persona;
use crate::state::AppState;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn send_chat_message(
    app: AppHandle,
    state: State<'_, AppState>,
    message: String,
) -> Result<(), String> {
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
    let app_for_message = app.clone();
    crate::llm::chat_loop::run_chat_turn(
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
        move |segment| {
            let _ = app_for_message.emit("chat-message-complete", segment);
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
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

#[tauri::command]
pub async fn list_personas(state: State<'_, AppState>) -> Result<Vec<Persona>, String> {
    state.personas.list().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_persona(state: State<'_, AppState>, persona: Persona) -> Result<(), String> {
    state.personas.save(&persona).map_err(|e| e.to_string())?;
    reset_history_if_active(&state, &persona.id, &persona).await;
    Ok(())
}

#[tauri::command]
pub async fn delete_persona(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let all = state.personas.list().map_err(|e| e.to_string())?;
    if all.len() <= 1 {
        return Err("can't delete the only remaining persona".to_string());
    }

    state.personas.delete(&id).map_err(|e| e.to_string())?;

    let was_active = {
        let settings = state.settings.lock().await;
        settings.active_persona_id.as_deref() == Some(id.as_str())
    };

    if was_active {
        let remaining = state.personas.list().map_err(|e| e.to_string())?;
        if let Some(fallback) = remaining.into_iter().next() {
            {
                let mut settings = state.settings.lock().await;
                settings.active_persona_id = Some(fallback.id.clone());
                settings.save(&state.app_data_dir).map_err(|e| e.to_string())?;
            }
            *state.history.lock().await =
                crate::llm::chat_loop::new_conversation(&fallback.compose_system_prompt());
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn set_active_persona(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let persona = state
        .personas
        .get(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("persona not found: {id}"))?;

    {
        let mut settings = state.settings.lock().await;
        settings.active_persona_id = Some(id);
        settings
            .save(&state.app_data_dir)
            .map_err(|e| e.to_string())?;
    }

    *state.history.lock().await = crate::llm::chat_loop::new_conversation(
        &persona.compose_system_prompt(),
    );

    Ok(())
}

#[tauri::command]
pub async fn reset_persona(state: State<'_, AppState>, id: String) -> Result<Persona, String> {
    let persona = state
        .personas
        .reset_to_bundled(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("{id} is not a built-in persona"))?;

    reset_history_if_active(&state, &id, &persona).await;
    Ok(persona)
}

/// Edits (and resets) to a persona only need to restart the conversation if
/// that persona is the one currently active — otherwise there's nothing live
/// to update.
async fn reset_history_if_active(state: &State<'_, AppState>, id: &str, persona: &Persona) {
    let is_active = state.settings.lock().await.active_persona_id.as_deref() == Some(id);
    if is_active {
        *state.history.lock().await =
            crate::llm::chat_loop::new_conversation(&persona.compose_system_prompt());
    }
}
