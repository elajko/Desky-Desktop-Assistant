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
pub async fn run_chat_turn(
    port: u16,
    history: &mut Vec<ChatMessage>,
    tools: &ToolRegistry,
    user_message: String,
    mut on_delta: impl FnMut(&str),
    mut on_phase: impl FnMut(ChatPhase),
    mut on_message_complete: impl FnMut(&str),
) -> anyhow::Result<()> {
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
            let tool_result = dispatch_tool_call(tools, &call.name, &call.arguments);
            history.push(ChatMessage::tool_result(call.id, tool_result));
        }
    }

    anyhow::bail!("exceeded max tool-call iterations without a final response")
}

fn dispatch_tool_call(tools: &ToolRegistry, name: &str, raw_arguments: &str) -> String {
    let Some(tool) = tools.get(name) else {
        return serde_json::json!({ "error": format!("unknown tool: {name}") }).to_string();
    };

    let args = if raw_arguments.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(raw_arguments).unwrap_or_else(|_| serde_json::json!({}))
    };

    match tool.execute(args) {
        Ok(value) => value.to_string(),
        Err(e) => serde_json::json!({ "error": e.0 }).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;
    use crate::llm::process::LlmProcess;
    use std::path::PathBuf;

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
        )
        .await
        .expect("chat turn should succeed");

        println!("=== FULL HISTORY ===\n{:#?}\n====================", history);

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
            )
            .await
            .expect("chat turn should succeed");
        }

        println!("=== FULL HISTORY ===\n{:#?}\n====================", history);

        llm.shutdown().await;
    }
}
