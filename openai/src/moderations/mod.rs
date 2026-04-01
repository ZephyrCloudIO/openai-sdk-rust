//! Moderation APIs.

use crate::{param::OneOrMany, shared::ModelId, Client, Result};

/// Moderation service.
#[derive(Clone)]
pub struct ModerationService {
    client: Client,
}

impl ModerationService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates moderation results for input text.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: ModerationCreateParams) -> Result<ModerationResponse> {
        self.client.post_json("/moderations", &params).await
    }
}

/// Moderation request payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModerationCreateParams {
    /// Model ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelId>,
    /// Input text or list of inputs.
    pub input: OneOrMany<String>,
}

/// Moderation API response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModerationResponse {
    /// Response ID.
    pub id: String,
    /// Model used.
    pub model: ModelId,
    /// Result list.
    pub results: Vec<ModerationResult>,
}

/// One moderation result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModerationResult {
    /// Flagged status.
    pub flagged: bool,
    /// Category booleans when returned by API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<ModerationCategories>,
}

/// Moderation category flags.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModerationCategories {
    /// Hate category.
    pub hate: bool,
    /// Violence category.
    pub violence: bool,
    /// Sexual category.
    pub sexual: bool,
}

#[cfg(test)]
mod tests {
    use super::{ModerationCreateParams, ModerationResponse};
    use crate::{param::OneOrMany, shared::ModelId};

    #[test]
    fn create_params_support_multiple_inputs() {
        let params = ModerationCreateParams {
            model: None,
            input: OneOrMany::Many(vec!["hello".to_owned(), "world".to_owned()]),
        };

        let value = serde_json::to_value(params).expect("serialize moderation params");
        assert!(value.get("model").is_none());
        assert_eq!(
            value
                .get("input")
                .and_then(serde_json::Value::as_array)
                .map(Vec::len),
            Some(2)
        );
    }

    #[test]
    fn moderation_response_deserializes_model_id() {
        let json = r#"{
            "id":"modr_123",
            "model":"omni-moderation-latest",
            "results":[{"flagged":false}]
        }"#;

        let response: ModerationResponse =
            serde_json::from_str(json).expect("deserialize moderation response");
        assert_eq!(response.model, ModelId::from("omni-moderation-latest"));
        assert_eq!(response.results.len(), 1);
    }
}
