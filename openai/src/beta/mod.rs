//! Beta Assistants API baseline.

use std::collections::HashMap;

use crate::{pagination::CursorPage, Client, Result};

/// Beta API namespace.
#[derive(Clone)]
pub struct BetaService {
    client: Client,
}

impl BetaService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Returns assistants service.
    #[must_use]
    pub fn assistants(&self) -> BetaAssistantService {
        BetaAssistantService::new(self.client.clone())
    }

    /// Returns threads service.
    #[must_use]
    pub fn threads(&self) -> BetaThreadService {
        BetaThreadService::new(self.client.clone())
    }
}

/// Assistants service.
#[derive(Clone)]
pub struct BetaAssistantService {
    client: Client,
}

impl BetaAssistantService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates an assistant.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: AssistantCreateParams) -> Result<Assistant> {
        self.client.post_json("/assistants", &params).await
    }

    /// Gets an assistant by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, assistant_id: impl AsRef<str>) -> Result<Assistant> {
        self.client
            .get_json(&format!(
                "/assistants/{}",
                urlencoding::encode(assistant_id.as_ref())
            ))
            .await
    }

    /// Updates an assistant.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn update(
        &self,
        assistant_id: impl AsRef<str>,
        params: AssistantUpdateParams,
    ) -> Result<Assistant> {
        self.client
            .post_json(
                &format!("/assistants/{}", urlencoding::encode(assistant_id.as_ref())),
                &params,
            )
            .await
    }

    /// Lists assistants.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(&self) -> Result<CursorPage<Assistant>> {
        self.client.get_json("/assistants").await
    }

    /// Deletes an assistant.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(&self, assistant_id: impl AsRef<str>) -> Result<AssistantDeleted> {
        self.client
            .delete_json(&format!(
                "/assistants/{}",
                urlencoding::encode(assistant_id.as_ref())
            ))
            .await
    }
}

/// Threads service.
#[derive(Clone)]
pub struct BetaThreadService {
    client: Client,
}

impl BetaThreadService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Returns thread runs service.
    #[must_use]
    pub fn runs(&self) -> BetaThreadRunService {
        BetaThreadRunService::new(self.client.clone())
    }

    /// Creates a thread.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: ThreadCreateParams) -> Result<Thread> {
        self.client.post_json("/threads", &params).await
    }

    /// Gets a thread by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, thread_id: impl AsRef<str>) -> Result<Thread> {
        self.client
            .get_json(&format!(
                "/threads/{}",
                urlencoding::encode(thread_id.as_ref())
            ))
            .await
    }

    /// Updates a thread.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn update(
        &self,
        thread_id: impl AsRef<str>,
        params: ThreadUpdateParams,
    ) -> Result<Thread> {
        self.client
            .post_json(
                &format!("/threads/{}", urlencoding::encode(thread_id.as_ref())),
                &params,
            )
            .await
    }

    /// Deletes a thread.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(&self, thread_id: impl AsRef<str>) -> Result<ThreadDeleted> {
        self.client
            .delete_json(&format!(
                "/threads/{}",
                urlencoding::encode(thread_id.as_ref())
            ))
            .await
    }
}

/// Thread runs service.
#[derive(Clone)]
pub struct BetaThreadRunService {
    client: Client,
}

impl BetaThreadRunService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates a run in a thread.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, thread_id: impl AsRef<str>, params: RunCreateParams) -> Result<Run> {
        self.client
            .post_json(
                &format!("/threads/{}/runs", urlencoding::encode(thread_id.as_ref())),
                &params,
            )
            .await
    }

    /// Gets a run.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, thread_id: impl AsRef<str>, run_id: impl AsRef<str>) -> Result<Run> {
        self.client
            .get_json(&format!(
                "/threads/{}/runs/{}",
                urlencoding::encode(thread_id.as_ref()),
                urlencoding::encode(run_id.as_ref())
            ))
            .await
    }

    /// Lists runs belonging to a thread.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(&self, thread_id: impl AsRef<str>) -> Result<CursorPage<Run>> {
        self.client
            .get_json(&format!(
                "/threads/{}/runs",
                urlencoding::encode(thread_id.as_ref())
            ))
            .await
    }

    /// Cancels an in-progress run.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn cancel(&self, thread_id: impl AsRef<str>, run_id: impl AsRef<str>) -> Result<Run> {
        let body = serde_json::json!({});
        self.client
            .post_json(
                &format!(
                    "/threads/{}/runs/{}/cancel",
                    urlencoding::encode(thread_id.as_ref()),
                    urlencoding::encode(run_id.as_ref())
                ),
                &body,
            )
            .await
    }
}

/// Assistant create request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssistantCreateParams {
    /// Model used by assistant.
    pub model: String,
    /// Optional display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Assistant update request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssistantUpdateParams {
    /// Optional display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Assistant object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Assistant {
    /// Assistant ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Model ID.
    pub model: String,
    /// Optional assistant name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional assistant instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

/// Assistant deletion response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssistantDeleted {
    /// Assistant ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Deletion flag.
    pub deleted: bool,
}

/// Thread create request.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ThreadCreateParams {
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Thread update request.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ThreadUpdateParams {
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Thread object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Thread {
    /// Thread ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Thread deletion response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThreadDeleted {
    /// Thread ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Deletion flag.
    pub deleted: bool,
}

/// Run create request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunCreateParams {
    /// Assistant ID used for this run.
    pub assistant_id: String,
    /// Optional model override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Optional run instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

/// Thread run object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Run {
    /// Run ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Thread ID.
    pub thread_id: String,
    /// Assistant ID.
    pub assistant_id: String,
    /// Run status.
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::{AssistantCreateParams, Run, RunCreateParams, ThreadCreateParams};

    #[test]
    fn assistant_create_omits_optional_fields() {
        let params = AssistantCreateParams {
            model: "gpt-4o-mini".to_owned(),
            name: None,
            instructions: None,
            metadata: None,
        };

        let value = serde_json::to_value(params).expect("serialize assistant params");
        assert!(value.get("name").is_none());
        assert!(value.get("instructions").is_none());
        assert!(value.get("metadata").is_none());
    }

    #[test]
    fn thread_create_default_serializes_empty_object() {
        let value =
            serde_json::to_value(ThreadCreateParams::default()).expect("serialize thread params");
        assert_eq!(value, serde_json::json!({}));
    }

    #[test]
    fn run_create_omits_optional_fields() {
        let params = RunCreateParams {
            assistant_id: "asst_1".to_owned(),
            model: None,
            instructions: None,
        };

        let value = serde_json::to_value(params).expect("serialize run create params");
        assert!(value.get("model").is_none());
        assert!(value.get("instructions").is_none());
    }

    #[test]
    fn run_deserializes_status() {
        let json = r#"{
            "id":"run_1",
            "object":"thread.run",
            "thread_id":"thread_1",
            "assistant_id":"asst_1",
            "status":"queued"
        }"#;

        let run: Run = serde_json::from_str(json).expect("deserialize run");
        assert_eq!(run.status, "queued");
    }
}
