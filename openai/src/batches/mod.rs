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

    /// Lists batches with optional query parameters.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(&self, params: Option<BatchListParams>) -> Result<CursorPage<Batch>> {
        match params {
            Some(p) => self.client.get_cursor_page_query("/batches", &p).await,
            None => self.client.get_cursor_page("/batches").await,
        }
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

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Completion window for batch creation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BatchCompletionWindow {
    #[serde(rename = "24h")]
    TwentyFourHours,
}

/// Supported batch endpoints.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BatchEndpoint {
    #[serde(rename = "/v1/responses")]
    V1Responses,
    #[serde(rename = "/v1/chat/completions")]
    V1ChatCompletions,
    #[serde(rename = "/v1/embeddings")]
    V1Embeddings,
    #[serde(rename = "/v1/completions")]
    V1Completions,
    #[serde(rename = "/v1/moderations")]
    V1Moderations,
    #[serde(rename = "/v1/images/generations")]
    V1ImagesGenerations,
    #[serde(rename = "/v1/images/edits")]
    V1ImagesEdits,
    #[serde(rename = "/v1/videos")]
    V1Videos,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Batch create request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchCreateParams {
    /// ID of the uploaded input file containing JSONL requests.
    pub input_file_id: String,
    /// API endpoint to execute for each request.
    pub endpoint: BatchEndpoint,
    /// Processing window (currently only `24h`).
    pub completion_window: BatchCompletionWindow,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Optional expiration policy for the output and/or error files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_expires_after: Option<BatchOutputExpiresAfter>,
}

/// Query parameters for listing batches.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct BatchListParams {
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Maximum number of results (1..100, default 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

/// Expiration policy for batch output files.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchOutputExpiresAfter {
    /// Seconds after anchor before the file expires (3600..2592000).
    pub seconds: i64,
    /// Anchor timestamp. Supported: `created_at`.
    #[serde(default = "default_created_at_anchor")]
    pub anchor: String,
}

fn default_created_at_anchor() -> String {
    "created_at".to_owned()
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Batch object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Batch {
    pub id: String,
    pub object: String,
    pub status: String,
    pub endpoint: String,
    pub input_file_id: String,
    pub completion_window: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_file_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_file_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelling_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expired_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finalizing_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_progress_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<BatchErrors>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_counts: Option<BatchRequestCounts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<BatchUsage>,
}

/// Batch error list.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchErrors {
    #[serde(default)]
    pub data: Vec<BatchError>,
    #[serde(default)]
    pub object: String,
}

/// Individual batch error.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
}

/// Batch request counts.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchRequestCounts {
    #[serde(default)]
    pub completed: i64,
    #[serde(default)]
    pub failed: i64,
    #[serde(default)]
    pub total: i64,
}

/// Batch token usage.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchUsage {
    #[serde(default)]
    pub input_tokens: i64,
    #[serde(default)]
    pub output_tokens: i64,
    #[serde(default)]
    pub total_tokens: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_create_params_with_typed_enums() {
        let params = BatchCreateParams {
            input_file_id: "file_123".to_owned(),
            endpoint: BatchEndpoint::V1Responses,
            completion_window: BatchCompletionWindow::TwentyFourHours,
            metadata: None,
            output_expires_after: None,
        };

        let value = serde_json::to_value(&params).expect("serialize batch create params");
        assert!(value.get("metadata").is_none());
        assert!(value.get("output_expires_after").is_none());
        assert_eq!(value["endpoint"], "/v1/responses");
        assert_eq!(value["completion_window"], "24h");
    }

    #[test]
    fn batch_create_params_with_output_expires_after() {
        let params = BatchCreateParams {
            input_file_id: "file_123".to_owned(),
            endpoint: BatchEndpoint::V1ChatCompletions,
            completion_window: BatchCompletionWindow::TwentyFourHours,
            metadata: None,
            output_expires_after: Some(BatchOutputExpiresAfter {
                seconds: 86400,
                anchor: "created_at".to_owned(),
            }),
        };

        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["output_expires_after"]["seconds"], 86400);
        assert_eq!(value["output_expires_after"]["anchor"], "created_at");
        assert_eq!(value["endpoint"], "/v1/chat/completions");
    }

    #[test]
    fn batch_endpoint_enum_round_trips() {
        for (variant, expected) in [
            (BatchEndpoint::V1Responses, "/v1/responses"),
            (BatchEndpoint::V1ChatCompletions, "/v1/chat/completions"),
            (BatchEndpoint::V1Embeddings, "/v1/embeddings"),
            (BatchEndpoint::V1Videos, "/v1/videos"),
        ] {
            let json = serde_json::to_string(&variant).expect("serialize");
            assert_eq!(json, format!("\"{expected}\""));
            let parsed: BatchEndpoint = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn batch_list_params_omit_optional_fields() {
        let params = BatchListParams::default();
        let value = serde_json::to_value(params).expect("serialize");
        assert!(value.get("after").is_none());
        assert!(value.get("limit").is_none());
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
