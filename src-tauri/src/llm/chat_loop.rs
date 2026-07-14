use super::client::{self, ChatMessage};
use crate::tools::ToolRegistry;

const MAX_TOOL_ITERATIONS: usize = 4;

/// What Desky is doing right now, for the frontend to reflect back to the
/// user (e.g. different flavor text while waiting) — distinct from the LLM
/// process's own Starting/Ready lifecycle, which covers the "waking up"
/// phase before any of this even begins.
pub enum ChatPhase {
    Thinking,
    CallingTool { name: String },
}

/// Starts a fresh conversation with the given (already-composed) system
/// prompt — normally the active persona's `compose_system_prompt()`.
pub fn new_conversation(system_prompt: &str) -> Vec<ChatMessage> {
    vec![ChatMessage::system(system_prompt)]
}

// Sent to the model in place of a display-panel tool's real result, so the
// model never sees (and can't garble) the precise values — it just knows
// to briefly acknowledge that the user is seeing them directly.
const DISPLAY_PANEL_PLACEHOLDER: &str = "{\"status\":\"success\",\"note\":\"Shown directly to the \
    user in a panel — don't restate the specific values, just briefly acknowledge it.\"}";

// Small local models sometimes narrate "let me check..." without actually
// calling anything, or just guess outright, especially once there's
// unrelated chat earlier in the conversation. A cheap keyword match on
// system/hardware topics lets us nudge it back toward actually using its
// tools right when it matters, without touching every message.
//
// Matched against whole tokens (not raw substrings) so e.g. "osmium" never
// matches "os", and "program"/"diagram" never match "ram".
const SYSTEM_TOPIC_WORDS: &[&str] = &[
    "sysinfo",
    "ram",
    "memory",
    "cpu",
    "processor",
    "kernel",
    "hostname",
    "hardware",
    "specs",
    "processes",
    "os",
];

const SYSTEM_TOPIC_PHRASES: &[&[&str]] = &[
    &["system", "info"],
    &["disk", "space"],
    &["disk", "usage"],
    &["storage", "space"],
    &["operating", "system"],
    &["host", "name"],
    &["process", "list"],
    &["running", "process"],
    &["free", "space"],
];

fn tokenize(message: &str) -> Vec<String> {
    message
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn contains_phrase(tokens: &[String], phrase: &[&str]) -> bool {
    phrase.len() <= tokens.len()
        && tokens
            .windows(phrase.len())
            .any(|window| window.iter().zip(phrase).all(|(t, p)| t == p))
}

fn mentions_system_topic(message: &str) -> bool {
    let tokens = tokenize(message);
    tokens.iter().any(|t| SYSTEM_TOPIC_WORDS.contains(&t.as_str()))
        || SYSTEM_TOPIC_PHRASES
            .iter()
            .any(|phrase| contains_phrase(&tokens, phrase))
}

const TOOL_REMINDER: &str = "\n\n(Remember: you have get_system_info and list_processes tools \
    available for questions like this — call the appropriate one to get real data rather than \
    guessing or just saying you will.)";

/// Appends the user's message to history, then drives the tool-calling loop:
/// streams a completion, and if the model requests tool calls, dispatches
/// them against `tools` and feeds the results back for another turn, up to
/// `MAX_TOOL_ITERATIONS` times.
///
/// A single iteration can produce text *and* a tool-call request together
/// (e.g. "let me check that for you..." alongside calling get_system_info).
/// Each iteration's text, if any, is a complete, standalone message segment
/// — `on_message_complete` fires once per segment, in order, so the frontend
/// can render each as its own permanent bubble rather than concatenating
/// everything (or losing earlier segments) into one.
///
/// Some tools (`Tool::is_display_panel`) are rendered directly by the
/// frontend rather than narrated: their real result goes to `on_panel`
/// instead of into the model's context, which only gets a placeholder.
pub async fn run_chat_turn(
    port: u16,
    history: &mut Vec<ChatMessage>,
    tools: &ToolRegistry,
    user_message: String,
    mut on_delta: impl FnMut(&str),
    mut on_phase: impl FnMut(ChatPhase),
    mut on_message_complete: impl FnMut(&str),
    mut on_panel: impl FnMut(&str, &serde_json::Value),
) -> anyhow::Result<()> {
    let user_message = if mentions_system_topic(&user_message) {
        format!("{user_message}{TOOL_REMINDER}")
    } else {
        user_message
    };
    history.push(ChatMessage::user(user_message));

    let tool_schemas = tools.to_openai_schema();

    for _ in 0..MAX_TOOL_ITERATIONS {
        on_phase(ChatPhase::Thinking);
        let result = client::stream_chat(port, history, &tool_schemas, &mut on_delta).await?;

        if result.tool_calls.is_empty() {
            history.push(ChatMessage::assistant(result.content.clone()));
            if !result.content.is_empty() {
                on_message_complete(&result.content);
            }
            return Ok(());
        }

        if !result.content.is_empty() {
            on_message_complete(&result.content);
        }
        let content = (!result.content.is_empty()).then_some(result.content);
        history.push(ChatMessage::assistant_tool_calls(
            content,
            result.tool_calls.clone(),
        ));

        for call in result.tool_calls {
            on_phase(ChatPhase::CallingTool {
                name: call.name.clone(),
            });
            let tool_value = dispatch_tool_call(tools, &call.name, &call.arguments);
            let is_panel = tools
                .get(&call.name)
                .map(|t| t.is_display_panel())
                .unwrap_or(false);

            let history_result = if is_panel {
                on_panel(&call.name, &tool_value);
                DISPLAY_PANEL_PLACEHOLDER.to_string()
            } else {
                tool_value.to_string()
            };
            history.push(ChatMessage::tool_result(call.id, history_result));
        }
    }

    anyhow::bail!("exceeded max tool-call iterations without a final response")
}

fn dispatch_tool_call(tools: &ToolRegistry, name: &str, raw_arguments: &str) -> serde_json::Value {
    let Some(tool) = tools.get(name) else {
        return serde_json::json!({ "error": format!("unknown tool: {name}") });
    };

    let args = if raw_arguments.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(raw_arguments).unwrap_or_else(|_| serde_json::json!({}))
    };

    match tool.execute(args) {
        Ok(value) => value,
        Err(e) => serde_json::json!({ "error": e.0 }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;
    use crate::llm::process::LlmProcess;
    use std::path::PathBuf;

    #[test]
    fn detects_system_topics() {
        assert!(mentions_system_topic("how much RAM do I have?"));
        assert!(mentions_system_topic("What's my OS and kernel version"));
        assert!(mentions_system_topic("check my cpu usage"));
        assert!(mentions_system_topic("list running processes"));
        assert!(mentions_system_topic("how much free space do I have on disk"));
    }

    #[test]
    fn ignores_unrelated_messages() {
        assert!(!mentions_system_topic("omg hi desky"));
        assert!(!mentions_system_topic("can you write me a poem"));
        assert!(!mentions_system_topic("I'm doing a marathon program tomorrow"));
        assert!(!mentions_system_topic("what's the closest grocery store"));
    }

    #[test]
    fn matches_whole_words_not_substrings() {
        // "osmium" contains "os", "diagram"/"program" contain "ram" — none
        // of these should match on raw substring alone.
        assert!(!mentions_system_topic("osmium is a dense metal"));
        assert!(!mentions_system_topic("draw me a diagram"));
        assert!(!mentions_system_topic("write a program that sorts a list"));
        assert!(!mentions_system_topic("host a dinner party for hostess duty"));
        // but the real standalone words still match
        assert!(mentions_system_topic("what OS is this"));
        assert!(mentions_system_topic("how much ram do I have"));
    }

    /// Not a CI test — exercises the real tool-calling loop against a real
    /// llama-server + model to manually verify M2 without touching the GUI.
    /// Run with: cargo test --lib -- --ignored --nocapture system_info_tool_call_round_trip
    #[ignore]
    #[tokio::test]
    async fn system_info_tool_call_round_trip() {
        let settings = Settings {
            llama_server_path: Some(PathBuf::from(
                "/home/erik/llama.cpp/build/bin/llama-server",
            )),
            model_path: Some(PathBuf::from(
                "/home/erik/models/qwen2.5-3b-instruct-q4_k_m.gguf",
            )),
            port: 8095,
            context_size: 4096,
            active_persona_id: None,
        };

        let mut llm = LlmProcess::default();
        let port = llm
            .ensure_running(&settings)
            .await
            .expect("llama-server should start");

        let tools = ToolRegistry::new();
        let mut history = new_conversation(
            "You are Desky, a friendly and concise local desktop assistant. You can call \
             tools to look up real information about the host system before answering \
             questions about it — never guess or make up system details when a tool can tell you.",
        );

        run_chat_turn(
            port,
            &mut history,
            &tools,
            "What OS and CPU is this machine running, and how much total and used RAM \
             does it have in GB? Use your tool, don't guess."
                .to_string(),
            |_delta| {},
            |_phase| {},
            |segment| println!("=== MESSAGE SEGMENT ===\n{segment}\n======================="),
            |tool, data| println!("=== PANEL ({tool}) ===\n{data:#}\n==================="),
        )
        .await
        .expect("chat turn should succeed");

        println!("=== FULL HISTORY ===\n{:#?}\n====================", history);

        llm.shutdown().await;
    }

    /// Repro for a user report: asking "what's my os" (fresh conversation,
    /// active "Concise" persona, currently-configured 1.5B model — matches
    /// their exact live setup) made the model answer without calling
    /// get_system_info at all, despite the keyword-triggered tool reminder.
    /// Run with: cargo test --lib -- --ignored --nocapture whats_my_os_repro
    #[ignore]
    #[tokio::test]
    async fn whats_my_os_repro() {
        let settings = Settings {
            llama_server_path: Some(PathBuf::from(
                "/home/erik/llama.cpp/build/bin/llama-server",
            )),
            model_path: Some(PathBuf::from(
                "/home/erik/models/qwen2.5-1.5b-instruct-q4_k_m.gguf",
            )),
            port: 8099,
            context_size: 4096,
            active_persona_id: None,
        };

        let mut llm = LlmProcess::default();
        let port = llm
            .ensure_running(&settings)
            .await
            .expect("llama-server should start");

        let tools = ToolRegistry::new();
        let concise = crate::persona::Persona {
            id: "concise".to_string(),
            name: "Concise".to_string(),
            description: String::new(),
            system_prompt: "You are Desky, a no-nonsense local desktop assistant who values the user's time."
                .to_string(),
            traits: crate::persona::schema::PersonaTraits {
                formality: 0.5,
                humor: 0.1,
                verbosity: 0.1,
                proactivity: 0.3,
            },
            sprite_sheet: None,
            is_builtin: true,
        };
        let history = new_conversation(&concise.compose_system_prompt());

        println!("SYSTEM PROMPT: {}\n", concise.compose_system_prompt());

        for attempt in 1..=3 {
            let mut attempt_history = history.clone();
            let called_tool = std::cell::Cell::new(false);
            run_chat_turn(
                port,
                &mut attempt_history,
                &tools,
                "what's my os".to_string(),
                |_| {},
                |phase| {
                    if let ChatPhase::CallingTool { name } = &phase {
                        called_tool.set(true);
                        println!("[attempt {attempt}] called tool: {name}");
                    }
                },
                |segment| println!("[attempt {attempt}] SEGMENT: {segment}"),
                |tool, data| println!("[attempt {attempt}] PANEL ({tool}): {data:#}"),
            )
            .await
            .expect("chat turn should succeed");

            if !called_tool.get() {
                println!("[attempt {attempt}] *** NO TOOL CALLED ***");
            }
            println!();
        }

        llm.shutdown().await;
    }

    /// Repro for a user report: after some unrelated casual chatting, asking
    /// about the OS made the model answer "Windows 10" instead of calling
    /// get_system_info. Mimics that multi-turn shape to see whether the model
    /// skips the tool call once there's unrelated history in context.
    /// Run with: cargo test --lib -- --ignored --nocapture windows_10_repro
    #[ignore]
    #[tokio::test]
    async fn windows_10_repro() {
        let settings = Settings {
            llama_server_path: Some(PathBuf::from(
                "/home/erik/llama.cpp/build/bin/llama-server",
            )),
            model_path: Some(PathBuf::from(
                "/home/erik/models/qwen2.5-3b-instruct-q4_k_m.gguf",
            )),
            port: 8096,
            context_size: 4096,
            active_persona_id: None,
        };

        let mut llm = LlmProcess::default();
        let port = llm
            .ensure_running(&settings)
            .await
            .expect("llama-server should start");

        let tools = ToolRegistry::new();
        let mut history = new_conversation(
            "You are Desky, a friendly and concise local desktop assistant. You can call \
             tools to look up real information about the host system before answering \
             questions about it — never guess or make up system details when a tool can tell you.",
        );

        for turn in [
            "omg hi desky",
            "can u make an ascii art of a cat",
            "what OS am i on",
        ] {
            println!("=== USER: {turn}");
            run_chat_turn(
                port,
                &mut history,
                &tools,
                turn.to_string(),
                |_| {},
                |_| {},
                |segment| println!("=== DESKY SEGMENT: {segment}"),
                |tool, data| println!("=== PANEL ({tool}): {data:#}"),
            )
            .await
            .expect("chat turn should succeed");
        }

        println!("=== FULL HISTORY ===\n{:#?}\n====================", history);

        llm.shutdown().await;
    }
}
