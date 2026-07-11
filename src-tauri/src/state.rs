use crate::config::Settings;
use crate::llm::client::ChatMessage;
use crate::llm::process::LlmProcess;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub llm: Arc<Mutex<LlmProcess>>,
    pub history: Arc<Mutex<Vec<ChatMessage>>>,
    pub settings: Arc<Mutex<Settings>>,
    pub app_data_dir: PathBuf,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let settings = Settings::load(&app_data_dir);
        Self {
            llm: Arc::new(Mutex::new(LlmProcess::default())),
            history: Arc::new(Mutex::new(crate::llm::chat_loop::new_conversation())),
            settings: Arc::new(Mutex::new(settings)),
            app_data_dir,
        }
    }
}
