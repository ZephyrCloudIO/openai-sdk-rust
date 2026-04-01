//! Legacy text completion APIs.

use crate::{
    shared::{FinishReason, ModelId},
    Client, Result,
};

/// Legacy completion service.
#[derive(Clone)]
pub struct CompletionService {
    client: Client,
}

impl CompletionService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates a legacy text completion.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: CompletionCreateParams) -> Result<Completion> {
        self.client.post_json("/completions", &params).await
    }
}

/// Completion request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompletionCreateParams {
    /// Model ID.
    pub model: ModelId,
    /// Prompt string.
    pub prompt: String,
    /// Max completion tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Optional sampling temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

/// Completion response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Completion {
    /// Response ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Model used to generate this completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelId>,
    /// Returned choices.
    pub choices: Vec<CompletionChoice>,
    /// Usage summary when returned by API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<CompletionUsage>,
}

/// Completion choice payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompletionChoice {
    /// Choice index.
    pub index: i64,
    /// Generated text.
    pub text: String,
    /// Stop reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
}

/// Token usage summary for completion responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompletionUsage {
    /// Prompt tokens billed.
    pub prompt_tokens: u32,
    /// Completion tokens billed.
    pub completion_tokens: u32,
    /// Total token count.
    pub total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::{Completion, CompletionCreateParams};
    use crate::shared::ModelId;

    #[test]
    fn create_params_serialize_without_optional_fields() {
        let params = CompletionCreateParams {
            model: ModelId::from("gpt-3.5-turbo-instruct"),
            prompt: "hello".to_owned(),
            max_tokens: None,
            temperature: None,
        };

        let value = serde_json::to_value(params).expect("serialize completion params");
        assert_eq!(
            value.get("model"),
            Some(&serde_json::Value::String(
                "gpt-3.5-turbo-instruct".to_owned()
            ))
        );
        assert!(value.get("max_tokens").is_none());
        assert!(value.get("temperature").is_none());
    }

    #[test]
    fn completion_deserializes_finish_reason_enum() {
        let json = r#"{
            "id":"cmpl_123",
            "object":"text_completion",
            "choices":[{"index":0,"text":"ok","finish_reason":"length"}]
        }"#;

        let completion: Completion = serde_json::from_str(json).expect("deserialize completion");
        assert_eq!(completion.choices.len(), 1);
        assert_eq!(
            serde_json::to_string(&completion.choices[0].finish_reason)
                .expect("serialize finish reason option"),
            "\"length\""
        );
    }
}
