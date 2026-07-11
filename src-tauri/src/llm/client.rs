use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    pub fn system(text: impl Into<String>) -> Self {
        Self::plain("system", text)
    }

    pub fn user(text: impl Into<String>) -> Self {
        Self::plain("user", text)
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self::plain("assistant", text)
    }

    pub fn assistant_tool_calls(calls: Vec<ToolCall>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: None,
            tool_calls: Some(
                calls
                    .into_iter()
                    .map(|c| ToolCallRequest {
                        id: c.id,
                        kind: "function".to_string(),
                        function: FunctionCall {
                            name: c.name,
                            arguments: c.arguments,
                        },
                    })
                    .collect(),
            ),
            tool_call_id: None,
        }
    }

    pub fn tool_result(tool_call_id: String, content: String) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(content),
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
        }
    }

    fn plain(role: &str, text: impl Into<String>) -> Self {
        Self {
            role: role.to_string(),
            content: Some(text.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// A fully-assembled tool call as requested by the model, ready to dispatch.
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// What a streamed turn resolved to once the stream ended.
pub enum StreamOutcome {
    Message(String),
    ToolCalls(Vec<ToolCall>),
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    messages: &'a [ChatMessage],
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<&'a [Value]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

// Small local models are noticeably more consistent about actually invoking
// a tool (rather than narrating "let me check..." or guessing) at lower
// sampling temperatures. Only applied to turns where tools are offered, so
// plain conversation keeps the model's normal, more expressive temperature.
const TOOL_CALL_TEMPERATURE: f32 = 0.2;

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
    #[serde(default)]
    tool_calls: Option<Vec<StreamToolCallDelta>>,
}

#[derive(Debug, Deserialize)]
struct StreamToolCallDelta {
    index: usize,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    function: Option<StreamFunctionDelta>,
}

#[derive(Debug, Default, Deserialize)]
struct StreamFunctionDelta {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

#[derive(Default)]
struct PartialToolCall {
    id: String,
    name: String,
    arguments: String,
}

/// Streams a chat completion from llama-server's OpenAI-compatible endpoint.
/// Calls `on_delta` for every incremental piece of assistant *text* as it
/// arrives (tool-call argument fragments are not forwarded to it). Resolves
/// to either the fully accumulated assistant message or the fully assembled
/// set of tool calls the model requested.
pub async fn stream_chat(
    port: u16,
    messages: &[ChatMessage],
    tools: &[Value],
    mut on_delta: impl FnMut(&str),
) -> anyhow::Result<StreamOutcome> {
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");
    let client = reqwest::Client::new();
    let body = ChatCompletionRequest {
        messages,
        stream: true,
        tools: if tools.is_empty() { None } else { Some(tools) },
        tool_choice: if tools.is_empty() { None } else { Some("auto") },
        temperature: if tools.is_empty() {
            None
        } else {
            Some(TOOL_CALL_TEMPERATURE)
        },
    };

    let response = client.post(&url).json(&body).send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("llama-server returned {status}: {text}");
    }

    let mut content_acc = String::new();
    let mut tool_calls_acc: BTreeMap<usize, PartialToolCall> = BTreeMap::new();
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

            for choice in parsed.choices {
                if let Some(content) = choice.delta.content {
                    if !content.is_empty() {
                        content_acc.push_str(&content);
                        on_delta(&content);
                    }
                }

                if let Some(deltas) = choice.delta.tool_calls {
                    for delta in deltas {
                        let entry = tool_calls_acc.entry(delta.index).or_default();
                        if let Some(id) = delta.id {
                            entry.id = id;
                        }
                        if let Some(function) = delta.function {
                            if let Some(name) = function.name {
                                entry.name.push_str(&name);
                            }
                            if let Some(arguments) = function.arguments {
                                entry.arguments.push_str(&arguments);
                            }
                        }
                    }
                }
            }
        }
    }

    if !tool_calls_acc.is_empty() {
        let calls = tool_calls_acc
            .into_values()
            .map(|p| ToolCall {
                id: p.id,
                name: p.name,
                arguments: p.arguments,
            })
            .collect();
        return Ok(StreamOutcome::ToolCalls(calls));
    }

    Ok(StreamOutcome::Message(content_acc))
}
