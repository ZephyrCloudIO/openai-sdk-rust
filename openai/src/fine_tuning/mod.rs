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

    /// Returns fine-tuning checkpoint permission service.
    #[must_use]
    pub fn checkpoint_permissions(&self) -> FineTuningCheckpointPermissionService {
        FineTuningCheckpointPermissionService::new(self.client.clone())
    }

    /// Returns fine-tuning alpha grader service.
    #[must_use]
    pub fn alpha_graders(&self) -> FineTuningAlphaGraderService {
        FineTuningAlphaGraderService::new(self.client.clone())
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

    /// Pauses a fine-tuning job.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn pause_job(&self, job_id: impl AsRef<str>) -> Result<FineTuningJob> {
        self.jobs().pause(job_id).await
    }

    /// Resumes a fine-tuning job.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn resume_job(&self, job_id: impl AsRef<str>) -> Result<FineTuningJob> {
        self.jobs().resume(job_id).await
    }
}

// ---------------------------------------------------------------------------
// Fine-tuning job service
// ---------------------------------------------------------------------------

/// Fine-tuning job service.
#[derive(Clone)]
pub struct FineTuningJobService {
    client: Client,
}

impl FineTuningJobService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Returns the job checkpoint sub-service.
    #[must_use]
    pub fn checkpoints(&self) -> FineTuningJobCheckpointService {
        FineTuningJobCheckpointService::new(self.client.clone())
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
        self.client.get_cursor_page("/fine_tuning/jobs").await
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
            .get_cursor_page(&format!(
                "/fine_tuning/jobs/{}/events",
                urlencoding::encode(job_id.as_ref())
            ))
            .await
    }

    /// Pauses a fine-tuning job.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn pause(&self, job_id: impl AsRef<str>) -> Result<FineTuningJob> {
        let body = serde_json::json!({});
        self.client
            .post_json(
                &format!(
                    "/fine_tuning/jobs/{}/pause",
                    urlencoding::encode(job_id.as_ref())
                ),
                &body,
            )
            .await
    }

    /// Resumes a fine-tuning job.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn resume(&self, job_id: impl AsRef<str>) -> Result<FineTuningJob> {
        let body = serde_json::json!({});
        self.client
            .post_json(
                &format!(
                    "/fine_tuning/jobs/{}/resume",
                    urlencoding::encode(job_id.as_ref())
                ),
                &body,
            )
            .await
    }
}

// ---------------------------------------------------------------------------
// Job checkpoint service
// ---------------------------------------------------------------------------

/// Service for listing checkpoints of a specific fine-tuning job.
#[derive(Clone)]
pub struct FineTuningJobCheckpointService {
    client: Client,
}

impl FineTuningJobCheckpointService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Lists checkpoints for a fine-tuning job with optional pagination parameters.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(
        &self,
        job_id: impl AsRef<str>,
        params: Option<JobCheckpointListParams>,
    ) -> Result<CursorPage<FineTuningJobCheckpoint>> {
        let path = format!(
            "/fine_tuning/jobs/{}/checkpoints",
            urlencoding::encode(job_id.as_ref())
        );
        match params {
            Some(p) => self.client.get_cursor_page_query(&path, &p).await,
            None => self.client.get_cursor_page(&path).await,
        }
    }
}

// ---------------------------------------------------------------------------
// Checkpoint permission service
// ---------------------------------------------------------------------------

/// Service for managing fine-tuning checkpoint permissions.
#[derive(Clone)]
pub struct FineTuningCheckpointPermissionService {
    client: Client,
}

impl FineTuningCheckpointPermissionService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates permissions for a fine-tuned model checkpoint.
    ///
    /// **NOTE:** Requires an admin API key.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(
        &self,
        checkpoint_id: impl AsRef<str>,
        params: CheckpointPermissionCreateParams,
    ) -> Result<CheckpointPermissionCreateResponse> {
        self.client
            .post_json(
                &format!(
                    "/fine_tuning/checkpoints/{}/permissions",
                    urlencoding::encode(checkpoint_id.as_ref())
                ),
                &params,
            )
            .await
    }

    /// Retrieves permissions for a fine-tuned model checkpoint (deprecated in
    /// favor of `list()`).
    ///
    /// **NOTE:** Requires an admin API key.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(
        &self,
        checkpoint_id: impl AsRef<str>,
        params: Option<CheckpointPermissionListParams>,
    ) -> Result<CheckpointPermissionGetResponse> {
        let path = format!(
            "/fine_tuning/checkpoints/{}/permissions",
            urlencoding::encode(checkpoint_id.as_ref())
        );
        match params {
            Some(p) => self.client.get_json_query(&path, &p).await,
            None => self.client.get_json(&path).await,
        }
    }

    /// Lists permissions for a fine-tuned model checkpoint.
    ///
    /// **NOTE:** Requires an admin API key.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(
        &self,
        checkpoint_id: impl AsRef<str>,
        params: Option<CheckpointPermissionListParams>,
    ) -> Result<CursorPage<CheckpointPermission>> {
        let path = format!(
            "/fine_tuning/checkpoints/{}/permissions",
            urlencoding::encode(checkpoint_id.as_ref())
        );
        match params {
            Some(p) => self.client.get_cursor_page_query(&path, &p).await,
            None => self.client.get_cursor_page(&path).await,
        }
    }

    /// Deletes a permission for a fine-tuned model checkpoint.
    ///
    /// **NOTE:** Requires an admin API key.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(
        &self,
        checkpoint_id: impl AsRef<str>,
        permission_id: impl AsRef<str>,
    ) -> Result<CheckpointPermissionDeleteResponse> {
        self.client
            .delete_json(&format!(
                "/fine_tuning/checkpoints/{}/permissions/{}",
                urlencoding::encode(checkpoint_id.as_ref()),
                urlencoding::encode(permission_id.as_ref()),
            ))
            .await
    }
}

// ---------------------------------------------------------------------------
// Alpha grader service
// ---------------------------------------------------------------------------

/// Service for fine-tuning alpha graders (run and validate).
#[derive(Clone)]
pub struct FineTuningAlphaGraderService {
    client: Client,
}

impl FineTuningAlphaGraderService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Runs a grader.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn run(&self, params: AlphaGraderRunParams) -> Result<AlphaGraderRunResponse> {
        self.client
            .post_json("/fine_tuning/alpha/graders/run", &params)
            .await
    }

    /// Validates a grader.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn validate(
        &self,
        params: AlphaGraderValidateParams,
    ) -> Result<AlphaGraderValidateResponse> {
        self.client
            .post_json("/fine_tuning/alpha/graders/validate", &params)
            .await
    }
}

// ===========================================================================
// Request / response types
// ===========================================================================

/// Fine-tuning job creation parameters.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FineTuningJobCreateParams {
    pub model: String,
    pub training_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrations: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyperparameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<serde_json::Value>,
}

/// Fine-tuning job object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FineTuningJob {
    pub id: String,
    pub object: String,
    pub model: String,
    pub status: String,
    pub training_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fine_tuned_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyperparameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trained_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_finish: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrations: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<serde_json::Value>,
}

/// Fine-tuning event object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FineTuningJobEvent {
    pub id: String,
    pub object: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<u64>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
}

/// Fine-tuning job checkpoint object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FineTuningJobCheckpoint {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub fine_tuned_model_checkpoint: String,
    pub fine_tuning_job_id: String,
    pub step_number: i64,
}

/// A checkpoint permission object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CheckpointPermission {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub project_id: String,
}

/// Parameters for creating checkpoint permissions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CheckpointPermissionCreateParams {
    pub project_ids: Vec<String>,
}

/// Response from creating checkpoint permissions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CheckpointPermissionCreateResponse {
    pub data: Vec<CheckpointPermission>,
    pub object: String,
    #[serde(default)]
    pub has_more: bool,
}

/// Response from the deprecated get permissions endpoint.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CheckpointPermissionGetResponse {
    pub data: Vec<CheckpointPermission>,
    pub has_more: bool,
    pub object: String,
}

/// Response from deleting a checkpoint permission.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CheckpointPermissionDeleteResponse {
    pub id: String,
    pub deleted: bool,
    pub object: String,
}

/// The order in which to retrieve permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckpointPermissionOrder {
    Ascending,
    Descending,
}

/// Query parameters for listing checkpoint permissions.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CheckpointPermissionListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<CheckpointPermissionOrder>,
}

/// Query parameters for listing checkpoints of a fine-tuning job.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct JobCheckpointListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

/// Parameters for running a grader.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AlphaGraderRunParams {
    pub grader: serde_json::Value,
    pub model_sample: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<serde_json::Value>,
}

/// Response from running a grader.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AlphaGraderRunResponse {
    pub reward: f64,
    #[serde(default)]
    pub sub_rewards: HashMap<String, serde_json::Value>,
}

/// Parameters for validating a grader.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AlphaGraderValidateParams {
    pub grader: serde_json::Value,
}

/// Response from validating a grader.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AlphaGraderValidateResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grader: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_params_omit_optional_fields() {
        let params = FineTuningJobCreateParams {
            model: "gpt-4o-mini-2024-07-18".to_owned(),
            training_file: "file_train".to_owned(),
            validation_file: None,
            suffix: None,
            metadata: None,
            seed: None,
            integrations: None,
            hyperparameters: None,
            method: None,
        };
        let value = serde_json::to_value(params).expect("serialize");
        assert!(value.get("validation_file").is_none());
        assert!(value.get("suffix").is_none());
    }

    #[test]
    fn fine_tuning_job_deserializes_status() {
        let json = r#"{"id":"ftjob_1","object":"fine_tuning.job","model":"gpt-4o-mini","status":"running","training_file":"file_train"}"#;
        let job: FineTuningJob = serde_json::from_str(json).expect("deserialize");
        assert_eq!(job.status, "running");
    }

    #[test]
    fn checkpoint_permission_list_params_default() {
        let params = CheckpointPermissionListParams::default();
        let value = serde_json::to_value(&params).expect("serialize");
        assert!(value.get("after").is_none());
        assert!(value.get("limit").is_none());
        assert!(value.get("project_id").is_none());
        assert!(value.get("order").is_none());
    }

    #[test]
    fn checkpoint_permission_list_params_serialize_all() {
        let params = CheckpointPermissionListParams {
            after: Some("cperm_abc".to_owned()),
            limit: Some(10),
            project_id: Some("proj_123".to_owned()),
            order: Some(CheckpointPermissionOrder::Ascending),
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["after"], "cperm_abc");
        assert_eq!(value["limit"], 10);
        assert_eq!(value["project_id"], "proj_123");
        assert_eq!(value["order"], "ascending");
    }

    #[test]
    fn checkpoint_permission_order_round_trips() {
        let asc = serde_json::to_string(&CheckpointPermissionOrder::Ascending).unwrap();
        assert_eq!(asc, "\"ascending\"");
        let desc = serde_json::to_string(&CheckpointPermissionOrder::Descending).unwrap();
        assert_eq!(desc, "\"descending\"");
    }

    #[test]
    fn job_checkpoint_list_params_default() {
        let params = JobCheckpointListParams::default();
        let value = serde_json::to_value(&params).expect("serialize");
        assert!(value.get("after").is_none());
        assert!(value.get("limit").is_none());
    }

    #[test]
    fn job_checkpoint_list_params_serialize_all() {
        let params = JobCheckpointListParams {
            after: Some("ftckpt_abc".to_owned()),
            limit: Some(20),
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["after"], "ftckpt_abc");
        assert_eq!(value["limit"], 20);
    }

    #[test]
    fn alpha_grader_run_params_serializes() {
        let params = AlphaGraderRunParams {
            grader: serde_json::json!({"type":"string_check","name":"test","input":"s","reference":"e","operation":"eq"}),
            model_sample: "Hello".to_owned(),
            item: None,
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["grader"]["type"], "string_check");
    }

    #[test]
    fn alpha_grader_validate_response_deserializes() {
        let json = r#"{"grader":{"type":"python","name":"g","source":"x"}}"#;
        let resp: AlphaGraderValidateResponse = serde_json::from_str(json).expect("deserialize");
        assert!(resp.grader.is_some());
    }
}
