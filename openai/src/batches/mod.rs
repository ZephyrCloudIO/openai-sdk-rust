//! Batch APIs.

use std::collections::HashMap;

use crate::{pagination::CursorPage, Client, Result};

/// Batch service.
#[derive(Clone)]
pub struct BatchService {
    client: Client,
}

impl BatchService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates and executes a batch from an uploaded file of requests.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: BatchCreateParams) -> Result<Batch> {
        self.client.post_json("/batches", &params).await
    }

    /// Retrieves a batch by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, batch_id: impl AsRef<str>) -> Result<Batch> {
        self.client
            .get_json(&format!(
                "/batches/{}",
                urlencoding::encode(batch_id.as_ref())
            ))
            .await
    }

    /// Lists batches.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(&self) -> Result<CursorPage<Batch>> {
        self.client.get_json("/batches").await
    }

    /// Cancels an in-progress batch.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn cancel(&self, batch_id: impl AsRef<str>) -> Result<Batch> {
        let body = serde_json::json!({});
        self.client
            .post_json(
                &format!("/batches/{}/cancel", urlencoding::encode(batch_id.as_ref())),
                &body,
            )
            .await
    }
}

/// Batch create request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchCreateParams {
    /// ID of the uploaded input file containing JSONL requests.
    pub input_file_id: String,
    /// API endpoint to execute for each request.
    pub endpoint: String,
    /// Processing window, e.g. `24h`.
    pub completion_window: String,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Batch object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Batch {
    /// Batch ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Batch status.
    pub status: String,
    /// Endpoint for the batch.
    pub endpoint: String,
    /// Input file ID.
    pub input_file_id: String,
    /// Completion window.
    pub completion_window: String,
    /// Optional output file ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_file_id: Option<String>,
    /// Optional error file ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_file_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{Batch, BatchCreateParams};

    #[test]
    fn batch_create_params_omit_metadata_when_none() {
        let params = BatchCreateParams {
            input_file_id: "file_123".to_owned(),
            endpoint: "/v1/responses".to_owned(),
            completion_window: "24h".to_owned(),
            metadata: None,
        };

        let value = serde_json::to_value(params).expect("serialize batch create params");
        assert!(value.get("metadata").is_none());
    }

    #[test]
    fn batch_deserializes_status() {
        let json = r#"{
            "id":"batch_1",
            "object":"batch",
            "status":"in_progress",
            "endpoint":"/v1/responses",
            "input_file_id":"file_1",
            "completion_window":"24h"
        }"#;

        let batch: Batch = serde_json::from_str(json).expect("deserialize batch");
        assert_eq!(batch.status, "in_progress");
    }
}
