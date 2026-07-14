pub mod system_info;

use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub struct ToolError(pub String);

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ToolError {}

/// A capability the model can invoke. Every tool registered here is exposed
/// to the LLM as a callable function — only put read-only operations behind
/// this trait. Mutating actions (e.g. applying a file move) must stay
/// reachable solely via a direct, human-triggered Tauri command instead.
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters_schema(&self) -> Value;
    fn execute(&self, args: Value) -> Result<Value, ToolError>;

    /// If true, this tool's real result is never fed back into the model's
    /// context — only a placeholder acknowledgment is. The actual result
    /// goes straight to the frontend instead, to render as a hardcoded
    /// panel (keyed by this tool's `name()`). Keeps precise data out of the
    /// model's context (faster, smaller model can't garble numbers it never
    /// sees) while still showing the user the real thing, deterministically
    /// rendered rather than narrated.
    fn is_display_panel(&self) -> bool {
        false
    }
}

pub struct ToolRegistry {
    tools: HashMap<&'static str, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut tools: HashMap<&'static str, Box<dyn Tool>> = HashMap::new();
        for tool in system_info::tools() {
            tools.insert(tool.name(), tool);
        }
        Self { tools }
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// Serializes every registered tool into the OpenAI `tools` request shape.
    pub fn to_openai_schema(&self) -> Vec<Value> {
        self.tools
            .values()
            .map(|tool| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.name(),
                        "description": tool.description(),
                        "parameters": tool.parameters_schema(),
                    }
                })
            })
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
