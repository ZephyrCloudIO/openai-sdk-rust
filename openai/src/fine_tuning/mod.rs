//! Fine-tuning APIs.

use std::collections::HashMap;

use crate::{pagination::CursorPage, Client, Result};

/// Fine-tuning service root.
#[derive(Clone)]
pub struct FineTuningService {
    client: Client,
}

impl FineTuningService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Returns fine-tuning job service.
    #[must_use]
    pub fn jobs(&self) -> FineTuningJobService {
        FineTuningJobService::new(self.client.clone())
    }

    /// Creates a fine-tuning job.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create_job(&self, params: FineTuningJobCreateParams) -> Result<FineTuningJob> {
        self.jobs().create(params).await
    }

    /// Gets one fine-tuning job.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get_job(&self, job_id: impl AsRef<str>) -> Result<FineTuningJob> {
        self.jobs().get(job_id).await
    }

    /// Lists fine-tuning jobs.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list_jobs(&self) -> Result<CursorPage<FineTuningJob>> {
        self.jobs().list().await
    }

    /// Cancels a fine-tuning job.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn cancel_job(&self, job_id: impl AsRef<str>) -> Result<FineTuningJob> {
        self.jobs().cancel(job_id).await
    }

    /// Lists events for a fine-tuning job.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list_job_events(
        &self,
        job_id: impl AsRef<str>,
    ) -> Result<CursorPage<FineTuningJobEvent>> {
        self.jobs().list_events(job_id).await
    }
}

/// Fine-tuning job service.
#[derive(Clone)]
pub struct FineTuningJobService {
    client: Client,
}

impl FineTuningJobService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates a fine-tuning job.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: FineTuningJobCreateParams) -> Result<FineTuningJob> {
        self.client.post_json("/fine_tuning/jobs", &params).await
    }

    /// Retrieves a fine-tuning job by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, job_id: impl AsRef<str>) -> Result<FineTuningJob> {
        self.client
            .get_json(&format!(
                "/fine_tuning/jobs/{}",
                urlencoding::encode(job_id.as_ref())
            ))
            .await
    }

    /// Lists fine-tuning jobs.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(&self) -> Result<CursorPage<FineTuningJob>> {
        self.client.get_json("/fine_tuning/jobs").await
    }

    /// Cancels a fine-tuning job.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn cancel(&self, job_id: impl AsRef<str>) -> Result<FineTuningJob> {
        let body = serde_json::json!({});
        self.client
            .post_json(
                &format!(
                    "/fine_tuning/jobs/{}/cancel",
                    urlencoding::encode(job_id.as_ref())
                ),
                &body,
            )
            .await
    }

    /// Lists status events for a fine-tuning job.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list_events(
        &self,
        job_id: impl AsRef<str>,
    ) -> Result<CursorPage<FineTuningJobEvent>> {
        self.client
            .get_json(&format!(
                "/fine_tuning/jobs/{}/events",
                urlencoding::encode(job_id.as_ref())
            ))
            .await
    }
}

/// Fine-tuning job creation parameters.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FineTuningJobCreateParams {
    /// Model to fine-tune.
    pub model: String,
    /// Training file ID.
    pub training_file: String,
    /// Optional validation file ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_file: Option<String>,
    /// Optional custom suffix.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Fine-tuning job object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FineTuningJob {
    /// Job ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Base model.
    pub model: String,
    /// Current status.
    pub status: String,
    /// Training file ID.
    pub training_file: String,
    /// Optional fine-tuned model name when complete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fine_tuned_model: Option<String>,
}

/// Fine-tuning event object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FineTuningJobEvent {
    /// Event ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Event timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<u64>,
    /// Human-readable message.
    pub message: String,
    /// Event severity level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{FineTuningJob, FineTuningJobCreateParams};

    #[test]
    fn create_params_omit_optional_fields() {
        let params = FineTuningJobCreateParams {
            model: "gpt-4o-mini-2024-07-18".to_owned(),
            training_file: "file_train".to_owned(),
            validation_file: None,
            suffix: None,
            metadata: None,
        };

        let value = serde_json::to_value(params).expect("serialize fine-tuning params");
        assert!(value.get("validation_file").is_none());
        assert!(value.get("suffix").is_none());
        assert!(value.get("metadata").is_none());
    }

    #[test]
    fn fine_tuning_job_deserializes_status() {
        let json = r#"{
            "id":"ftjob_1",
            "object":"fine_tuning.job",
            "model":"gpt-4o-mini",
            "status":"running",
            "training_file":"file_train"
        }"#;

        let job: FineTuningJob = serde_json::from_str(json).expect("deserialize fine-tuning job");
        assert_eq!(job.status, "running");
    }
}
