use crate::config::Settings;
use crate::llm::client::ChatMessage;
use crate::llm::process::LlmProcess;
use crate::persona::PersonaStore;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

// Only used if the personas dir is somehow empty (bundled presets failed to
// copy) — should not happen in normal operation.
const FALLBACK_SYSTEM_PROMPT: &str = "You are Desky, a friendly and concise local desktop assistant.";

pub struct AppState {
    pub llm: Arc<Mutex<LlmProcess>>,
    pub history: Arc<Mutex<Vec<ChatMessage>>>,
    pub settings: Arc<Mutex<Settings>>,
    pub personas: PersonaStore,
    pub app_data_dir: PathBuf,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let settings = Settings::load(&app_data_dir);
        let personas = PersonaStore::new(&app_data_dir);

        let system_prompt = personas
            .resolve_active(settings.active_persona_id.as_deref())
            .ok()
            .flatten()
            .map(|p| p.compose_system_prompt())
            .unwrap_or_else(|| FALLBACK_SYSTEM_PROMPT.to_string());

        Self {
            llm: Arc::new(Mutex::new(LlmProcess::default())),
            history: Arc::new(Mutex::new(crate::llm::chat_loop::new_conversation(
                &system_prompt,
            ))),
            settings: Arc::new(Mutex::new(settings)),
            personas,
            app_data_dir,
        }
    }
}
