//! Embeddings APIs.

use crate::{param::OneOrMany, shared::ModelId, Client, Result};

/// Embedding service.
#[derive(Clone)]
pub struct EmbeddingService {
    client: Client,
}

impl EmbeddingService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates embeddings for one or more inputs.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: EmbeddingCreateParams) -> Result<EmbeddingResponse> {
        self.client.post_json("/embeddings", &params).await
    }
}

/// Embedding request payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmbeddingCreateParams {
    /// Model ID.
    pub model: ModelId,
    /// Single input or list of inputs.
    pub input: OneOrMany<String>,
    /// Optional vector size for supported models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
}

/// Embedding API response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmbeddingResponse {
    /// Object type.
    pub object: String,
    /// Embedding list.
    pub data: Vec<EmbeddingData>,
    /// Model used for this embedding response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelId>,
    /// Usage details when returned by API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<EmbeddingUsage>,
}

/// One embedding vector item.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmbeddingData {
    /// Object type.
    pub object: String,
    /// Embedding index.
    pub index: u32,
    /// Embedding vector.
    pub embedding: Vec<f32>,
}

/// Token accounting for embeddings responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmbeddingUsage {
    /// Input tokens billed.
    pub prompt_tokens: u32,
    /// Total token count.
    pub total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::{EmbeddingCreateParams, EmbeddingResponse};
    use crate::{param::OneOrMany, shared::ModelId};

    #[test]
    fn create_params_support_single_input_shape() {
        let params = EmbeddingCreateParams {
            model: ModelId::from("text-embedding-3-small"),
            input: OneOrMany::One("hello".to_owned()),
            dimensions: None,
        };

        let value = serde_json::to_value(params).expect("serialize embedding params");
        assert_eq!(
            value.get("input"),
            Some(&serde_json::Value::String("hello".to_owned()))
        );
        assert!(value.get("dimensions").is_none());
    }

    #[test]
    fn embedding_response_deserializes_data() {
        let json = r#"{
            "object":"list",
            "data":[{"object":"embedding","index":0,"embedding":[0.1,0.2]}]
        }"#;

        let response: EmbeddingResponse =
            serde_json::from_str(json).expect("deserialize embedding response");
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].embedding.len(), 2);
    }
}
