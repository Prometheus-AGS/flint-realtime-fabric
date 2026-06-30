use serde::{Deserialize, Serialize};

/// Typed payload for an agent protocol event.
///
/// Each variant corresponds to one `AgentEventKind`. The `Unknown` variant
/// absorbs unrecognized or future content without panicking.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Incremental text output from a streaming model response.
    TextDelta { delta: String },

    /// A tool invocation by the model.
    ToolCall {
        tool_name: String,
        input: serde_json::Value,
    },

    /// The result returned from a tool execution.
    ToolResult {
        tool_name: String,
        output: serde_json::Value,
        is_error: bool,
    },

    /// Full agent state snapshot (for resumable sessions).
    StateSnapshot { state: serde_json::Value },

    /// Emitted at the start of an agent run.
    RunStart { model: Option<String> },

    /// Emitted when an agent run completes.
    RunEnd { stop_reason: Option<String> },

    /// An error emitted by the agent.
    Error {
        message: String,
        code: Option<String>,
    },

    /// Unrecognized or future variant — preserved without loss.
    #[serde(other)]
    Unknown,
}

impl ContentBlock {
    /// Return the `type` discriminant as a static string slice.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::TextDelta { .. } => "text_delta",
            Self::ToolCall { .. } => "tool_call",
            Self::ToolResult { .. } => "tool_result",
            Self::StateSnapshot { .. } => "state_snapshot",
            Self::RunStart { .. } => "run_start",
            Self::RunEnd { .. } => "run_end",
            Self::Error { .. } => "error",
            Self::Unknown => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_delta_roundtrips() {
        let block = ContentBlock::TextDelta {
            delta: "hello".to_owned(),
        };
        let json = serde_json::to_string(&block).unwrap();
        let back: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(block, back);
    }

    #[test]
    fn unknown_variant_does_not_panic() {
        let json = r#"{"type":"future_field","extra":42}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        assert_eq!(block, ContentBlock::Unknown);
    }

    #[test]
    fn tool_result_roundtrips() {
        let block = ContentBlock::ToolResult {
            tool_name: "search".to_owned(),
            output: serde_json::json!({"results": []}),
            is_error: false,
        };
        let json = serde_json::to_string(&block).unwrap();
        let back: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(block, back);
    }

    #[test]
    fn type_name_matches_serde_tag() {
        assert_eq!(ContentBlock::Unknown.type_name(), "unknown");
        assert_eq!(
            ContentBlock::RunStart { model: None }.type_name(),
            "run_start"
        );
    }
}
