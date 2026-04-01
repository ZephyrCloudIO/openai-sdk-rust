//! Chat APIs.

use futures::Stream;

use crate::{
    shared::{FinishReason, ModelId},
    ssestream::SseStream,
    Client, Result,
};

/// Chat service root.
#[derive(Clone)]
pub struct ChatService {
    client: Client,
}

impl ChatService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Returns chat completions service.
    #[must_use]
    pub fn completions(&self) -> ChatCompletionsService {
        ChatCompletionsService {
            client: self.client.clone(),
        }
    }
}

/// Chat completion endpoints.
#[derive(Clone)]
pub struct ChatCompletionsService {
    client: Client,
}

impl ChatCompletionsService {
    /// Creates a chat completion.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: ChatCompletionCreateParams) -> Result<ChatCompletion> {
        self.client.post_json("/chat/completions", &params).await
    }

    /// Creates a streaming chat completion.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or stream decode failures.
    pub async fn create_stream(
        &self,
        mut params: ChatCompletionCreateParams,
    ) -> Result<impl Stream<Item = Result<ChatCompletionChunk>>> {
        params.stream = Some(true);
        let response = self
            .client
            .post_raw_json("/chat/completions", &params)
            .await?;
        Ok(SseStream::new(response))
    }
}

/// Role-tagged chat message.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    /// System instruction role.
    System,
    /// End-user input role.
    User,
    /// Assistant output role.
    Assistant,
    /// Tool response role.
    Tool,
}

/// Role-tagged chat message.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessageParam {
    /// Message role.
    pub role: ChatRole,
    /// Message content.
    pub content: String,
}

/// Request payload for chat completion creation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionCreateParams {
    /// Model ID.
    pub model: ModelId,
    /// Message list.
    pub messages: Vec<ChatMessageParam>,
    /// Enables SSE streaming mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Optional temperature value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

/// Simplified chat completion response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletion {
    /// Response ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Model used.
    pub model: ModelId,
    /// Choices list.
    pub choices: Vec<ChatCompletionChoice>,
    /// Token accounting details when returned by API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ChatCompletionUsage>,
}

/// Chat completion choice.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionChoice {
    /// Choice index.
    pub index: i64,
    /// Output message.
    pub message: ChatMessageParam,
    /// Stop reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
}

/// Token usage summary for chat completion responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionUsage {
    /// Prompt tokens billed.
    pub prompt_tokens: u32,
    /// Completion tokens billed.
    pub completion_tokens: u32,
    /// Total token count.
    pub total_tokens: u32,
}

/// Streaming chat chunk.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionChunk {
    /// Chunk ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Model used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelId>,
    /// Streaming choices.
    pub choices: Vec<ChatCompletionChunkChoice>,
}

/// Chunk choice delta.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionChunkChoice {
    /// Choice index.
    pub index: i64,
    /// Delta payload.
    pub delta: ChatCompletionChunkDelta,
    /// Optional finish reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
}

/// Delta content within a stream chunk.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ChatCompletionChunkDelta {
    /// Optional role update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<ChatRole>,
    /// Optional token fragment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{ChatCompletionChunk, ChatCompletionCreateParams, ChatMessageParam, ChatRole};
    use crate::shared::ModelId;

    #[test]
    fn create_params_omit_optional_fields() {
        let params = ChatCompletionCreateParams {
            model: ModelId::from("gpt-4o-mini"),
            messages: vec![ChatMessageParam {
                role: ChatRole::User,
                content: "hello".to_owned(),
            }],
            stream: None,
            temperature: None,
        };

        let value = serde_json::to_value(params).expect("serialize create params");
        assert_eq!(
            value.get("model"),
            Some(&serde_json::Value::String("gpt-4o-mini".to_owned()))
        );
        assert!(value.get("stream").is_none());
        assert!(value.get("temperature").is_none());
    }

    #[test]
    fn chunk_deserializes_finish_reason() {
        let json = r#"{
            "id":"chatcmpl_123",
            "object":"chat.completion.chunk",
            "choices":[
                {"index":0,"delta":{"role":"assistant","content":"ok"},"finish_reason":"stop"}
            ]
        }"#;

        let chunk: ChatCompletionChunk = serde_json::from_str(json).expect("deserialize chunk");
        assert_eq!(chunk.choices.len(), 1);
        assert_eq!(
            serde_json::to_string(&chunk.choices[0].finish_reason)
                .expect("serialize finish reason option"),
            "\"stop\""
        );
    }
}
