//! Legacy text completion APIs.

use std::collections::HashMap;

use futures::Stream;

use crate::{
    chat::ChatCompletionStreamOptions,
    shared::{FinishReason, ModelId},
    ssestream::SseStream,
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

    /// Creates a streaming legacy text completion.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or stream decode failures.
    pub async fn create_stream(
        &self,
        mut params: CompletionCreateParams,
    ) -> Result<impl Stream<Item = Result<Completion>>> {
        params.stream = Some(true);
        let response = self.client.post_raw_json("/completions", &params).await?;
        Ok(SseStream::new(response))
    }
}

/// Prompt union: either a single string or array of strings (or token arrays).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum CompletionPrompt {
    /// Single text prompt.
    Single(String),
    /// Multiple text prompts.
    Multiple(Vec<String>),
    /// Token array (array of token IDs).
    Tokens(Vec<i64>),
    /// Array of token arrays.
    TokenArrays(Vec<Vec<i64>>),
}

impl Default for CompletionPrompt {
    fn default() -> Self {
        Self::Single(String::new())
    }
}

impl From<String> for CompletionPrompt {
    fn from(s: String) -> Self {
        Self::Single(s)
    }
}

impl From<&str> for CompletionPrompt {
    fn from(s: &str) -> Self {
        Self::Single(s.to_owned())
    }
}

impl From<Vec<String>> for CompletionPrompt {
    fn from(v: Vec<String>) -> Self {
        Self::Multiple(v)
    }
}

impl CompletionPrompt {
    /// Returns the union value as JSON for dynamic matching.
    #[must_use]
    pub fn as_any(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Returns the single string prompt variant.
    #[must_use]
    pub fn as_single(&self) -> Option<&str> {
        match self {
            Self::Single(value) => Some(value),
            Self::Multiple(_) | Self::Tokens(_) | Self::TokenArrays(_) => None,
        }
    }

    /// Returns the multiple string prompt variant.
    #[must_use]
    pub fn as_multiple(&self) -> Option<&[String]> {
        match self {
            Self::Multiple(value) => Some(value),
            Self::Single(_) | Self::Tokens(_) | Self::TokenArrays(_) => None,
        }
    }

    /// Creates a single string prompt variant.
    #[must_use]
    pub fn param_of_single(value: impl Into<String>) -> Self {
        Self::Single(value.into())
    }

    /// Creates a multiple string prompt variant.
    #[must_use]
    pub fn param_of_multiple(value: Vec<String>) -> Self {
        Self::Multiple(value)
    }
}

/// Stop sequence: either a single string or array of strings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum CompletionStop {
    /// Single stop string.
    Single(String),
    /// Multiple stop strings.
    Multiple(Vec<String>),
}

impl CompletionStop {
    /// Returns the union value as JSON for dynamic matching.
    #[must_use]
    pub fn as_any(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Returns the single stop sequence variant.
    #[must_use]
    pub fn as_single(&self) -> Option<&str> {
        match self {
            Self::Single(value) => Some(value),
            Self::Multiple(_) => None,
        }
    }

    /// Returns the multiple stop sequence variant.
    #[must_use]
    pub fn as_multiple(&self) -> Option<&[String]> {
        match self {
            Self::Single(_) => None,
            Self::Multiple(value) => Some(value),
        }
    }

    /// Creates a single stop sequence variant.
    #[must_use]
    pub fn param_of_single(value: impl Into<String>) -> Self {
        Self::Single(value.into())
    }

    /// Creates a multiple stop sequence variant.
    #[must_use]
    pub fn param_of_multiple(value: Vec<String>) -> Self {
        Self::Multiple(value)
    }
}

/// Completion request.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CompletionCreateParams {
    /// Model ID.
    pub model: ModelId,
    /// Prompt string(s) or token array(s).
    pub prompt: CompletionPrompt,
    /// Max completion tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Optional sampling temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Enables SSE streaming mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Generates best_of completions server-side and returns the best.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_of: Option<i64>,
    /// Echo back the prompt in addition to the completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<bool>,
    /// Frequency penalty (-2.0 to 2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    /// Include log probabilities on the most likely output tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<i64>,
    /// How many completions to generate for each prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,
    /// Presence penalty (-2.0 to 2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    /// Seed for deterministic sampling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// The suffix that comes after a completion of inserted text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    /// Top-p nucleus sampling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// End-user identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Modify likelihood of specified tokens. Maps token IDs to bias values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i64>>,
    /// Up to 4 stop sequences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<CompletionStop>,
    /// Options for streaming response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<ChatCompletionStreamOptions>,
}

/// Log probabilities for a completion choice.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompletionChoiceLogprobs {
    /// Offsets of each token in the text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_offset: Option<Vec<i64>>,
    /// Log probabilities of each token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_logprobs: Option<Vec<f64>>,
    /// The tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<Vec<String>>,
    /// Top log probabilities at each position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<Vec<HashMap<String, f64>>>,
}

/// Completion response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Completion {
    /// Response ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Unix timestamp of creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<i64>,
    /// Model used to generate this completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelId>,
    /// Returned choices.
    pub choices: Vec<CompletionChoice>,
    /// Usage summary when returned by API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<CompletionUsage>,
    /// System fingerprint for determinism tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
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
    /// Log probability information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<CompletionChoiceLogprobs>,
}

/// Breakdown of completion tokens by type.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompletionUsageCompletionTokensDetails {
    /// Tokens generated that were accepted by the prediction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_prediction_tokens: Option<i64>,
    /// Audio tokens generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<i64>,
    /// Reasoning tokens generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<i64>,
    /// Tokens generated that were rejected by the prediction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_prediction_tokens: Option<i64>,
}

/// Breakdown of prompt tokens by type.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompletionUsagePromptTokensDetails {
    /// Audio tokens in the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<i64>,
    /// Cached tokens in the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<i64>,
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
    /// Breakdown of completion tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<CompletionUsageCompletionTokensDetails>,
    /// Breakdown of prompt tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<CompletionUsagePromptTokensDetails>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ModelId;

    #[test]
    fn create_params_serialize_without_optional_fields() {
        let params = CompletionCreateParams {
            model: ModelId::from("gpt-3.5-turbo-instruct"),
            prompt: CompletionPrompt::Single("hello".to_owned()),
            max_tokens: None,
            temperature: None,
            ..Default::default()
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
        assert!(value.get("best_of").is_none());
        assert!(value.get("echo").is_none());
        assert!(value.get("stream").is_none());
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

    #[test]
    fn prompt_union_string_round_trip() {
        let prompt = CompletionPrompt::Single("test".to_owned());
        let json = serde_json::to_value(&prompt).expect("serialize prompt");
        assert_eq!(json, serde_json::Value::String("test".to_owned()));
    }

    #[test]
    fn prompt_union_array_round_trip() {
        let prompt = CompletionPrompt::Multiple(vec!["hello".to_owned(), "world".to_owned()]);
        let json = serde_json::to_value(&prompt).expect("serialize prompt array");
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 2);
    }

    #[test]
    fn completion_with_system_fingerprint() {
        let json = r#"{
            "id":"cmpl_456",
            "object":"text_completion",
            "created":1700000000,
            "model":"gpt-3.5-turbo-instruct",
            "system_fingerprint":"fp_abc123",
            "choices":[{"index":0,"text":"done","finish_reason":"stop"}]
        }"#;
        let completion: Completion = serde_json::from_str(json).expect("deserialize");
        assert_eq!(completion.system_fingerprint, Some("fp_abc123".to_owned()));
        assert_eq!(completion.created, Some(1700000000));
    }

    #[test]
    fn create_params_with_all_new_fields() {
        let params = CompletionCreateParams {
            model: ModelId::from("gpt-3.5-turbo-instruct"),
            prompt: CompletionPrompt::Single("Say hello".to_owned()),
            best_of: Some(2),
            echo: Some(true),
            frequency_penalty: Some(0.5),
            logprobs: Some(5),
            n: Some(2),
            presence_penalty: Some(0.3),
            seed: Some(42),
            suffix: Some("!".to_owned()),
            top_p: Some(0.9),
            user: Some("user123".to_owned()),
            logit_bias: Some({
                let mut m = HashMap::new();
                m.insert("50256".to_owned(), -100);
                m
            }),
            stop: Some(CompletionStop::Multiple(vec![
                "\n".to_owned(),
                "END".to_owned(),
            ])),
            ..Default::default()
        };
        let json = serde_json::to_value(&params).expect("serialize all fields");
        assert_eq!(json["best_of"], 2);
        assert_eq!(json["echo"], true);
        assert_eq!(json["frequency_penalty"], 0.5);
        assert_eq!(json["logprobs"], 5);
        assert_eq!(json["n"], 2);
        assert_eq!(json["seed"], 42);
        assert_eq!(json["suffix"], "!");
        assert_eq!(json["user"], "user123");
        assert_eq!(json["logit_bias"]["50256"], -100);
    }

    #[test]
    fn completion_choice_logprobs_deserializes() {
        let json = r#"{
            "text_offset": [0, 5],
            "token_logprobs": [-0.5, -0.3],
            "tokens": ["hello", "world"],
            "top_logprobs": [{"hello": -0.5, "hi": -1.2}, {"world": -0.3}]
        }"#;
        let lp: CompletionChoiceLogprobs =
            serde_json::from_str(json).expect("deserialize logprobs");
        assert_eq!(lp.tokens.as_ref().unwrap().len(), 2);
        assert_eq!(lp.top_logprobs.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn completion_with_logprobs_in_choice() {
        let json = r#"{
            "id":"cmpl_lp",
            "object":"text_completion",
            "choices":[{
                "index":0,
                "text":"ok",
                "finish_reason":"stop",
                "logprobs":{
                    "tokens":["ok"],
                    "token_logprobs":[-0.1],
                    "text_offset":[0]
                }
            }]
        }"#;
        let completion: Completion = serde_json::from_str(json).expect("deserialize with logprobs");
        let lp = completion.choices[0]
            .logprobs
            .as_ref()
            .expect("expected logprobs");
        assert_eq!(lp.tokens.as_ref().unwrap()[0], "ok");
    }

    #[test]
    fn completion_tokens_details_round_trip() {
        let details = CompletionUsageCompletionTokensDetails {
            accepted_prediction_tokens: Some(10),
            audio_tokens: Some(5),
            reasoning_tokens: Some(20),
            rejected_prediction_tokens: Some(2),
        };
        let json = serde_json::to_value(&details).expect("serialize completion_tokens_details");
        assert_eq!(json["accepted_prediction_tokens"], 10);
        assert_eq!(json["audio_tokens"], 5);
        assert_eq!(json["reasoning_tokens"], 20);
        assert_eq!(json["rejected_prediction_tokens"], 2);

        let decoded: CompletionUsageCompletionTokensDetails =
            serde_json::from_value(json).expect("deserialize completion_tokens_details");
        assert_eq!(decoded.accepted_prediction_tokens, Some(10));
        assert_eq!(decoded.reasoning_tokens, Some(20));
    }

    #[test]
    fn completion_tokens_details_omits_none_fields() {
        let details = CompletionUsageCompletionTokensDetails {
            accepted_prediction_tokens: None,
            audio_tokens: None,
            reasoning_tokens: Some(15),
            rejected_prediction_tokens: None,
        };
        let json = serde_json::to_value(&details).expect("serialize sparse details");
        assert!(json.get("accepted_prediction_tokens").is_none());
        assert!(json.get("audio_tokens").is_none());
        assert_eq!(json["reasoning_tokens"], 15);
        assert!(json.get("rejected_prediction_tokens").is_none());
    }

    #[test]
    fn prompt_tokens_details_round_trip() {
        let details = CompletionUsagePromptTokensDetails {
            audio_tokens: Some(3),
            cached_tokens: Some(100),
        };
        let json = serde_json::to_value(&details).expect("serialize prompt_tokens_details");
        assert_eq!(json["audio_tokens"], 3);
        assert_eq!(json["cached_tokens"], 100);

        let decoded: CompletionUsagePromptTokensDetails =
            serde_json::from_value(json).expect("deserialize prompt_tokens_details");
        assert_eq!(decoded.audio_tokens, Some(3));
        assert_eq!(decoded.cached_tokens, Some(100));
    }

    #[test]
    fn completion_usage_with_details_deserializes() {
        let json = r#"{
            "prompt_tokens": 50,
            "completion_tokens": 100,
            "total_tokens": 150,
            "completion_tokens_details": {
                "reasoning_tokens": 30,
                "accepted_prediction_tokens": 60,
                "rejected_prediction_tokens": 10
            },
            "prompt_tokens_details": {
                "cached_tokens": 20
            }
        }"#;
        let usage: CompletionUsage = serde_json::from_str(json).expect("deserialize usage");
        assert_eq!(usage.prompt_tokens, 50);
        assert_eq!(usage.completion_tokens, 100);
        assert_eq!(usage.total_tokens, 150);

        let ctd = usage
            .completion_tokens_details
            .expect("expected completion_tokens_details");
        assert_eq!(ctd.reasoning_tokens, Some(30));
        assert_eq!(ctd.accepted_prediction_tokens, Some(60));
        assert_eq!(ctd.rejected_prediction_tokens, Some(10));
        assert!(ctd.audio_tokens.is_none());

        let ptd = usage
            .prompt_tokens_details
            .expect("expected prompt_tokens_details");
        assert_eq!(ptd.cached_tokens, Some(20));
        assert!(ptd.audio_tokens.is_none());
    }

    #[test]
    fn completion_usage_without_details_deserializes() {
        let json = r#"{
            "prompt_tokens": 10,
            "completion_tokens": 20,
            "total_tokens": 30
        }"#;
        let usage: CompletionUsage = serde_json::from_str(json).expect("deserialize usage");
        assert_eq!(usage.prompt_tokens, 10);
        assert!(usage.completion_tokens_details.is_none());
        assert!(usage.prompt_tokens_details.is_none());
    }
}
