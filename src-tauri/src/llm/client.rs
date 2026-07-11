use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    messages: &'a [ChatMessage],
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Debug, Default, Deserialize)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
}

/// Streams a chat completion from llama-server's OpenAI-compatible endpoint.
/// Calls `on_delta` for every incremental piece of assistant text as it arrives,
/// and returns the fully accumulated response text once the stream ends.
pub async fn stream_chat(
    port: u16,
    messages: &[ChatMessage],
    mut on_delta: impl FnMut(&str),
) -> anyhow::Result<String> {
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");
    let client = reqwest::Client::new();
    let body = ChatCompletionRequest {
        messages,
        stream: true,
    };

    let response = client.post(&url).json(&body).send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("llama-server returned {status}: {text}");
    }

    let mut accumulated = String::new();
    let mut buffer = String::new();
    let mut byte_stream = response.bytes_stream();

    while let Some(chunk) = byte_stream.next().await {
        let chunk = chunk?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].trim().to_string();
            buffer.drain(..=newline_pos);

            let Some(data) = line.strip_prefix("data:") else {
                continue;
            };
            let data = data.trim();
            if data.is_empty() || data == "[DONE]" {
                continue;
            }

            let parsed: StreamChunk = match serde_json::from_str(data) {
                Ok(parsed) => parsed,
                Err(_) => continue,
            };

            if let Some(choice) = parsed.choices.first() {
                if let Some(content) = &choice.delta.content {
                    accumulated.push_str(content);
                    on_delta(content);
                }
            }
        }
    }

    Ok(accumulated)
}
