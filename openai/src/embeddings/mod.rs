//! Embeddings APIs.

use crate::{param::OneOrMany, shared::ModelId, Client, Result};

/// Encoding format for embedding vectors.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingEncodingFormat {
    /// Float vector format (default).
    Float,
    /// Base64-encoded vector format.
    Base64,
}

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
    /// End-user identifier for abuse monitoring.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// The format to return embeddings in: float or base64.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<EmbeddingEncodingFormat>,
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
    pub index: i64,
    /// Embedding vector (f64 for precision parity with Go SDK).
    pub embedding: Vec<f64>,
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
    use super::*;
    use crate::{param::OneOrMany, shared::ModelId};

    #[test]
    fn create_params_support_single_input_shape() {
        let params = EmbeddingCreateParams {
            model: ModelId::from("text-embedding-3-small"),
            input: OneOrMany::One("hello".to_owned()),
            dimensions: None,
            user: None,
            encoding_format: None,
        };

        let value = serde_json::to_value(params).expect("serialize embedding params");
        assert_eq!(
            value.get("input"),
            Some(&serde_json::Value::String("hello".to_owned()))
        );
        assert!(value.get("dimensions").is_none());
        assert!(value.get("user").is_none());
        assert!(value.get("encoding_format").is_none());
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

    #[test]
    fn embedding_vector_is_f64() {
        let json = r#"{
            "object":"list",
            "data":[{"object":"embedding","index":0,"embedding":[0.123456789012345]}]
        }"#;
        let response: EmbeddingResponse = serde_json::from_str(json).expect("deserialize");
        let val = response.data[0].embedding[0];
        // f64 can represent this value exactly; f32 could not
        assert!((val - 0.123_456_789_012_345).abs() < 1e-15);
    }

    #[test]
    fn embedding_index_is_i64() {
        let json = r#"{
            "object":"list",
            "data":[{"object":"embedding","index":42,"embedding":[0.1]}]
        }"#;
        let response: EmbeddingResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(response.data[0].index, 42i64);
    }

    #[test]
    fn encoding_format_serializes() {
        assert_eq!(
            serde_json::to_string(&EmbeddingEncodingFormat::Float).unwrap(),
            "\"float\""
        );
        assert_eq!(
            serde_json::to_string(&EmbeddingEncodingFormat::Base64).unwrap(),
            "\"base64\""
        );
        let decoded: EmbeddingEncodingFormat = serde_json::from_str("\"base64\"").unwrap();
        assert_eq!(decoded, EmbeddingEncodingFormat::Base64);
    }

    #[test]
    fn create_params_with_user_and_encoding_format() {
        let params = EmbeddingCreateParams {
            model: ModelId::from("text-embedding-3-small"),
            input: OneOrMany::One("test".to_owned()),
            dimensions: Some(512),
            user: Some("user-abc".to_owned()),
            encoding_format: Some(EmbeddingEncodingFormat::Base64),
        };
        let json = serde_json::to_value(&params).expect("serialize");
        assert_eq!(json["user"], "user-abc");
        assert_eq!(json["encoding_format"], "base64");
        assert_eq!(json["dimensions"], 512);
    }
}
