use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub llama_server_path: Option<PathBuf>,
    pub model_path: Option<PathBuf>,
    pub port: u16,
    pub context_size: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            llama_server_path: None,
            model_path: None,
            port: 8090,
            context_size: 4096,
        }
    }
}

impl Settings {
    pub fn load(app_data_dir: &Path) -> Self {
        let path = settings_path(app_data_dir);
        match std::fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, app_data_dir: &Path) -> anyhow::Result<()> {
        std::fs::create_dir_all(app_data_dir)?;
        let path = settings_path(app_data_dir);
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}

fn settings_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("settings.json")
}
