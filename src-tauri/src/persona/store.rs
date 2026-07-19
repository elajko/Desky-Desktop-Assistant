use super::schema::Persona;
use std::path::{Path, PathBuf};

const BUNDLED_PRESETS: &[(&str, &str)] = &[
    ("friendly", include_str!("../../resources/personas/friendly.json")),
    ("concise", include_str!("../../resources/personas/concise.json")),
    ("snarky", include_str!("../../resources/personas/snarky.json")),
];

pub struct PersonaStore {
    dir: PathBuf,
}

impl PersonaStore {
    /// Copies bundled presets into the app-data personas dir on first run
    /// only (never overwriting an existing file), so user edits to a preset
    /// persist across app restarts/updates.
    pub fn new(app_data_dir: &Path) -> Self {
        let dir = app_data_dir.join("personas");
        let _ = std::fs::create_dir_all(&dir);
        for (id, contents) in BUNDLED_PRESETS {
            let path = dir.join(format!("{id}.json"));
            if !path.exists() {
                let _ = std::fs::write(&path, contents);
            }
        }
        Self { dir }
    }

    pub fn list(&self) -> anyhow::Result<Vec<Persona>> {
        let mut personas = Vec::new();
        for entry in std::fs::read_dir(&self.dir)? {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(persona) = serde_json::from_str::<Persona>(&contents) {
                    personas.push(persona);
                }
            }
        }
        personas.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(personas)
    }

    pub fn get(&self, id: &str) -> anyhow::Result<Option<Persona>> {
        let path = self.dir.join(format!("{id}.json"));
        if !path.exists() {
            return Ok(None);
        }
        let contents = std::fs::read_to_string(path)?;
        Ok(Some(serde_json::from_str(&contents)?))
    }

    pub fn save(&self, persona: &Persona) -> anyhow::Result<()> {
        let path = self.dir.join(format!("{}.json", persona.id));
        std::fs::write(path, serde_json::to_string_pretty(persona)?)?;
        Ok(())
    }

    /// Personas are app configuration, not user files — deleting one just
    /// removes a personality profile, easily recreated. Unlike the Desktop
    /// organizer/scanner, there's no "never delete" constraint here.
    pub fn delete(&self, id: &str) -> anyhow::Result<()> {
        let path = self.dir.join(format!("{id}.json"));
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Re-copies a built-in preset's bundled JSON over the (possibly
    /// user-edited) file on disk, discarding local edits to it.
    pub fn reset_to_bundled(&self, id: &str) -> anyhow::Result<Option<Persona>> {
        let Some((_, contents)) = BUNDLED_PRESETS.iter().find(|(bid, _)| *bid == id) else {
            return Ok(None);
        };
        let path = self.dir.join(format!("{id}.json"));
        std::fs::write(&path, contents)?;
        Ok(Some(serde_json::from_str(contents)?))
    }

    /// Resolves the persona that should actually be active: the requested
    /// id if it still exists, else the first persona alphabetically, else
    /// None if somehow there are no personas at all.
    pub fn resolve_active(&self, active_id: Option<&str>) -> anyhow::Result<Option<Persona>> {
        let all = self.list()?;
        if let Some(id) = active_id {
            if let Some(persona) = all.iter().find(|p| p.id == id) {
                return Ok(Some(persona.clone()));
            }
        }
        Ok(all.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;
    use crate::llm::chat_loop::{new_conversation, run_chat_turn};
    use crate::llm::process::LlmProcess;
    use std::path::PathBuf;

    #[test]
    fn bundled_presets_load_and_parse() {
        let dir = std::env::temp_dir().join(format!("desky-persona-test-{}", std::process::id()));
        let store = PersonaStore::new(&dir);
        let personas = store.list().expect("list should succeed");

        assert_eq!(personas.len(), 3);
        assert!(personas.iter().any(|p| p.id == "friendly"));
        assert!(personas.iter().any(|p| p.id == "concise"));
        assert!(personas.iter().any(|p| p.id == "snarky"));
        assert!(personas.iter().all(|p| p.is_builtin));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Not a CI test — sends the same question through two different
    /// personas' composed prompts and prints both replies so the actual
    /// tonal/length difference can be eyeballed, verifying persona switching
    /// really does change model behavior and not just the stored config.
    /// Run with: cargo test --lib -- --ignored --nocapture persona_actually_changes_model_behavior
    #[ignore]
    #[tokio::test]
    async fn persona_actually_changes_model_behavior() {
        let dir = std::env::temp_dir().join(format!("desky-persona-live-test-{}", std::process::id()));
        let store = PersonaStore::new(&dir);
        let concise = store.get("concise").unwrap().expect("concise should exist");
        let snarky = store.get("snarky").unwrap().expect("snarky should exist");

        let settings = Settings {
            llama_server_path: Some(PathBuf::from(
                "/home/erik/llama.cpp/build/bin/llama-server",
            )),
            model_path: Some(PathBuf::from(
                "/home/erik/models/qwen2.5-3b-instruct-q4_k_m.gguf",
            )),
            port: 8098,
            context_size: 4096,
            active_persona_id: None,
        };

        let mut llm = LlmProcess::default();
        let port = llm
            .ensure_running(&settings)
            .await
            .expect("llama-server should start");

        let question = "My code won't compile. What do I do?";

        for persona in [&concise, &snarky] {
            let mut history =
                new_conversation(&persona.compose_system_prompt(), &persona.first_message);
            println!(
                "\n=== PERSONA: {} ===\nSYSTEM PROMPT: {}",
                persona.name,
                persona.compose_system_prompt()
            );
            let reply = run_chat_turn(port, &mut history, question.to_string(), |_| {})
                .await
                .expect("chat turn should succeed");
            println!("REPLY: {reply}");
        }

        llm.shutdown().await;
        let _ = std::fs::remove_dir_all(&dir);
    }
}
