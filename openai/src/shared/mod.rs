//! Shared reusable SDK types.

use std::fmt;

/// Newtype wrapper for model IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ModelId(pub String);

impl ModelId {
    /// Creates a model ID from any owned or borrowed string input.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for ModelId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for ModelId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for ModelId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// Common completion finish reasons.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Natural stop.
    Stop,
    /// Max tokens hit.
    Length,
    /// Content policy filter.
    ContentFilter,
    /// Tool call stop.
    ToolCalls,
    /// Legacy function call stop.
    FunctionCall,
}

#[cfg(test)]
mod tests {
    use super::{FinishReason, ModelId};

    #[test]
    fn model_id_is_a_transparent_string() {
        let model_id = ModelId::new("gpt-4o-mini");
        let json = serde_json::to_string(&model_id).expect("serialize model id");
        assert_eq!(json, "\"gpt-4o-mini\"");

        let decoded: ModelId = serde_json::from_str(&json).expect("deserialize model id");
        assert_eq!(decoded.as_ref(), "gpt-4o-mini");
    }

    #[test]
    fn finish_reason_serializes_snake_case() {
        let json =
            serde_json::to_string(&FinishReason::ToolCalls).expect("serialize finish reason");
        assert_eq!(json, "\"tool_calls\"");

        let decoded: FinishReason =
            serde_json::from_str("\"function_call\"").expect("deserialize finish reason");
        assert!(matches!(decoded, FinishReason::FunctionCall));
    }
}
