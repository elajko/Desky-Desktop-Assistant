use super::client::{self, ChatMessage};

// Hardcoded for M1; replaced by the persona system's composed prompt in M3.
const SYSTEM_PROMPT: &str = "You are Desky, a friendly and concise local desktop assistant.";

pub fn new_conversation() -> Vec<ChatMessage> {
    vec![ChatMessage {
        role: "system".to_string(),
        content: SYSTEM_PROMPT.to_string(),
    }]
}

/// Appends the user's message to history, streams the assistant's reply
/// (invoking `on_delta` per chunk), appends the reply to history, and
/// returns the full reply text.
pub async fn run_chat_turn(
    port: u16,
    history: &mut Vec<ChatMessage>,
    user_message: String,
    on_delta: impl FnMut(&str),
) -> anyhow::Result<String> {
    history.push(ChatMessage {
        role: "user".to_string(),
        content: user_message,
    });

    let reply = client::stream_chat(port, history, on_delta).await?;

    history.push(ChatMessage {
        role: "assistant".to_string(),
        content: reply.clone(),
    });

    Ok(reply)
}
