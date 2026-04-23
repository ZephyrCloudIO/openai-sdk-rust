//! Beta Assistants API — assistants, threads, messages, runs, run steps.

use std::collections::HashMap;

use futures::Stream;
use serde_json::Value;

use crate::{
    config::RequestOptions,
    pagination::{ConversationCursorPage, CursorPage},
    ssestream::SseStream,
    Client, Result,
};

fn assistants_beta_options() -> RequestOptions {
    RequestOptions::new().with_header("OpenAI-Beta", "assistants=v2")
}

fn chatkit_beta_options() -> RequestOptions {
    RequestOptions::new().with_header("OpenAI-Beta", "chatkit_beta=v1")
}

// ---------------------------------------------------------------------------
// Common list-params order enum
// ---------------------------------------------------------------------------

/// Sort order for list endpoints.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

// ---------------------------------------------------------------------------
// Service hierarchy
// ---------------------------------------------------------------------------

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

    /// Returns ChatKit service.
    #[must_use]
    pub fn chat_kit(&self) -> BetaChatKitService {
        BetaChatKitService::new(self.client.clone())
    }
}

// ---------------------------------------------------------------------------
// Assistants
// ---------------------------------------------------------------------------

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
        self.client.post_json_beta("/assistants", &params).await
    }

    /// Gets an assistant by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, assistant_id: impl AsRef<str>) -> Result<Assistant> {
        self.client
            .get_json_beta(&format!(
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
            .post_json_beta(
                &format!("/assistants/{}", urlencoding::encode(assistant_id.as_ref())),
                &params,
            )
            .await
    }

    /// Lists assistants with optional pagination parameters.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(
        &self,
        params: Option<BetaAssistantListParams>,
    ) -> Result<CursorPage<Assistant>> {
        match params {
            Some(p) => {
                self.client
                    .get_cursor_page_query_options("/assistants", &p, assistants_beta_options())
                    .await
            }
            None => {
                self.client
                    .get_cursor_page_options("/assistants", assistants_beta_options())
                    .await
            }
        }
    }

    /// Deletes an assistant.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(&self, assistant_id: impl AsRef<str>) -> Result<AssistantDeleted> {
        self.client
            .delete_json_beta(&format!(
                "/assistants/{}",
                urlencoding::encode(assistant_id.as_ref())
            ))
            .await
    }
}

// ---------------------------------------------------------------------------
// Threads
// ---------------------------------------------------------------------------

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

    /// Returns thread messages service.
    #[must_use]
    pub fn messages(&self) -> BetaThreadMessageService {
        BetaThreadMessageService::new(self.client.clone())
    }

    /// Creates a thread.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: ThreadCreateParams) -> Result<Thread> {
        self.client.post_json_beta("/threads", &params).await
    }

    /// Gets a thread by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, thread_id: impl AsRef<str>) -> Result<Thread> {
        self.client
            .get_json_beta(&format!(
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
            .post_json_beta(
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
            .delete_json_beta(&format!(
                "/threads/{}",
                urlencoding::encode(thread_id.as_ref())
            ))
            .await
    }

    /// Creates a thread and runs it in one request.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create_and_run(&self, params: ThreadCreateAndRunParams) -> Result<Run> {
        self.client.post_json_beta("/threads/runs", &params).await
    }

    /// Creates a thread and runs it in one request, returning an SSE stream.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or stream decode failures.
    pub async fn create_and_run_stream(
        &self,
        mut params: ThreadCreateAndRunParams,
    ) -> Result<impl Stream<Item = Result<AssistantStreamEvent>>> {
        params.stream = Some(true);
        let response = self
            .client
            .post_raw_json_beta("/threads/runs", &params)
            .await?;
        Ok(SseStream::new(response))
    }
}

// ---------------------------------------------------------------------------
// Thread Messages
// ---------------------------------------------------------------------------

/// Thread messages service (nested under threads).
#[derive(Clone)]
pub struct BetaThreadMessageService {
    client: Client,
}

impl BetaThreadMessageService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates a message in a thread.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(
        &self,
        thread_id: impl AsRef<str>,
        params: MessageCreateParams,
    ) -> Result<Message> {
        self.client
            .post_json_beta(
                &format!(
                    "/threads/{}/messages",
                    urlencoding::encode(thread_id.as_ref())
                ),
                &params,
            )
            .await
    }

    /// Gets a message by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(
        &self,
        thread_id: impl AsRef<str>,
        message_id: impl AsRef<str>,
    ) -> Result<Message> {
        self.client
            .get_json_beta(&format!(
                "/threads/{}/messages/{}",
                urlencoding::encode(thread_id.as_ref()),
                urlencoding::encode(message_id.as_ref())
            ))
            .await
    }

    /// Updates a message (metadata only).
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn update(
        &self,
        thread_id: impl AsRef<str>,
        message_id: impl AsRef<str>,
        params: MessageUpdateParams,
    ) -> Result<Message> {
        self.client
            .post_json_beta(
                &format!(
                    "/threads/{}/messages/{}",
                    urlencoding::encode(thread_id.as_ref()),
                    urlencoding::encode(message_id.as_ref())
                ),
                &params,
            )
            .await
    }

    /// Lists messages for a thread with optional pagination parameters.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(
        &self,
        thread_id: impl AsRef<str>,
        params: Option<BetaThreadMessageListParams>,
    ) -> Result<CursorPage<Message>> {
        let path = format!(
            "/threads/{}/messages",
            urlencoding::encode(thread_id.as_ref())
        );
        match params {
            Some(p) => {
                self.client
                    .get_cursor_page_query_options(&path, &p, assistants_beta_options())
                    .await
            }
            None => {
                self.client
                    .get_cursor_page_options(&path, assistants_beta_options())
                    .await
            }
        }
    }

    /// Deletes a message.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(
        &self,
        thread_id: impl AsRef<str>,
        message_id: impl AsRef<str>,
    ) -> Result<MessageDeleted> {
        self.client
            .delete_json_beta(&format!(
                "/threads/{}/messages/{}",
                urlencoding::encode(thread_id.as_ref()),
                urlencoding::encode(message_id.as_ref())
            ))
            .await
    }
}

// ---------------------------------------------------------------------------
// Thread Runs
// ---------------------------------------------------------------------------

/// Thread runs service.
#[derive(Clone)]
pub struct BetaThreadRunService {
    client: Client,
}

impl BetaThreadRunService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Returns run steps service.
    #[must_use]
    pub fn steps(&self) -> BetaThreadRunStepService {
        BetaThreadRunStepService::new(self.client.clone())
    }

    /// Creates a run in a thread.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, thread_id: impl AsRef<str>, params: RunCreateParams) -> Result<Run> {
        self.client
            .post_json_beta(
                &format!("/threads/{}/runs", urlencoding::encode(thread_id.as_ref())),
                &params,
            )
            .await
    }

    /// Creates a run in a thread, returning an SSE stream of events.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or stream decode failures.
    pub async fn create_stream(
        &self,
        thread_id: impl AsRef<str>,
        mut params: RunCreateParams,
    ) -> Result<impl Stream<Item = Result<AssistantStreamEvent>>> {
        params.stream = Some(true);
        let response = self
            .client
            .post_raw_json_beta(
                &format!("/threads/{}/runs", urlencoding::encode(thread_id.as_ref())),
                &params,
            )
            .await?;
        Ok(SseStream::new(response))
    }

    /// Gets a run.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, thread_id: impl AsRef<str>, run_id: impl AsRef<str>) -> Result<Run> {
        self.client
            .get_json_beta(&format!(
                "/threads/{}/runs/{}",
                urlencoding::encode(thread_id.as_ref()),
                urlencoding::encode(run_id.as_ref())
            ))
            .await
    }

    /// Updates a run (metadata only).
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn update(
        &self,
        thread_id: impl AsRef<str>,
        run_id: impl AsRef<str>,
        params: BetaThreadRunUpdateParams,
    ) -> Result<Run> {
        self.client
            .post_json_beta(
                &format!(
                    "/threads/{}/runs/{}",
                    urlencoding::encode(thread_id.as_ref()),
                    urlencoding::encode(run_id.as_ref())
                ),
                &params,
            )
            .await
    }

    /// Lists runs belonging to a thread with optional pagination parameters.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(
        &self,
        thread_id: impl AsRef<str>,
        params: Option<BetaThreadRunListParams>,
    ) -> Result<CursorPage<Run>> {
        let path = format!("/threads/{}/runs", urlencoding::encode(thread_id.as_ref()));
        match params {
            Some(p) => {
                self.client
                    .get_cursor_page_query_options(&path, &p, assistants_beta_options())
                    .await
            }
            None => {
                self.client
                    .get_cursor_page_options(&path, assistants_beta_options())
                    .await
            }
        }
    }

    /// Cancels an in-progress run.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn cancel(&self, thread_id: impl AsRef<str>, run_id: impl AsRef<str>) -> Result<Run> {
        let body = serde_json::json!({});
        self.client
            .post_json_beta(
                &format!(
                    "/threads/{}/runs/{}/cancel",
                    urlencoding::encode(thread_id.as_ref()),
                    urlencoding::encode(run_id.as_ref())
                ),
                &body,
            )
            .await
    }

    /// Submits tool outputs for a run requiring action.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn submit_tool_outputs(
        &self,
        thread_id: impl AsRef<str>,
        run_id: impl AsRef<str>,
        params: SubmitToolOutputsParams,
    ) -> Result<Run> {
        self.client
            .post_json_beta(
                &format!(
                    "/threads/{}/runs/{}/submit_tool_outputs",
                    urlencoding::encode(thread_id.as_ref()),
                    urlencoding::encode(run_id.as_ref())
                ),
                &params,
            )
            .await
    }

    /// Submits tool outputs for a run requiring action, returning an SSE stream.
    ///
    /// Forces `stream: true` on the request body.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or stream decode failures.
    pub async fn submit_tool_outputs_stream(
        &self,
        thread_id: impl AsRef<str>,
        run_id: impl AsRef<str>,
        mut params: SubmitToolOutputsParams,
    ) -> Result<impl Stream<Item = Result<AssistantStreamEvent>>> {
        params.stream = Some(true);
        let response = self
            .client
            .post_raw_json_beta(
                &format!(
                    "/threads/{}/runs/{}/submit_tool_outputs",
                    urlencoding::encode(thread_id.as_ref()),
                    urlencoding::encode(run_id.as_ref())
                ),
                &params,
            )
            .await?;
        Ok(SseStream::new(response))
    }
}

// ---------------------------------------------------------------------------
// Thread Run Steps
// ---------------------------------------------------------------------------

/// Thread run steps service (nested under runs).
#[derive(Clone)]
pub struct BetaThreadRunStepService {
    client: Client,
}

impl BetaThreadRunStepService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Gets a run step.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(
        &self,
        thread_id: impl AsRef<str>,
        run_id: impl AsRef<str>,
        step_id: impl AsRef<str>,
    ) -> Result<RunStep> {
        self.client
            .get_json_beta(&format!(
                "/threads/{}/runs/{}/steps/{}",
                urlencoding::encode(thread_id.as_ref()),
                urlencoding::encode(run_id.as_ref()),
                urlencoding::encode(step_id.as_ref())
            ))
            .await
    }

    /// Lists run steps belonging to a run with optional pagination parameters.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(
        &self,
        thread_id: impl AsRef<str>,
        run_id: impl AsRef<str>,
        params: Option<BetaThreadRunStepListParams>,
    ) -> Result<CursorPage<RunStep>> {
        let path = format!(
            "/threads/{}/runs/{}/steps",
            urlencoding::encode(thread_id.as_ref()),
            urlencoding::encode(run_id.as_ref())
        );
        match params {
            Some(p) => {
                self.client
                    .get_cursor_page_query_options(&path, &p, assistants_beta_options())
                    .await
            }
            None => {
                self.client
                    .get_cursor_page_options(&path, assistants_beta_options())
                    .await
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ChatKit Services
// ---------------------------------------------------------------------------

/// ChatKit service — manages sessions and threads for the ChatKit beta.
#[derive(Clone)]
pub struct BetaChatKitService {
    client: Client,
}

impl BetaChatKitService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Returns ChatKit sessions service.
    #[must_use]
    pub fn sessions(&self) -> BetaChatKitSessionService {
        BetaChatKitSessionService::new(self.client.clone())
    }

    /// Returns ChatKit threads service.
    #[must_use]
    pub fn threads(&self) -> BetaChatKitThreadService {
        BetaChatKitThreadService::new(self.client.clone())
    }
}

/// ChatKit session service.
#[derive(Clone)]
pub struct BetaChatKitSessionService {
    client: Client,
}

impl BetaChatKitSessionService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates a ChatKit session.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: ChatKitSessionCreateParams) -> Result<ChatSession> {
        self.client
            .post_json_chatkit_beta("/chatkit/sessions", &params)
            .await
    }

    /// Cancels an active ChatKit session.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn cancel(&self, session_id: impl AsRef<str>) -> Result<ChatSession> {
        let body = serde_json::json!({});
        self.client
            .post_json_chatkit_beta(
                &format!(
                    "/chatkit/sessions/{}/cancel",
                    urlencoding::encode(session_id.as_ref())
                ),
                &body,
            )
            .await
    }
}

/// ChatKit thread service.
#[derive(Clone)]
pub struct BetaChatKitThreadService {
    client: Client,
}

impl BetaChatKitThreadService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Retrieves a ChatKit thread by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, thread_id: impl AsRef<str>) -> Result<ChatKitThread> {
        self.client
            .get_json_chatkit_beta(&format!(
                "/chatkit/threads/{}",
                urlencoding::encode(thread_id.as_ref())
            ))
            .await
    }

    /// Lists ChatKit threads with optional pagination and filters.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(
        &self,
        params: Option<ChatKitThreadListParams>,
    ) -> Result<ConversationCursorPage<ChatKitThread>> {
        match params {
            Some(p) => {
                self.client
                    .get_conversation_cursor_page_query_options(
                        "/chatkit/threads",
                        &p,
                        chatkit_beta_options(),
                    )
                    .await
            }
            None => {
                self.client
                    .get_conversation_cursor_page_options(
                        "/chatkit/threads",
                        chatkit_beta_options(),
                    )
                    .await
            }
        }
    }

    /// Deletes a ChatKit thread.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(&self, thread_id: impl AsRef<str>) -> Result<ChatKitThreadDeleted> {
        self.client
            .delete_json_chatkit_beta(&format!(
                "/chatkit/threads/{}",
                urlencoding::encode(thread_id.as_ref())
            ))
            .await
    }

    /// Lists items belonging to a ChatKit thread.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list_items(
        &self,
        thread_id: impl AsRef<str>,
        params: Option<ChatKitThreadListItemsParams>,
    ) -> Result<ConversationCursorPage<Value>> {
        let path = format!(
            "/chatkit/threads/{}/items",
            urlencoding::encode(thread_id.as_ref())
        );
        match params {
            Some(p) => {
                self.client
                    .get_conversation_cursor_page_query_options(&path, &p, chatkit_beta_options())
                    .await
            }
            None => {
                self.client
                    .get_conversation_cursor_page_options(&path, chatkit_beta_options())
                    .await
            }
        }
    }
}

// ===========================================================================
// Data types
// ===========================================================================

// ---------------------------------------------------------------------------
// Status enums
// ---------------------------------------------------------------------------

/// The status of a run.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    /// The run is queued.
    Queued,
    /// The run is in progress.
    InProgress,
    /// The run requires action (e.g., tool outputs).
    RequiresAction,
    /// The run is being cancelled.
    Cancelling,
    /// The run was cancelled.
    Cancelled,
    /// The run failed.
    Failed,
    /// The run completed successfully.
    Completed,
    /// The run completed but is incomplete.
    Incomplete,
    /// The run expired.
    Expired,
}

/// The status of a run step.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStepStatus {
    /// The step is in progress.
    InProgress,
    /// The step was cancelled.
    Cancelled,
    /// The step failed.
    Failed,
    /// The step completed.
    Completed,
    /// The step expired.
    Expired,
}

/// The type of a run step.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStepType {
    /// The step created a message.
    MessageCreation,
    /// The step involved tool calls.
    ToolCalls,
}

/// The status of a message.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageStatus {
    /// The message is in progress.
    InProgress,
    /// The message is incomplete.
    Incomplete,
    /// The message completed.
    Completed,
}

/// Reasoning effort level for models.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    /// No reasoning.
    None,
    /// Minimal reasoning.
    Minimal,
    /// Low reasoning effort.
    Low,
    /// Medium reasoning effort.
    Medium,
    /// High reasoning effort.
    High,
    /// Extra-high reasoning effort.
    Xhigh,
}

// ---------------------------------------------------------------------------
// Response format / tool choice unions
// ---------------------------------------------------------------------------

/// The format the model must output. Can be "auto" or a typed object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum AssistantResponseFormatOption {
    /// Let the model decide the output format.
    Auto(String),
    /// A typed response format object.
    Typed(ResponseFormatObject),
}

/// A typed response format object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormatObject {
    /// Plain text output.
    Text,
    /// JSON object output.
    JsonObject,
    /// JSON schema output (structured outputs).
    JsonSchema {
        /// The JSON schema definition.
        json_schema: ResponseFormatJsonSchema,
    },
}

/// JSON schema definition for structured outputs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseFormatJsonSchema {
    /// The name of the schema.
    pub name: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The JSON schema object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Value>,
    /// Whether to enable strict schema adherence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

/// Controls which tool is called by the model.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum AssistantToolChoiceOption {
    /// A string keyword: "none", "auto", or "required".
    Keyword(String),
    /// A typed tool choice object specifying a particular tool.
    Typed(AssistantToolChoice),
}

/// A specific tool choice object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssistantToolChoice {
    /// The type of tool (e.g., "function", "code_interpreter", "file_search").
    #[serde(rename = "type")]
    pub tool_type: AssistantToolChoiceType,
    /// Function details (when type is "function").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<AssistantToolChoiceFunction>,
}

/// Tool type for tool choice.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssistantToolChoiceType {
    /// Function tool.
    Function,
    /// Code interpreter tool.
    CodeInterpreter,
    /// File search tool.
    FileSearch,
}

/// Function name for a tool choice.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssistantToolChoiceFunction {
    /// The name of the function.
    pub name: String,
}

// ---------------------------------------------------------------------------
// Truncation strategy
// ---------------------------------------------------------------------------

/// Controls how a thread is truncated before a run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunTruncationStrategy {
    /// The truncation strategy type: "auto" or "last_messages".
    #[serde(rename = "type")]
    pub strategy_type: String,
    /// Number of most recent messages to keep (for "last_messages" type).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_messages: Option<i64>,
}

// ---------------------------------------------------------------------------
// Incomplete details
// ---------------------------------------------------------------------------

/// Details on why a run is incomplete.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunIncompleteDetails {
    /// The reason: "max_completion_tokens" or "max_prompt_tokens".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Details on why a message is incomplete.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageIncompleteDetails {
    /// Reason: "content_filter", "max_tokens", "run_cancelled", "run_expired", "run_failed".
    pub reason: String,
}

// ---------------------------------------------------------------------------
// File search ranking options
// ---------------------------------------------------------------------------

/// Ranking options for file search.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileSearchRankingOptions {
    /// The score threshold (0 to 1).
    pub score_threshold: f64,
    /// The ranker to use. "auto" or "default_2024_08_21".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranker: Option<String>,
}

// ---------------------------------------------------------------------------
// Assistant tool types
// ---------------------------------------------------------------------------

/// An assistant tool definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssistantTool {
    /// Code interpreter tool.
    CodeInterpreter,
    /// File search tool with optional configuration.
    FileSearch {
        /// Optional file search configuration.
        #[serde(skip_serializing_if = "Option::is_none")]
        file_search: Option<FileSearchConfig>,
    },
    /// Function tool.
    Function {
        /// Function definition.
        function: FunctionDefinition,
    },
}

/// Configuration for the file_search tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileSearchConfig {
    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_num_results: Option<u32>,
    /// Ranking options for results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranking_options: Option<FileSearchRankingOptions>,
}

/// A function tool definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionDefinition {
    /// The name of the function.
    pub name: String,
    /// A description of the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The function parameters as a JSON Schema object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
}

/// Tool resources attached to an assistant or thread.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ToolResources {
    /// Code interpreter resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_interpreter: Option<CodeInterpreterResources>,
    /// File search resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_search: Option<FileSearchResources>,
}

/// Code interpreter tool resources.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeInterpreterResources {
    /// File IDs available to the code interpreter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
}

/// File search tool resources.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileSearchResources {
    /// Vector store IDs for file search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_store_ids: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// List params
// ---------------------------------------------------------------------------

/// Pagination parameters for listing assistants.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct BetaAssistantListParams {
    /// Cursor for pagination; fetch items after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Cursor for pagination; fetch items before this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Limit on the number of objects (1-100, default 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Sort order by `created_at` timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<SortOrder>,
}

/// Pagination parameters for listing thread messages.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct BetaThreadMessageListParams {
    /// Cursor for pagination; fetch items after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Cursor for pagination; fetch items before this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Limit on the number of objects (1-100, default 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Sort order by `created_at` timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<SortOrder>,
    /// Filter messages by the run ID that generated them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
}

/// Pagination parameters for listing thread runs.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct BetaThreadRunListParams {
    /// Cursor for pagination; fetch items after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Cursor for pagination; fetch items before this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Limit on the number of objects (1-100, default 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Sort order by `created_at` timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<SortOrder>,
}

/// Pagination parameters for listing run steps.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct BetaThreadRunStepListParams {
    /// Cursor for pagination; fetch items after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Cursor for pagination; fetch items before this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Limit on the number of objects (1-100, default 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Sort order by `created_at` timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<SortOrder>,
    /// Additional fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// Assistant types
// ---------------------------------------------------------------------------

/// Assistant create request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssistantCreateParams {
    /// Model used by assistant.
    pub model: String,
    /// Optional display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional description (max 512 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Optional tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AssistantTool>>,
    /// Optional tool resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<ToolResources>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Optional temperature (0-2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Optional top_p (nucleus sampling).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Optional reasoning effort.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Optional response format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<AssistantResponseFormatOption>,
}

/// Assistant update request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssistantUpdateParams {
    /// Optional model override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Optional display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Optional tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AssistantTool>>,
    /// Optional tool resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<ToolResources>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Optional temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Optional top_p.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Optional reasoning effort.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Optional response format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<AssistantResponseFormatOption>,
}

/// Assistant object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Assistant {
    /// Assistant ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Unix timestamp (seconds) for when the assistant was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// Model ID.
    pub model: String,
    /// Optional assistant name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional assistant description (max 512 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional assistant instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Tools enabled on the assistant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AssistantTool>>,
    /// Tool resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<ToolResources>,
    /// Response format configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<AssistantResponseFormatOption>,
    /// Optional temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Optional top_p.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
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

// ---------------------------------------------------------------------------
// Thread types
// ---------------------------------------------------------------------------

/// Thread create request.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ThreadCreateParams {
    /// Optional messages to prepopulate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<MessageCreateParams>>,
    /// Optional tool resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<ToolResources>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Thread update request.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ThreadUpdateParams {
    /// Optional tool resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<ToolResources>,
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
    /// Unix timestamp (seconds) for when the thread was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// Optional tool resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<ToolResources>,
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

/// Create thread and run in one request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThreadCreateAndRunParams {
    /// Assistant ID to run.
    pub assistant_id: String,
    /// Thread to create.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<ThreadCreateParams>,
    /// Optional model override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Optional instructions override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Optional tools override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AssistantTool>>,
    /// Whether to stream the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Optional temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Optional top_p.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Optional max completion tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<i64>,
    /// Optional max prompt tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_prompt_tokens: Option<i64>,
    /// Optional response format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<AssistantResponseFormatOption>,
    /// Optional tool choice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<AssistantToolChoiceOption>,
    /// Optional truncation strategy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation_strategy: Option<RunTruncationStrategy>,
}

// ---------------------------------------------------------------------------
// Message types
// ---------------------------------------------------------------------------

/// The role of a message sender.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// User-submitted message.
    User,
    /// Assistant-generated message.
    Assistant,
}

/// Message content types.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
    /// Text content block.
    Text {
        /// The text value with annotations.
        text: TextContent,
    },
    /// Image file reference.
    ImageFile {
        /// Image file details.
        image_file: ImageFileContent,
    },
    /// Image URL reference.
    ImageUrl {
        /// Image URL details.
        image_url: ImageUrlContent,
    },
    /// Refusal content block.
    Refusal {
        /// The refusal text.
        refusal: String,
    },
}

/// Text content with optional annotations.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TextContent {
    /// The text value.
    pub value: String,
    /// Annotations within the text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<Annotation>>,
}

/// An annotation within message text.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Annotation {
    /// A file citation annotation.
    FileCitation {
        /// The text being annotated.
        text: String,
        /// Citation details.
        file_citation: FileCitationBody,
        /// Start offset in the text.
        start_index: u64,
        /// End offset in the text.
        end_index: u64,
    },
    /// A file path annotation.
    FilePath {
        /// The text being annotated.
        text: String,
        /// File path details.
        file_path: FilePathBody,
        /// Start offset in the text.
        start_index: u64,
        /// End offset in the text.
        end_index: u64,
    },
}

/// Body of a file citation annotation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileCitationBody {
    /// The ID of the file being cited.
    pub file_id: String,
}

/// Body of a file path annotation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FilePathBody {
    /// The ID of the generated file.
    pub file_id: String,
}

/// Image file reference content.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageFileContent {
    /// The file ID for the image.
    pub file_id: String,
    /// Optional detail level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Image URL content.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageUrlContent {
    /// External image URL.
    pub url: String,
    /// Optional detail level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Message create params.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageCreateParams {
    /// The role of the message sender.
    pub role: MessageRole,
    /// The text content of the message.
    pub content: String,
    /// Optional file attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<MessageAttachment>>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// A file attachment on a message.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageAttachment {
    /// The file ID to attach.
    pub file_id: String,
    /// The tools this file should be available to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<MessageAttachmentTool>>,
}

/// Tool reference for a message attachment.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageAttachmentTool {
    /// Code interpreter tool.
    CodeInterpreter,
    /// File search tool.
    FileSearch,
}

/// Message update params (metadata only).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MessageUpdateParams {
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Message object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    /// Message ID.
    pub id: String,
    /// Object type (`thread.message`).
    pub object: String,
    /// Unix timestamp (seconds) for when the message was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// Thread this message belongs to.
    pub thread_id: String,
    /// Role of the author.
    pub role: MessageRole,
    /// Message content blocks.
    pub content: Vec<MessageContent>,
    /// Optional assistant ID if assistant authored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant_id: Option<String>,
    /// Optional run ID associated with creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    /// Message status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MessageStatus>,
    /// Attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<MessageAttachment>>,
    /// Metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Unix timestamp (seconds) for when the message was completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    /// Unix timestamp (seconds) for when the message was marked incomplete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incomplete_at: Option<i64>,
    /// Details on why the message is incomplete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incomplete_details: Option<MessageIncompleteDetails>,
}

/// Message deletion response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageDeleted {
    /// Message ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Deletion flag.
    pub deleted: bool,
}

// ---------------------------------------------------------------------------
// Run types
// ---------------------------------------------------------------------------

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
    /// Additional instructions appended to the run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_instructions: Option<String>,
    /// Additional messages to add before the run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_messages: Option<Vec<MessageCreateParams>>,
    /// Optional tools override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AssistantTool>>,
    /// Whether to stream the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Maximum completion tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<i64>,
    /// Maximum prompt tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_prompt_tokens: Option<i64>,
    /// Sampling temperature (0-2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Nucleus sampling (top_p).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Whether to enable parallel function calling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    /// Truncation strategy for the thread context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation_strategy: Option<RunTruncationStrategy>,
    /// Response format constraint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<AssistantResponseFormatOption>,
    /// Tool choice constraint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<AssistantToolChoiceOption>,
    /// Reasoning effort level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Additional fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}

/// Run update request (metadata only).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct BetaThreadRunUpdateParams {
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Tool output for `submit_tool_outputs`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolOutput {
    /// The tool call ID.
    pub tool_call_id: String,
    /// The output to submit.
    pub output: String,
}

/// Submit tool outputs params.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubmitToolOutputsParams {
    /// The tool outputs to submit.
    pub tool_outputs: Vec<ToolOutput>,
    /// Whether to stream the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// Thread run object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Run {
    /// Run ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Unix timestamp (seconds) for when the run was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// Thread ID.
    pub thread_id: String,
    /// Assistant ID.
    pub assistant_id: String,
    /// Run status.
    pub status: RunStatus,
    /// Model used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Instructions used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Tools used in the run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AssistantTool>>,
    /// Required action (e.g., submit_tool_outputs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_action: Option<RequiredAction>,
    /// Last error, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<RunLastError>,
    /// Usage statistics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<RunUsage>,
    /// Metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Unix timestamp (seconds) for when the run was cancelled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled_at: Option<i64>,
    /// Unix timestamp (seconds) for when the run completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    /// Unix timestamp (seconds) for when the run will expire.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    /// Unix timestamp (seconds) for when the run failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_at: Option<i64>,
    /// Unix timestamp (seconds) for when the run started.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<i64>,
    /// Maximum completion tokens for the run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<i64>,
    /// Maximum prompt tokens for the run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_prompt_tokens: Option<i64>,
    /// Whether parallel function calling is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    /// Response format constraint used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<AssistantResponseFormatOption>,
    /// Tool choice constraint used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<AssistantToolChoiceOption>,
    /// Truncation strategy used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation_strategy: Option<RunTruncationStrategy>,
    /// Sampling temperature used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Nucleus sampling value used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Details on why the run is incomplete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incomplete_details: Option<RunIncompleteDetails>,
}

/// A required action on a run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequiredAction {
    /// Action type.
    #[serde(rename = "type")]
    pub action_type: String,
    /// Tool outputs needed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submit_tool_outputs: Option<SubmitToolOutputsAction>,
}

/// Details of tool calls that need outputs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubmitToolOutputsAction {
    /// The tool calls requiring outputs.
    pub tool_calls: Vec<RunToolCall>,
}

/// A tool call within a run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunToolCall {
    /// Tool call ID.
    pub id: String,
    /// Tool call type.
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function call details (for function tool calls).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<RunToolCallFunction>,
}

/// Function call details in a tool call.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunToolCallFunction {
    /// Function name.
    pub name: String,
    /// Stringified arguments.
    pub arguments: String,
}

/// Run error information.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunLastError {
    /// Error code.
    pub code: String,
    /// Error message.
    pub message: String,
}

/// Usage statistics for a run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunUsage {
    /// Prompt tokens consumed.
    pub prompt_tokens: u64,
    /// Completion tokens consumed.
    pub completion_tokens: u64,
    /// Total tokens consumed.
    pub total_tokens: u64,
}

// ---------------------------------------------------------------------------
// Run Step types
// ---------------------------------------------------------------------------

/// Run step object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunStep {
    /// Run step ID.
    pub id: String,
    /// Object type (`thread.run.step`).
    pub object: String,
    /// Unix timestamp (seconds) for when the step was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// Thread ID.
    pub thread_id: String,
    /// Run ID.
    pub run_id: String,
    /// Assistant ID.
    pub assistant_id: String,
    /// Step type.
    #[serde(rename = "type")]
    pub step_type: RunStepType,
    /// Step status.
    pub status: RunStepStatus,
    /// Step details.
    pub step_details: StepDetails,
    /// Last error, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<RunLastError>,
    /// Usage statistics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<RunUsage>,
    /// Metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Unix timestamp (seconds) for when the step was cancelled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled_at: Option<i64>,
    /// Unix timestamp (seconds) for when the step completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    /// Unix timestamp (seconds) for when the step expired.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expired_at: Option<i64>,
    /// Unix timestamp (seconds) for when the step failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_at: Option<i64>,
}

/// Step details -- either message creation or tool calls.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepDetails {
    /// The step created a message.
    MessageCreation {
        /// Details about the message created.
        message_creation: MessageCreationDetail,
    },
    /// The step involved tool calls.
    ToolCalls {
        /// The tool calls made.
        tool_calls: Vec<StepToolCall>,
    },
}

/// Details of a message creation step.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageCreationDetail {
    /// The ID of the message created.
    pub message_id: String,
}

/// A tool call within a run step.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepToolCall {
    /// Code interpreter tool call.
    CodeInterpreter {
        /// Tool call ID.
        id: String,
        /// Code interpreter details.
        code_interpreter: CodeInterpreterCall,
    },
    /// File search tool call.
    FileSearch {
        /// Tool call ID.
        id: String,
        /// File search details (opaque).
        file_search: Value,
    },
    /// Function tool call.
    Function {
        /// Tool call ID.
        id: String,
        /// Function call details.
        function: FunctionCallDetail,
    },
}

/// Code interpreter call details.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeInterpreterCall {
    /// Input code.
    pub input: String,
    /// Output items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Vec<CodeInterpreterOutput>>,
}

/// Code interpreter output.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CodeInterpreterOutput {
    /// Text/log output.
    Logs {
        /// Log text.
        logs: String,
    },
    /// Image output.
    Image {
        /// Image details.
        image: CodeInterpreterImage,
    },
}

/// Code interpreter image output.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeInterpreterImage {
    /// File ID of the generated image.
    pub file_id: String,
}

/// Function call details within a run step.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionCallDetail {
    /// Function name.
    pub name: String,
    /// Stringified arguments.
    pub arguments: String,
    /// Function output (null until submitted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

// ---------------------------------------------------------------------------
// ChatKit types
// ---------------------------------------------------------------------------

/// Parameters for creating a ChatKit session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatKitSessionCreateParams {
    /// User identifier for the session.
    pub user: String,
    /// Workflow that powers the session.
    pub workflow: ChatSessionWorkflowParam,
    /// Optional ChatKit runtime configuration overrides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chatkit_configuration: Option<ChatSessionChatKitConfigurationParam>,
    /// Optional session expiration timing override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<ChatSessionExpiresAfterParam>,
    /// Optional per-minute request limit override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limits: Option<ChatSessionRateLimitsParam>,
}

/// Workflow reference for a ChatKit session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSessionWorkflowParam {
    /// Identifier for the workflow.
    pub id: String,
    /// Specific workflow version to run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// State variables forwarded to the workflow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_variables: Option<HashMap<String, Value>>,
    /// Optional tracing overrides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracing: Option<ChatSessionWorkflowTracingParam>,
}

/// Tracing override for a workflow invocation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSessionWorkflowTracingParam {
    /// Whether tracing is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// Optional ChatKit configuration for a session.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ChatSessionChatKitConfigurationParam {
    /// Automatic thread titling preferences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automatic_thread_titling: Option<ChatSessionAutoThreadTitlingParam>,
    /// File upload configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_upload: Option<ChatSessionFileUploadParam>,
    /// History retention configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<ChatSessionHistoryParam>,
}

/// Automatic thread titling configuration parameter.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSessionAutoThreadTitlingParam {
    /// Whether automatic thread titling is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// File upload configuration parameter.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSessionFileUploadParam {
    /// Whether uploads are enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Maximum file size in megabytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_file_size: Option<i64>,
    /// Maximum number of files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_files: Option<i64>,
}

/// History retention configuration parameter.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSessionHistoryParam {
    /// Whether history is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Number of recent threads to retain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_threads: Option<i64>,
}

/// Session expiration timing parameter.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSessionExpiresAfterParam {
    /// Number of seconds after the anchor when the session expires.
    pub seconds: i64,
    /// Base timestamp used to calculate expiration (always `created_at`).
    #[serde(default = "default_created_at")]
    pub anchor: String,
}

fn default_created_at() -> String {
    "created_at".to_owned()
}

/// Rate limits parameter for a session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSessionRateLimitsParam {
    /// Maximum requests per minute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_requests_per_1_minute: Option<i64>,
}

/// A ChatKit session and its resolved configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSession {
    /// Session identifier.
    pub id: String,
    /// Object type (`chatkit.session`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// Ephemeral client secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    /// Unix timestamp for when the session expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    /// Per-minute request limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_requests_per_1_minute: Option<i64>,
    /// Resolved rate limits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limits: Option<ChatSessionRateLimits>,
    /// Session lifecycle status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ChatSessionStatus>,
    /// User identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Workflow metadata for the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<ChatKitWorkflow>,
    /// Resolved ChatKit configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chatkit_configuration: Option<ChatSessionChatKitConfiguration>,
}

/// Resolved ChatKit configuration for a session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSessionChatKitConfiguration {
    /// Automatic thread titling preferences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automatic_thread_titling: Option<ChatSessionAutoThreadTitling>,
    /// File upload settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_upload: Option<ChatSessionFileUpload>,
    /// History retention settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<ChatSessionHistory>,
}

/// Automatic thread titling preferences.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSessionAutoThreadTitling {
    /// Whether automatic thread titling is enabled.
    pub enabled: bool,
}

/// File upload settings for a session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSessionFileUpload {
    /// Whether uploads are enabled.
    pub enabled: bool,
    /// Maximum file size in megabytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_file_size: Option<i64>,
    /// Maximum number of uploads.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_files: Option<i64>,
}

/// History retention settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSessionHistory {
    /// Whether history is enabled.
    pub enabled: bool,
    /// Number of recent threads retained.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_threads: Option<i64>,
}

/// Rate limits for a session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSessionRateLimits {
    /// Maximum requests per minute.
    pub max_requests_per_1_minute: i64,
}

/// Session lifecycle status.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatSessionStatus {
    /// Session is active.
    Active,
    /// Session has expired.
    Expired,
    /// Session was cancelled.
    Cancelled,
}

/// Workflow metadata returned for a session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatKitWorkflow {
    /// Workflow identifier.
    pub id: String,
    /// Workflow version used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// State variable key-value pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_variables: Option<HashMap<String, Value>>,
    /// Tracing settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracing: Option<ChatKitWorkflowTracing>,
}

/// Tracing settings for a workflow.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatKitWorkflowTracing {
    /// Whether tracing is enabled.
    pub enabled: bool,
}

/// A ChatKit thread.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatKitThread {
    /// Thread identifier.
    pub id: String,
    /// Object type (`chatkit.thread`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// Unix timestamp for when the thread was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// Thread status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ChatKitThreadStatus>,
    /// Optional title for the thread.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// User identifier who owns the thread.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// Status of a ChatKit thread.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ChatKitThreadStatus {
    /// Thread is active.
    Active,
    /// Thread is locked (with optional reason).
    Locked {
        /// Reason for locking.
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    /// Thread is closed (with optional reason).
    Closed {
        /// Reason for closing.
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
}

/// ChatKit thread deletion response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatKitThreadDeleted {
    /// Thread identifier.
    pub id: String,
    /// Object type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// Whether the thread was deleted.
    pub deleted: bool,
}

/// Parameters for listing ChatKit threads.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ChatKitThreadListParams {
    /// Cursor: list threads after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Cursor: list threads before this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Maximum number of threads to return (default 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Filter by user identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Sort order by creation time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<SortOrder>,
}

/// Parameters for listing items within a ChatKit thread.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ChatKitThreadListItemsParams {
    /// Cursor: list items after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Cursor: list items before this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Maximum number of items to return (default 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Sort order by creation time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<SortOrder>,
}

// ---------------------------------------------------------------------------
// Delta / streaming types
// ---------------------------------------------------------------------------

/// A message delta event received during streaming.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageDeltaEvent {
    /// The message ID this delta applies to.
    pub id: String,
    /// Object type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The incremental delta content.
    pub delta: MessageDelta,
}

/// Incremental update to a message.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageDelta {
    /// The role of the message author, if changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<MessageRole>,
    /// Incremental content blocks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<MessageContentDelta>>,
}

/// A delta content block within a message.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContentDelta {
    /// Text delta content block.
    Text {
        /// Index of this content block within the message.
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<u64>,
        /// Incremental text changes.
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<TextDelta>,
    },
    /// Image file delta content block.
    ImageFile {
        /// Index of this content block.
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<u64>,
        /// Image file delta details.
        #[serde(skip_serializing_if = "Option::is_none")]
        image_file: Option<Value>,
    },
    /// Refusal delta content block.
    Refusal {
        /// Index of this content block.
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<u64>,
        /// Refusal text delta.
        #[serde(skip_serializing_if = "Option::is_none")]
        refusal: Option<String>,
    },
}

/// Incremental text content.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TextDelta {
    /// The incremental text value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Incremental annotations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<AnnotationDelta>>,
}

/// An incremental annotation within text.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnnotationDelta {
    /// File citation annotation delta.
    FileCitation {
        /// Index of this annotation.
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<u64>,
        /// The annotated text.
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        /// Citation details.
        #[serde(skip_serializing_if = "Option::is_none")]
        file_citation: Option<FileCitationBody>,
        /// Start offset in the text.
        #[serde(skip_serializing_if = "Option::is_none")]
        start_index: Option<u64>,
        /// End offset in the text.
        #[serde(skip_serializing_if = "Option::is_none")]
        end_index: Option<u64>,
    },
    /// File path annotation delta.
    FilePath {
        /// Index of this annotation.
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<u64>,
        /// The annotated text.
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        /// File path details.
        #[serde(skip_serializing_if = "Option::is_none")]
        file_path: Option<FilePathBody>,
        /// Start offset in the text.
        #[serde(skip_serializing_if = "Option::is_none")]
        start_index: Option<u64>,
        /// End offset in the text.
        #[serde(skip_serializing_if = "Option::is_none")]
        end_index: Option<u64>,
    },
}

/// A run step delta event received during streaming.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunStepDeltaEvent {
    /// The run step ID this delta applies to.
    pub id: String,
    /// Object type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The incremental delta content.
    pub delta: RunStepDelta,
}

/// Incremental update to a run step.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunStepDelta {
    /// Incremental step details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_details: Option<StepDetailsDelta>,
}

/// Delta step details -- either message creation or tool calls.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepDetailsDelta {
    /// Message creation step delta.
    MessageCreation {
        /// Message creation details delta.
        #[serde(skip_serializing_if = "Option::is_none")]
        message_creation: Option<Value>,
    },
    /// Tool calls step delta.
    ToolCalls {
        /// Incremental tool call deltas.
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCallDelta>>,
    },
}

/// A delta for a single tool call within a run step.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolCallDelta {
    /// Code interpreter tool call delta.
    CodeInterpreter {
        /// Index of this tool call.
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<u64>,
        /// Tool call ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        /// Code interpreter delta details.
        #[serde(skip_serializing_if = "Option::is_none")]
        code_interpreter: Option<Value>,
    },
    /// File search tool call delta.
    FileSearch {
        /// Index of this tool call.
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<u64>,
        /// Tool call ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        /// File search delta details.
        #[serde(skip_serializing_if = "Option::is_none")]
        file_search: Option<Value>,
    },
    /// Function tool call delta.
    Function {
        /// Index of this tool call.
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<u64>,
        /// Tool call ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        /// Function delta details.
        #[serde(skip_serializing_if = "Option::is_none")]
        function: Option<FunctionCallDelta>,
    },
}

/// Incremental function call details.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionCallDelta {
    /// Function name (may be partial).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Stringified arguments (may be partial).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    /// Function output (may be partial).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

/// A typed event from the assistants streaming API.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum AssistantStreamEvent {
    /// A thread was created.
    #[serde(rename = "thread.created")]
    ThreadCreated {
        /// The thread object.
        data: Value,
    },
    /// A run was created.
    #[serde(rename = "thread.run.created")]
    ThreadRunCreated {
        /// The run object.
        data: Value,
    },
    /// A run was queued.
    #[serde(rename = "thread.run.queued")]
    ThreadRunQueued {
        /// The run object.
        data: Value,
    },
    /// A run is in progress.
    #[serde(rename = "thread.run.in_progress")]
    ThreadRunInProgress {
        /// The run object.
        data: Value,
    },
    /// A run requires action (tool outputs).
    #[serde(rename = "thread.run.requires_action")]
    ThreadRunRequiresAction {
        /// The run object.
        data: Value,
    },
    /// A run completed successfully.
    #[serde(rename = "thread.run.completed")]
    ThreadRunCompleted {
        /// The run object.
        data: Value,
    },
    /// A run completed but is incomplete.
    #[serde(rename = "thread.run.incomplete")]
    ThreadRunIncomplete {
        /// The run object.
        data: Value,
    },
    /// A run failed.
    #[serde(rename = "thread.run.failed")]
    ThreadRunFailed {
        /// The run object.
        data: Value,
    },
    /// A run is being cancelled.
    #[serde(rename = "thread.run.cancelling")]
    ThreadRunCancelling {
        /// The run object.
        data: Value,
    },
    /// A run was cancelled.
    #[serde(rename = "thread.run.cancelled")]
    ThreadRunCancelled {
        /// The run object.
        data: Value,
    },
    /// A run expired.
    #[serde(rename = "thread.run.expired")]
    ThreadRunExpired {
        /// The run object.
        data: Value,
    },
    /// A run step was created.
    #[serde(rename = "thread.run.step.created")]
    ThreadRunStepCreated {
        /// The run step object.
        data: Value,
    },
    /// A run step is in progress.
    #[serde(rename = "thread.run.step.in_progress")]
    ThreadRunStepInProgress {
        /// The run step object.
        data: Value,
    },
    /// A run step delta (incremental update).
    #[serde(rename = "thread.run.step.delta")]
    ThreadRunStepDelta {
        /// The typed delta event data.
        data: RunStepDeltaEvent,
    },
    /// A run step completed.
    #[serde(rename = "thread.run.step.completed")]
    ThreadRunStepCompleted {
        /// The run step object.
        data: Value,
    },
    /// A run step failed.
    #[serde(rename = "thread.run.step.failed")]
    ThreadRunStepFailed {
        /// The run step object.
        data: Value,
    },
    /// A run step was cancelled.
    #[serde(rename = "thread.run.step.cancelled")]
    ThreadRunStepCancelled {
        /// The run step object.
        data: Value,
    },
    /// A run step expired.
    #[serde(rename = "thread.run.step.expired")]
    ThreadRunStepExpired {
        /// The run step object.
        data: Value,
    },
    /// A message was created.
    #[serde(rename = "thread.message.created")]
    ThreadMessageCreated {
        /// The message object.
        data: Value,
    },
    /// A message is in progress.
    #[serde(rename = "thread.message.in_progress")]
    ThreadMessageInProgress {
        /// The message object.
        data: Value,
    },
    /// A message delta (incremental update).
    #[serde(rename = "thread.message.delta")]
    ThreadMessageDelta {
        /// The typed delta event data.
        data: MessageDeltaEvent,
    },
    /// A message completed.
    #[serde(rename = "thread.message.completed")]
    ThreadMessageCompleted {
        /// The message object.
        data: Value,
    },
    /// A message is incomplete.
    #[serde(rename = "thread.message.incomplete")]
    ThreadMessageIncomplete {
        /// The message object.
        data: Value,
    },
    /// An error event.
    #[serde(rename = "error")]
    Error {
        /// The error object.
        data: Value,
    },
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assistant_create_omits_optional_fields() {
        let params = AssistantCreateParams {
            model: "gpt-4o-mini".to_owned(),
            name: None,
            description: None,
            instructions: None,
            tools: None,
            tool_resources: None,
            metadata: None,
            temperature: None,
            top_p: None,
            reasoning_effort: None,
            response_format: None,
        };

        let value = serde_json::to_value(params).expect("serialize assistant params");
        assert!(value.get("name").is_none());
        assert!(value.get("instructions").is_none());
        assert!(value.get("tools").is_none());
        assert!(value.get("tool_resources").is_none());
        assert!(value.get("metadata").is_none());
        assert!(value.get("description").is_none());
        assert!(value.get("temperature").is_none());
        assert!(value.get("top_p").is_none());
        assert!(value.get("reasoning_effort").is_none());
        assert!(value.get("response_format").is_none());
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
            additional_instructions: None,
            additional_messages: None,
            tools: None,
            stream: None,
            metadata: None,
            max_completion_tokens: None,
            max_prompt_tokens: None,
            temperature: None,
            top_p: None,
            parallel_tool_calls: None,
            truncation_strategy: None,
            response_format: None,
            tool_choice: None,
            reasoning_effort: None,
            include: None,
        };

        let value = serde_json::to_value(params).expect("serialize run create params");
        assert!(value.get("model").is_none());
        assert!(value.get("instructions").is_none());
        assert!(value.get("tools").is_none());
        assert!(value.get("stream").is_none());
        assert!(value.get("additional_instructions").is_none());
        assert!(value.get("max_completion_tokens").is_none());
        assert!(value.get("temperature").is_none());
        assert!(value.get("parallel_tool_calls").is_none());
        assert!(value.get("truncation_strategy").is_none());
        assert!(value.get("response_format").is_none());
        assert!(value.get("tool_choice").is_none());
    }

    #[test]
    fn run_deserializes_status_enum() {
        let json = r#"{
            "id":"run_1",
            "object":"thread.run",
            "thread_id":"thread_1",
            "assistant_id":"asst_1",
            "status":"queued"
        }"#;

        let run: Run = serde_json::from_str(json).expect("deserialize run");
        assert_eq!(run.status, RunStatus::Queued);
    }

    #[test]
    fn run_deserializes_all_statuses() {
        for (status_str, expected) in [
            ("queued", RunStatus::Queued),
            ("in_progress", RunStatus::InProgress),
            ("requires_action", RunStatus::RequiresAction),
            ("cancelling", RunStatus::Cancelling),
            ("cancelled", RunStatus::Cancelled),
            ("failed", RunStatus::Failed),
            ("completed", RunStatus::Completed),
            ("incomplete", RunStatus::Incomplete),
            ("expired", RunStatus::Expired),
        ] {
            let json = format!(
                r#"{{"id":"r","object":"thread.run","thread_id":"t","assistant_id":"a","status":"{}"}}"#,
                status_str
            );
            let run: Run = serde_json::from_str(&json).expect("deserialize run status");
            assert_eq!(run.status, expected);
        }
    }

    #[test]
    fn run_deserializes_timestamps() {
        let json = r#"{
            "id":"run_1",
            "object":"thread.run",
            "thread_id":"thread_1",
            "assistant_id":"asst_1",
            "status":"completed",
            "created_at":1700000000,
            "cancelled_at":null,
            "completed_at":1700000100,
            "started_at":1700000050,
            "expires_at":1700001000,
            "failed_at":null
        }"#;
        let run: Run = serde_json::from_str(json).expect("deserialize run with timestamps");
        assert_eq!(run.created_at, Some(1_700_000_000));
        assert_eq!(run.completed_at, Some(1_700_000_100));
        assert_eq!(run.started_at, Some(1_700_000_050));
        assert!(run.cancelled_at.is_none());
        assert!(run.failed_at.is_none());
    }

    #[test]
    fn run_truncation_strategy_roundtrips() {
        let strategy = RunTruncationStrategy {
            strategy_type: "last_messages".to_owned(),
            last_messages: Some(10),
        };
        let value = serde_json::to_value(&strategy).expect("serialize");
        assert_eq!(value["type"], "last_messages");
        assert_eq!(value["last_messages"], 10);

        let parsed: RunTruncationStrategy = serde_json::from_value(value).expect("deserialize");
        assert_eq!(parsed.strategy_type, "last_messages");
        assert_eq!(parsed.last_messages, Some(10));
    }

    #[test]
    fn run_incomplete_details_roundtrips() {
        let details = RunIncompleteDetails {
            reason: Some("max_completion_tokens".to_owned()),
        };
        let value = serde_json::to_value(&details).expect("serialize");
        assert_eq!(value["reason"], "max_completion_tokens");
    }

    #[test]
    fn message_incomplete_details_roundtrips() {
        let details = MessageIncompleteDetails {
            reason: "content_filter".to_owned(),
        };
        let value = serde_json::to_value(&details).expect("serialize");
        assert_eq!(value["reason"], "content_filter");
    }

    #[test]
    fn response_format_auto_roundtrips() {
        let fmt = AssistantResponseFormatOption::Auto("auto".to_owned());
        let value = serde_json::to_value(&fmt).expect("serialize");
        assert_eq!(value, "auto");
    }

    #[test]
    fn response_format_json_object_roundtrips() {
        let fmt = AssistantResponseFormatOption::Typed(ResponseFormatObject::JsonObject);
        let value = serde_json::to_value(&fmt).expect("serialize");
        assert_eq!(value["type"], "json_object");
    }

    #[test]
    fn tool_choice_keyword_roundtrips() {
        let choice = AssistantToolChoiceOption::Keyword("auto".to_owned());
        let value = serde_json::to_value(&choice).expect("serialize");
        assert_eq!(value, "auto");
    }

    #[test]
    fn tool_choice_typed_roundtrips() {
        let choice = AssistantToolChoiceOption::Typed(AssistantToolChoice {
            tool_type: AssistantToolChoiceType::Function,
            function: Some(AssistantToolChoiceFunction {
                name: "get_weather".to_owned(),
            }),
        });
        let value = serde_json::to_value(&choice).expect("serialize");
        assert_eq!(value["type"], "function");
        assert_eq!(value["function"]["name"], "get_weather");
    }

    #[test]
    fn file_search_ranking_options_roundtrips() {
        let opts = FileSearchRankingOptions {
            score_threshold: 0.5,
            ranker: Some("auto".to_owned()),
        };
        let value = serde_json::to_value(&opts).expect("serialize");
        assert_eq!(value["score_threshold"], 0.5);
        assert_eq!(value["ranker"], "auto");
    }

    #[test]
    fn message_create_params_serializes_correctly() {
        let params = MessageCreateParams {
            role: MessageRole::User,
            content: "Hello!".to_owned(),
            attachments: None,
            metadata: None,
        };

        let value = serde_json::to_value(&params).expect("serialize message create");
        assert_eq!(value["role"], "user");
        assert_eq!(value["content"], "Hello!");
        assert!(value.get("attachments").is_none());
        assert!(value.get("metadata").is_none());
    }

    #[test]
    fn message_deserializes_text_content() {
        let json = r#"{
            "id":"msg_1",
            "object":"thread.message",
            "thread_id":"thread_1",
            "role":"user",
            "content":[{
                "type":"text",
                "text":{"value":"Hello world","annotations":[]}
            }]
        }"#;

        let msg: Message = serde_json::from_str(json).expect("deserialize message");
        assert_eq!(msg.id, "msg_1");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content.len(), 1);
        match &msg.content[0] {
            MessageContent::Text { text } => assert_eq!(text.value, "Hello world"),
            _ => panic!("expected text content"),
        }
    }

    #[test]
    fn message_deserializes_image_file_content() {
        let json = r#"{
            "id":"msg_2",
            "object":"thread.message",
            "thread_id":"thread_1",
            "role":"assistant",
            "content":[{
                "type":"image_file",
                "image_file":{"file_id":"file-abc123"}
            }]
        }"#;

        let msg: Message = serde_json::from_str(json).expect("deserialize image message");
        match &msg.content[0] {
            MessageContent::ImageFile { image_file } => {
                assert_eq!(image_file.file_id, "file-abc123");
            }
            _ => panic!("expected image_file content"),
        }
    }

    #[test]
    fn message_deserializes_image_url_content() {
        let json = r#"{
            "id":"msg_3",
            "object":"thread.message",
            "thread_id":"thread_1",
            "role":"user",
            "content":[{
                "type":"image_url",
                "image_url":{"url":"https://example.com/image.png"}
            }]
        }"#;

        let msg: Message = serde_json::from_str(json).expect("deserialize image_url message");
        match &msg.content[0] {
            MessageContent::ImageUrl { image_url } => {
                assert_eq!(image_url.url, "https://example.com/image.png");
            }
            _ => panic!("expected image_url content"),
        }
    }

    #[test]
    fn message_deserializes_refusal_content() {
        let json = r#"{
            "id":"msg_4",
            "object":"thread.message",
            "thread_id":"thread_1",
            "role":"assistant",
            "content":[{
                "type":"refusal",
                "refusal":"I cannot help with that request."
            }]
        }"#;

        let msg: Message = serde_json::from_str(json).expect("deserialize refusal message");
        match &msg.content[0] {
            MessageContent::Refusal { refusal } => {
                assert_eq!(refusal, "I cannot help with that request.");
            }
            _ => panic!("expected refusal content"),
        }
    }

    #[test]
    fn message_deserializes_with_timestamps_and_status() {
        let json = r#"{
            "id":"msg_5",
            "object":"thread.message",
            "thread_id":"thread_1",
            "role":"assistant",
            "content":[],
            "created_at":1700000000,
            "completed_at":1700000100,
            "incomplete_at":null,
            "status":"completed"
        }"#;

        let msg: Message = serde_json::from_str(json).expect("deserialize message timestamps");
        assert_eq!(msg.created_at, Some(1_700_000_000));
        assert_eq!(msg.completed_at, Some(1_700_000_100));
        assert!(msg.incomplete_at.is_none());
        assert_eq!(msg.status, Some(MessageStatus::Completed));
    }

    #[test]
    fn message_update_params_default_is_empty() {
        let value =
            serde_json::to_value(MessageUpdateParams::default()).expect("serialize message update");
        assert_eq!(value, serde_json::json!({}));
    }

    #[test]
    fn message_deleted_deserializes() {
        let json = r#"{"id":"msg_1","object":"thread.message.deleted","deleted":true}"#;
        let deleted: MessageDeleted = serde_json::from_str(json).expect("deserialize deleted");
        assert!(deleted.deleted);
        assert_eq!(deleted.id, "msg_1");
    }

    #[test]
    fn annotation_file_citation_roundtrips() {
        let json = r#"{
            "type":"file_citation",
            "text":"[1]",
            "file_citation":{"file_id":"file-abc"},
            "start_index":10,
            "end_index":13
        }"#;

        let ann: Annotation = serde_json::from_str(json).expect("deserialize annotation");
        match &ann {
            Annotation::FileCitation {
                text,
                file_citation,
                start_index,
                end_index,
            } => {
                assert_eq!(text, "[1]");
                assert_eq!(file_citation.file_id, "file-abc");
                assert_eq!(*start_index, 10);
                assert_eq!(*end_index, 13);
            }
            _ => panic!("expected file_citation"),
        }

        let serialized = serde_json::to_value(&ann).expect("re-serialize");
        assert_eq!(serialized["type"], "file_citation");
    }

    #[test]
    fn annotation_file_path_roundtrips() {
        let json = r#"{
            "type":"file_path",
            "text":"sandbox:/output.csv",
            "file_path":{"file_id":"file-xyz"},
            "start_index":0,
            "end_index":19
        }"#;

        let ann: Annotation = serde_json::from_str(json).expect("deserialize file_path");
        match &ann {
            Annotation::FilePath { file_path, .. } => {
                assert_eq!(file_path.file_id, "file-xyz");
            }
            _ => panic!("expected file_path"),
        }
    }

    #[test]
    fn run_step_deserializes_message_creation() {
        let json = r#"{
            "id":"step_1",
            "object":"thread.run.step",
            "thread_id":"thread_1",
            "run_id":"run_1",
            "assistant_id":"asst_1",
            "type":"message_creation",
            "status":"completed",
            "step_details":{
                "type":"message_creation",
                "message_creation":{"message_id":"msg_1"}
            }
        }"#;

        let step: RunStep = serde_json::from_str(json).expect("deserialize run step");
        assert_eq!(step.id, "step_1");
        assert_eq!(step.step_type, RunStepType::MessageCreation);
        assert_eq!(step.status, RunStepStatus::Completed);
        match &step.step_details {
            StepDetails::MessageCreation {
                message_creation, ..
            } => {
                assert_eq!(message_creation.message_id, "msg_1");
            }
            _ => panic!("expected message_creation step"),
        }
    }

    #[test]
    fn run_step_deserializes_with_timestamps() {
        let json = r#"{
            "id":"step_1",
            "object":"thread.run.step",
            "thread_id":"thread_1",
            "run_id":"run_1",
            "assistant_id":"asst_1",
            "type":"message_creation",
            "status":"completed",
            "created_at":1700000000,
            "completed_at":1700000100,
            "cancelled_at":null,
            "expired_at":null,
            "failed_at":null,
            "step_details":{
                "type":"message_creation",
                "message_creation":{"message_id":"msg_1"}
            }
        }"#;

        let step: RunStep = serde_json::from_str(json).expect("deserialize run step timestamps");
        assert_eq!(step.created_at, Some(1_700_000_000));
        assert_eq!(step.completed_at, Some(1_700_000_100));
        assert!(step.cancelled_at.is_none());
        assert!(step.expired_at.is_none());
        assert!(step.failed_at.is_none());
    }

    #[test]
    fn run_step_deserializes_tool_calls() {
        let json = r#"{
            "id":"step_2",
            "object":"thread.run.step",
            "thread_id":"thread_1",
            "run_id":"run_1",
            "assistant_id":"asst_1",
            "type":"tool_calls",
            "status":"completed",
            "step_details":{
                "type":"tool_calls",
                "tool_calls":[{
                    "id":"call_1",
                    "type":"code_interpreter",
                    "code_interpreter":{"input":"print('hi')","outputs":[{"type":"logs","logs":"hi"}]}
                }]
            }
        }"#;

        let step: RunStep = serde_json::from_str(json).expect("deserialize run step tool_calls");
        match &step.step_details {
            StepDetails::ToolCalls { tool_calls } => {
                assert_eq!(tool_calls.len(), 1);
                match &tool_calls[0] {
                    StepToolCall::CodeInterpreter {
                        id,
                        code_interpreter,
                    } => {
                        assert_eq!(id, "call_1");
                        assert_eq!(code_interpreter.input, "print('hi')");
                        let outputs = code_interpreter.outputs.as_ref().unwrap();
                        assert_eq!(outputs.len(), 1);
                        match &outputs[0] {
                            CodeInterpreterOutput::Logs { logs } => assert_eq!(logs, "hi"),
                            _ => panic!("expected logs output"),
                        }
                    }
                    _ => panic!("expected code_interpreter call"),
                }
            }
            _ => panic!("expected tool_calls step"),
        }
    }

    #[test]
    fn run_step_deserializes_function_tool_call() {
        let json = r#"{
            "id":"step_3",
            "object":"thread.run.step",
            "thread_id":"thread_1",
            "run_id":"run_1",
            "assistant_id":"asst_1",
            "type":"tool_calls",
            "status":"completed",
            "step_details":{
                "type":"tool_calls",
                "tool_calls":[{
                    "id":"call_fn",
                    "type":"function",
                    "function":{
                        "name":"get_weather",
                        "arguments":"{\"city\":\"NYC\"}",
                        "output":"sunny"
                    }
                }]
            }
        }"#;

        let step: RunStep = serde_json::from_str(json).expect("deserialize function call step");
        match &step.step_details {
            StepDetails::ToolCalls { tool_calls } => match &tool_calls[0] {
                StepToolCall::Function { id, function } => {
                    assert_eq!(id, "call_fn");
                    assert_eq!(function.name, "get_weather");
                    assert_eq!(function.output.as_deref(), Some("sunny"));
                }
                _ => panic!("expected function call"),
            },
            _ => panic!("expected tool_calls"),
        }
    }

    #[test]
    fn assistant_tool_code_interpreter_roundtrips() {
        let tool = AssistantTool::CodeInterpreter;
        let value = serde_json::to_value(&tool).expect("serialize");
        assert_eq!(value["type"], "code_interpreter");

        let deserialized: AssistantTool = serde_json::from_value(value).expect("deserialize");
        assert!(matches!(deserialized, AssistantTool::CodeInterpreter));
    }

    #[test]
    fn assistant_tool_file_search_roundtrips() {
        let tool = AssistantTool::FileSearch { file_search: None };
        let value = serde_json::to_value(&tool).expect("serialize");
        assert_eq!(value["type"], "file_search");
    }

    #[test]
    fn assistant_tool_file_search_with_ranking_options() {
        let tool = AssistantTool::FileSearch {
            file_search: Some(FileSearchConfig {
                max_num_results: Some(20),
                ranking_options: Some(FileSearchRankingOptions {
                    score_threshold: 0.8,
                    ranker: Some("auto".to_owned()),
                }),
            }),
        };
        let value = serde_json::to_value(&tool).expect("serialize");
        assert_eq!(value["type"], "file_search");
        assert_eq!(value["file_search"]["max_num_results"], 20);
        assert_eq!(
            value["file_search"]["ranking_options"]["score_threshold"],
            0.8
        );
    }

    #[test]
    fn assistant_tool_function_roundtrips() {
        let tool = AssistantTool::Function {
            function: FunctionDefinition {
                name: "greet".to_owned(),
                description: Some("Greets user".to_owned()),
                parameters: Some(serde_json::json!({"type":"object","properties":{}})),
            },
        };
        let value = serde_json::to_value(&tool).expect("serialize");
        assert_eq!(value["type"], "function");
        assert_eq!(value["function"]["name"], "greet");

        let deserialized: AssistantTool = serde_json::from_value(value).expect("deserialize");
        match deserialized {
            AssistantTool::Function { function } => {
                assert_eq!(function.name, "greet");
                assert_eq!(function.description.as_deref(), Some("Greets user"));
            }
            _ => panic!("expected function tool"),
        }
    }

    #[test]
    fn tool_resources_serializes_with_file_ids() {
        let resources = ToolResources {
            code_interpreter: Some(CodeInterpreterResources {
                file_ids: Some(vec!["file-1".to_owned(), "file-2".to_owned()]),
            }),
            file_search: Some(FileSearchResources {
                vector_store_ids: Some(vec!["vs-1".to_owned()]),
            }),
        };
        let value = serde_json::to_value(&resources).expect("serialize");
        assert_eq!(
            value["code_interpreter"]["file_ids"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            value["file_search"]["vector_store_ids"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn tool_resources_default_serializes_empty() {
        let value = serde_json::to_value(ToolResources::default()).expect("serialize");
        assert_eq!(value, serde_json::json!({}));
    }

    #[test]
    fn assistant_stream_event_thread_created_deserializes() {
        let json = r#"{"event":"thread.created","data":{"id":"thread_1","object":"thread"}}"#;
        let event: AssistantStreamEvent =
            serde_json::from_str(json).expect("deserialize stream event");
        match event {
            AssistantStreamEvent::ThreadCreated { data } => {
                assert_eq!(data["id"], "thread_1");
            }
            _ => panic!("expected ThreadCreated"),
        }
    }

    #[test]
    fn assistant_stream_event_run_created_deserializes() {
        let json = r#"{"event":"thread.run.created","data":{"id":"run_1","status":"queued"}}"#;
        let event: AssistantStreamEvent =
            serde_json::from_str(json).expect("deserialize stream event");
        match event {
            AssistantStreamEvent::ThreadRunCreated { data } => {
                assert_eq!(data["id"], "run_1");
            }
            _ => panic!("expected ThreadRunCreated"),
        }
    }

    #[test]
    fn assistant_stream_event_message_delta_deserializes() {
        let json = r#"{"event":"thread.message.delta","data":{"id":"msg_1","delta":{"content":[{"type":"text","text":{"value":"Hello"}}]}}}"#;
        let event: AssistantStreamEvent =
            serde_json::from_str(json).expect("deserialize message delta");
        match event {
            AssistantStreamEvent::ThreadMessageDelta { data } => {
                assert_eq!(data.id, "msg_1");
                let content = data.delta.content.as_ref().unwrap();
                assert_eq!(content.len(), 1);
                match &content[0] {
                    MessageContentDelta::Text { text, .. } => {
                        assert_eq!(text.as_ref().unwrap().value.as_deref(), Some("Hello"));
                    }
                    _ => panic!("expected text delta"),
                }
            }
            _ => panic!("expected ThreadMessageDelta"),
        }
    }

    #[test]
    fn assistant_stream_event_error_deserializes() {
        let json =
            r#"{"event":"error","data":{"code":"rate_limit","message":"Too many requests"}}"#;
        let event: AssistantStreamEvent =
            serde_json::from_str(json).expect("deserialize error event");
        match event {
            AssistantStreamEvent::Error { data } => {
                assert_eq!(data["code"], "rate_limit");
            }
            _ => panic!("expected Error"),
        }
    }

    #[test]
    fn create_and_run_params_omits_optional() {
        let params = ThreadCreateAndRunParams {
            assistant_id: "asst_1".to_owned(),
            thread: None,
            model: None,
            instructions: None,
            tools: None,
            stream: None,
            metadata: None,
            temperature: None,
            top_p: None,
            max_completion_tokens: None,
            max_prompt_tokens: None,
            response_format: None,
            tool_choice: None,
            truncation_strategy: None,
        };
        let value = serde_json::to_value(params).expect("serialize");
        assert_eq!(value["assistant_id"], "asst_1");
        assert!(value.get("thread").is_none());
        assert!(value.get("stream").is_none());
        assert!(value.get("temperature").is_none());
        assert!(value.get("max_completion_tokens").is_none());
    }

    #[test]
    fn submit_tool_outputs_serializes() {
        let params = SubmitToolOutputsParams {
            tool_outputs: vec![ToolOutput {
                tool_call_id: "call_1".to_owned(),
                output: "42".to_owned(),
            }],
            stream: None,
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["tool_outputs"][0]["tool_call_id"], "call_1");
        assert_eq!(value["tool_outputs"][0]["output"], "42");
        assert!(value.get("stream").is_none());
    }

    #[test]
    fn submit_tool_outputs_with_stream_serializes() {
        let params = SubmitToolOutputsParams {
            tool_outputs: vec![ToolOutput {
                tool_call_id: "call_1".to_owned(),
                output: "42".to_owned(),
            }],
            stream: Some(true),
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["stream"], true);
    }

    #[test]
    fn run_deserializes_required_action() {
        let json = r#"{
            "id":"run_1",
            "object":"thread.run",
            "thread_id":"thread_1",
            "assistant_id":"asst_1",
            "status":"requires_action",
            "required_action":{
                "type":"submit_tool_outputs",
                "submit_tool_outputs":{
                    "tool_calls":[{
                        "id":"call_1",
                        "type":"function",
                        "function":{
                            "name":"get_weather",
                            "arguments":"{\"city\":\"NYC\"}"
                        }
                    }]
                }
            }
        }"#;

        let run: Run = serde_json::from_str(json).expect("deserialize run with required_action");
        assert_eq!(run.status, RunStatus::RequiresAction);
        let action = run.required_action.unwrap();
        assert_eq!(action.action_type, "submit_tool_outputs");
        let tool_calls = action.submit_tool_outputs.unwrap().tool_calls;
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].id, "call_1");
        assert_eq!(tool_calls[0].function.as_ref().unwrap().name, "get_weather");
    }

    #[test]
    fn message_attachment_tool_roundtrips() {
        let tools = vec![
            MessageAttachmentTool::CodeInterpreter,
            MessageAttachmentTool::FileSearch,
        ];
        let value = serde_json::to_value(&tools).expect("serialize");
        assert_eq!(value[0]["type"], "code_interpreter");
        assert_eq!(value[1]["type"], "file_search");

        let deserialized: Vec<MessageAttachmentTool> =
            serde_json::from_value(value).expect("deserialize");
        assert!(matches!(
            deserialized[0],
            MessageAttachmentTool::CodeInterpreter
        ));
        assert!(matches!(deserialized[1], MessageAttachmentTool::FileSearch));
    }

    #[test]
    fn code_interpreter_output_image_roundtrips() {
        let output = CodeInterpreterOutput::Image {
            image: CodeInterpreterImage {
                file_id: "file-img".to_owned(),
            },
        };
        let value = serde_json::to_value(&output).expect("serialize");
        assert_eq!(value["type"], "image");
        assert_eq!(value["image"]["file_id"], "file-img");
    }

    #[test]
    fn run_usage_deserializes() {
        let json = r#"{"prompt_tokens":100,"completion_tokens":50,"total_tokens":150}"#;
        let usage: RunUsage = serde_json::from_str(json).expect("deserialize");
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn assistant_list_params_default_serializes_empty() {
        let value = serde_json::to_value(BetaAssistantListParams::default())
            .expect("serialize list params");
        assert_eq!(value, serde_json::json!({}));
    }

    #[test]
    fn assistant_list_params_with_values() {
        let params = BetaAssistantListParams {
            after: Some("asst_abc".to_owned()),
            before: None,
            limit: Some(10),
            order: Some(SortOrder::Desc),
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["after"], "asst_abc");
        assert_eq!(value["limit"], 10);
        assert_eq!(value["order"], "desc");
        assert!(value.get("before").is_none());
    }

    #[test]
    fn run_update_params_default_serializes_empty() {
        let value = serde_json::to_value(BetaThreadRunUpdateParams::default())
            .expect("serialize run update params");
        assert_eq!(value, serde_json::json!({}));
    }

    #[test]
    fn message_status_enum_roundtrips() {
        for (s, expected) in [
            ("in_progress", MessageStatus::InProgress),
            ("incomplete", MessageStatus::Incomplete),
            ("completed", MessageStatus::Completed),
        ] {
            let json = format!("\"{}\"", s);
            let status: MessageStatus =
                serde_json::from_str(&json).expect("deserialize message status");
            assert_eq!(status, expected);
        }
    }

    #[test]
    fn run_step_status_enum_roundtrips() {
        for (s, expected) in [
            ("in_progress", RunStepStatus::InProgress),
            ("cancelled", RunStepStatus::Cancelled),
            ("failed", RunStepStatus::Failed),
            ("completed", RunStepStatus::Completed),
            ("expired", RunStepStatus::Expired),
        ] {
            let json = format!("\"{}\"", s);
            let status: RunStepStatus =
                serde_json::from_str(&json).expect("deserialize run step status");
            assert_eq!(status, expected);
        }
    }

    #[test]
    fn run_step_type_enum_roundtrips() {
        for (s, expected) in [
            ("message_creation", RunStepType::MessageCreation),
            ("tool_calls", RunStepType::ToolCalls),
        ] {
            let json = format!("\"{}\"", s);
            let step_type: RunStepType =
                serde_json::from_str(&json).expect("deserialize run step type");
            assert_eq!(step_type, expected);
        }
    }

    #[test]
    fn reasoning_effort_enum_roundtrips() {
        for (s, expected) in [
            ("none", ReasoningEffort::None),
            ("low", ReasoningEffort::Low),
            ("medium", ReasoningEffort::Medium),
            ("high", ReasoningEffort::High),
        ] {
            let json = format!("\"{}\"", s);
            let effort: ReasoningEffort =
                serde_json::from_str(&json).expect("deserialize reasoning effort");
            assert_eq!(effort, expected);
        }
    }

    #[test]
    fn assistant_with_description_and_response_format_deserializes() {
        let json = r#"{
            "id":"asst_1",
            "object":"assistant",
            "model":"gpt-4o",
            "created_at":1700000000,
            "description":"A helpful assistant",
            "response_format":"auto",
            "temperature":0.7,
            "top_p":0.9
        }"#;
        let assistant: Assistant = serde_json::from_str(json).expect("deserialize assistant");
        assert_eq!(
            assistant.description.as_deref(),
            Some("A helpful assistant")
        );
        assert_eq!(assistant.created_at, Some(1_700_000_000));
        assert_eq!(assistant.temperature, Some(0.7));
        assert_eq!(assistant.top_p, Some(0.9));
    }

    #[test]
    fn thread_with_created_at_deserializes() {
        let json = r#"{
            "id":"thread_1",
            "object":"thread",
            "created_at":1700000000
        }"#;
        let thread: Thread = serde_json::from_str(json).expect("deserialize thread");
        assert_eq!(thread.created_at, Some(1_700_000_000));
    }

    #[test]
    fn message_list_params_with_run_id() {
        let params = BetaThreadMessageListParams {
            run_id: Some("run_abc".to_owned()),
            limit: Some(5),
            ..Default::default()
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["run_id"], "run_abc");
        assert_eq!(value["limit"], 5);
    }

    #[test]
    fn run_step_list_params_with_include() {
        let params = BetaThreadRunStepListParams {
            include: Some(vec![
                "step_details.tool_calls[*].file_search.results[*].content".to_owned(),
            ]),
            ..Default::default()
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["include"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn run_with_all_new_fields_deserializes() {
        let json = r#"{
            "id":"run_1",
            "object":"thread.run",
            "thread_id":"thread_1",
            "assistant_id":"asst_1",
            "status":"completed",
            "created_at":1700000000,
            "max_completion_tokens":1000,
            "max_prompt_tokens":500,
            "parallel_tool_calls":true,
            "temperature":0.5,
            "top_p":0.8,
            "truncation_strategy":{"type":"auto"},
            "response_format":"auto",
            "tool_choice":"auto",
            "incomplete_details":{"reason":"max_completion_tokens"}
        }"#;
        let run: Run = serde_json::from_str(json).expect("deserialize run with new fields");
        assert_eq!(run.max_completion_tokens, Some(1000));
        assert_eq!(run.max_prompt_tokens, Some(500));
        assert_eq!(run.parallel_tool_calls, Some(true));
        assert_eq!(run.temperature, Some(0.5));
        assert_eq!(run.top_p, Some(0.8));
        assert!(run.truncation_strategy.is_some());
        assert!(run.response_format.is_some());
        assert!(run.tool_choice.is_some());
        assert!(run.incomplete_details.is_some());
    }

    // -----------------------------------------------------------------------
    // Delta type tests
    // -----------------------------------------------------------------------

    #[test]
    fn message_delta_event_deserializes() {
        let json = r#"{
            "id":"msg_1",
            "object":"thread.message.delta",
            "delta":{
                "role":"assistant",
                "content":[{
                    "type":"text",
                    "index":0,
                    "text":{"value":"Hello world"}
                }]
            }
        }"#;
        let event: MessageDeltaEvent = serde_json::from_str(json).expect("deserialize");
        assert_eq!(event.id, "msg_1");
        assert_eq!(event.delta.role, Some(MessageRole::Assistant));
        let content = event.delta.content.unwrap();
        assert_eq!(content.len(), 1);
        match &content[0] {
            MessageContentDelta::Text { index, text } => {
                assert_eq!(*index, Some(0));
                assert_eq!(text.as_ref().unwrap().value.as_deref(), Some("Hello world"));
            }
            _ => panic!("expected text delta"),
        }
    }

    #[test]
    fn message_delta_with_annotations() {
        let json = r#"{
            "id":"msg_2",
            "delta":{
                "content":[{
                    "type":"text",
                    "index":0,
                    "text":{
                        "value":"See [1]",
                        "annotations":[{
                            "type":"file_citation",
                            "index":0,
                            "text":"[1]",
                            "file_citation":{"file_id":"file-abc"},
                            "start_index":4,
                            "end_index":7
                        }]
                    }
                }]
            }
        }"#;
        let event: MessageDeltaEvent = serde_json::from_str(json).expect("deserialize");
        let content = event.delta.content.unwrap();
        match &content[0] {
            MessageContentDelta::Text { text, .. } => {
                let annotations = text.as_ref().unwrap().annotations.as_ref().unwrap();
                assert_eq!(annotations.len(), 1);
                match &annotations[0] {
                    AnnotationDelta::FileCitation {
                        text,
                        file_citation,
                        start_index,
                        end_index,
                        ..
                    } => {
                        assert_eq!(text.as_deref(), Some("[1]"));
                        assert_eq!(file_citation.as_ref().unwrap().file_id, "file-abc");
                        assert_eq!(*start_index, Some(4));
                        assert_eq!(*end_index, Some(7));
                    }
                    _ => panic!("expected file_citation annotation"),
                }
            }
            _ => panic!("expected text delta"),
        }
    }

    #[test]
    fn message_delta_refusal() {
        let json = r#"{
            "id":"msg_3",
            "delta":{
                "content":[{
                    "type":"refusal",
                    "index":0,
                    "refusal":"I cannot help with that."
                }]
            }
        }"#;
        let event: MessageDeltaEvent = serde_json::from_str(json).expect("deserialize");
        let content = event.delta.content.unwrap();
        match &content[0] {
            MessageContentDelta::Refusal { refusal, .. } => {
                assert_eq!(refusal.as_deref(), Some("I cannot help with that."));
            }
            _ => panic!("expected refusal delta"),
        }
    }

    #[test]
    fn run_step_delta_event_deserializes() {
        let json = r#"{
            "id":"step_1",
            "object":"thread.run.step.delta",
            "delta":{
                "step_details":{
                    "type":"tool_calls",
                    "tool_calls":[{
                        "type":"function",
                        "index":0,
                        "id":"call_1",
                        "function":{"name":"get_weather","arguments":"{\"city\":"}
                    }]
                }
            }
        }"#;
        let event: RunStepDeltaEvent = serde_json::from_str(json).expect("deserialize");
        assert_eq!(event.id, "step_1");
        let details = event.delta.step_details.unwrap();
        match details {
            StepDetailsDelta::ToolCalls { tool_calls } => {
                let calls = tool_calls.unwrap();
                assert_eq!(calls.len(), 1);
                match &calls[0] {
                    ToolCallDelta::Function {
                        index,
                        id,
                        function,
                    } => {
                        assert_eq!(*index, Some(0));
                        assert_eq!(id.as_deref(), Some("call_1"));
                        let f = function.as_ref().unwrap();
                        assert_eq!(f.name.as_deref(), Some("get_weather"));
                        assert_eq!(f.arguments.as_deref(), Some("{\"city\":"));
                    }
                    _ => panic!("expected function tool call delta"),
                }
            }
            _ => panic!("expected tool_calls step details delta"),
        }
    }

    #[test]
    fn run_step_delta_code_interpreter() {
        let json = r#"{
            "id":"step_2",
            "delta":{
                "step_details":{
                    "type":"tool_calls",
                    "tool_calls":[{
                        "type":"code_interpreter",
                        "index":0,
                        "id":"call_ci"
                    }]
                }
            }
        }"#;
        let event: RunStepDeltaEvent = serde_json::from_str(json).expect("deserialize");
        match event.delta.step_details.unwrap() {
            StepDetailsDelta::ToolCalls { tool_calls } => match &tool_calls.unwrap()[0] {
                ToolCallDelta::CodeInterpreter { id, .. } => {
                    assert_eq!(id.as_deref(), Some("call_ci"));
                }
                _ => panic!("expected code_interpreter delta"),
            },
            _ => panic!("expected tool_calls"),
        }
    }

    #[test]
    fn run_step_delta_message_creation() {
        let json = r#"{
            "id":"step_3",
            "delta":{
                "step_details":{
                    "type":"message_creation"
                }
            }
        }"#;
        let event: RunStepDeltaEvent = serde_json::from_str(json).expect("deserialize");
        assert!(matches!(
            event.delta.step_details.unwrap(),
            StepDetailsDelta::MessageCreation { .. }
        ));
    }

    #[test]
    fn text_delta_roundtrips() {
        let delta = TextDelta {
            value: Some("hello".to_owned()),
            annotations: None,
        };
        let value = serde_json::to_value(&delta).expect("serialize");
        assert_eq!(value["value"], "hello");
        assert!(value.get("annotations").is_none());

        let parsed: TextDelta = serde_json::from_value(value).expect("deserialize");
        assert_eq!(parsed.value.as_deref(), Some("hello"));
    }

    #[test]
    fn function_call_delta_roundtrips() {
        let delta = FunctionCallDelta {
            name: Some("get_weather".to_owned()),
            arguments: Some("{\"city\":\"NYC\"}".to_owned()),
            output: None,
        };
        let value = serde_json::to_value(&delta).expect("serialize");
        assert_eq!(value["name"], "get_weather");
        assert!(value.get("output").is_none());
    }

    // -----------------------------------------------------------------------
    // ChatKit type tests
    // -----------------------------------------------------------------------

    #[test]
    fn chat_session_status_roundtrips() {
        for (s, expected) in [
            ("active", ChatSessionStatus::Active),
            ("expired", ChatSessionStatus::Expired),
            ("cancelled", ChatSessionStatus::Cancelled),
        ] {
            let json = format!("\"{}\"", s);
            let status: ChatSessionStatus =
                serde_json::from_str(&json).expect("deserialize session status");
            assert_eq!(status, expected);
        }
    }

    #[test]
    fn chat_session_deserializes() {
        let json = r#"{
            "id":"sess_1",
            "object":"chatkit.session",
            "client_secret":"secret_abc",
            "expires_at":1700001000,
            "max_requests_per_1_minute":10,
            "status":"active",
            "user":"user_1",
            "workflow":{"id":"wf_1"},
            "rate_limits":{"max_requests_per_1_minute":10}
        }"#;
        let session: ChatSession = serde_json::from_str(json).expect("deserialize session");
        assert_eq!(session.id, "sess_1");
        assert_eq!(session.status, Some(ChatSessionStatus::Active));
        assert_eq!(session.client_secret.as_deref(), Some("secret_abc"));
        assert_eq!(session.expires_at, Some(1_700_001_000));
        assert_eq!(session.workflow.as_ref().unwrap().id, "wf_1");
        assert_eq!(
            session
                .rate_limits
                .as_ref()
                .unwrap()
                .max_requests_per_1_minute,
            10
        );
    }

    #[test]
    fn chatkit_session_create_params_serializes() {
        let params = ChatKitSessionCreateParams {
            user: "user_1".to_owned(),
            workflow: ChatSessionWorkflowParam {
                id: "wf_1".to_owned(),
                version: None,
                state_variables: None,
                tracing: None,
            },
            chatkit_configuration: None,
            expires_after: None,
            rate_limits: None,
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["user"], "user_1");
        assert_eq!(value["workflow"]["id"], "wf_1");
        assert!(value.get("chatkit_configuration").is_none());
    }

    #[test]
    fn chatkit_session_create_params_with_all_fields() {
        let params = ChatKitSessionCreateParams {
            user: "user_1".to_owned(),
            workflow: ChatSessionWorkflowParam {
                id: "wf_1".to_owned(),
                version: Some("v2".to_owned()),
                state_variables: Some(HashMap::from([(
                    "key".to_owned(),
                    serde_json::json!("value"),
                )])),
                tracing: Some(ChatSessionWorkflowTracingParam {
                    enabled: Some(true),
                }),
            },
            chatkit_configuration: Some(ChatSessionChatKitConfigurationParam {
                automatic_thread_titling: Some(ChatSessionAutoThreadTitlingParam {
                    enabled: Some(false),
                }),
                file_upload: Some(ChatSessionFileUploadParam {
                    enabled: Some(true),
                    max_file_size: Some(512),
                    max_files: Some(10),
                }),
                history: Some(ChatSessionHistoryParam {
                    enabled: Some(true),
                    recent_threads: Some(5),
                }),
            }),
            expires_after: Some(ChatSessionExpiresAfterParam {
                seconds: 600,
                anchor: "created_at".to_owned(),
            }),
            rate_limits: Some(ChatSessionRateLimitsParam {
                max_requests_per_1_minute: Some(20),
            }),
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["workflow"]["version"], "v2");
        assert_eq!(value["workflow"]["state_variables"]["key"], "value");
        assert_eq!(
            value["chatkit_configuration"]["file_upload"]["enabled"],
            true
        );
        assert_eq!(
            value["chatkit_configuration"]["file_upload"]["max_files"],
            10
        );
        assert_eq!(value["expires_after"]["seconds"], 600);
        assert_eq!(value["rate_limits"]["max_requests_per_1_minute"], 20);
    }

    #[test]
    fn chatkit_workflow_deserializes() {
        let json = r#"{
            "id":"wf_1",
            "version":"v2",
            "state_variables":{"key":"value"},
            "tracing":{"enabled":true}
        }"#;
        let wf: ChatKitWorkflow = serde_json::from_str(json).expect("deserialize workflow");
        assert_eq!(wf.id, "wf_1");
        assert_eq!(wf.version.as_deref(), Some("v2"));
        assert!(wf.tracing.as_ref().unwrap().enabled);
    }

    #[test]
    fn chatkit_thread_deserializes() {
        let json = r#"{
            "id":"thread_ck_1",
            "object":"chatkit.thread",
            "created_at":1700000000,
            "status":{"type":"active"},
            "title":"My Thread",
            "user":"user_1"
        }"#;
        let thread: ChatKitThread = serde_json::from_str(json).expect("deserialize thread");
        assert_eq!(thread.id, "thread_ck_1");
        assert_eq!(thread.title.as_deref(), Some("My Thread"));
        assert!(matches!(
            thread.status.unwrap(),
            ChatKitThreadStatus::Active
        ));
    }

    #[test]
    fn chatkit_thread_status_locked_deserializes() {
        let json = r#"{"type":"locked","reason":"maintenance"}"#;
        let status: ChatKitThreadStatus =
            serde_json::from_str(json).expect("deserialize locked status");
        match status {
            ChatKitThreadStatus::Locked { reason } => {
                assert_eq!(reason.as_deref(), Some("maintenance"));
            }
            _ => panic!("expected locked status"),
        }
    }

    #[test]
    fn chatkit_thread_status_closed_deserializes() {
        let json = r#"{"type":"closed","reason":"completed"}"#;
        let status: ChatKitThreadStatus =
            serde_json::from_str(json).expect("deserialize closed status");
        match status {
            ChatKitThreadStatus::Closed { reason } => {
                assert_eq!(reason.as_deref(), Some("completed"));
            }
            _ => panic!("expected closed status"),
        }
    }

    #[test]
    fn chatkit_thread_deleted_deserializes() {
        let json = r#"{"id":"thread_ck_1","object":"chatkit.thread.deleted","deleted":true}"#;
        let deleted: ChatKitThreadDeleted =
            serde_json::from_str(json).expect("deserialize deleted");
        assert!(deleted.deleted);
        assert_eq!(deleted.id, "thread_ck_1");
    }

    #[test]
    fn chatkit_thread_list_params_default_is_empty() {
        let value = serde_json::to_value(ChatKitThreadListParams::default())
            .expect("serialize list params");
        assert_eq!(value, serde_json::json!({}));
    }

    #[test]
    fn chatkit_thread_list_params_with_filters() {
        let params = ChatKitThreadListParams {
            user: Some("user_1".to_owned()),
            limit: Some(10),
            order: Some(SortOrder::Desc),
            ..Default::default()
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["user"], "user_1");
        assert_eq!(value["limit"], 10);
        assert_eq!(value["order"], "desc");
    }

    #[test]
    fn chatkit_thread_list_items_params_default_is_empty() {
        let value =
            serde_json::to_value(ChatKitThreadListItemsParams::default()).expect("serialize");
        assert_eq!(value, serde_json::json!({}));
    }

    #[test]
    fn chat_session_chatkit_configuration_deserializes() {
        let json = r#"{
            "automatic_thread_titling":{"enabled":true},
            "file_upload":{"enabled":false,"max_file_size":512,"max_files":10},
            "history":{"enabled":true,"recent_threads":5}
        }"#;
        let config: ChatSessionChatKitConfiguration =
            serde_json::from_str(json).expect("deserialize config");
        assert!(config.automatic_thread_titling.unwrap().enabled);
        assert!(!config.file_upload.as_ref().unwrap().enabled);
        assert_eq!(
            config.file_upload.as_ref().unwrap().max_file_size,
            Some(512)
        );
        assert!(config.history.as_ref().unwrap().enabled);
        assert_eq!(config.history.unwrap().recent_threads, Some(5));
    }

    #[test]
    fn chatkit_configuration_param_default_is_empty() {
        let value = serde_json::to_value(ChatSessionChatKitConfigurationParam::default())
            .expect("serialize");
        assert_eq!(value, serde_json::json!({}));
    }

    #[test]
    fn expires_after_param_serializes() {
        let param = ChatSessionExpiresAfterParam {
            seconds: 600,
            anchor: "created_at".to_owned(),
        };
        let value = serde_json::to_value(&param).expect("serialize");
        assert_eq!(value["seconds"], 600);
        assert_eq!(value["anchor"], "created_at");
    }

    #[test]
    fn stream_event_run_step_delta_typed() {
        let json = r#"{
            "event":"thread.run.step.delta",
            "data":{
                "id":"step_d1",
                "delta":{
                    "step_details":{
                        "type":"tool_calls",
                        "tool_calls":[{
                            "type":"function",
                            "index":0,
                            "function":{"arguments":"partial"}
                        }]
                    }
                }
            }
        }"#;
        let event: AssistantStreamEvent =
            serde_json::from_str(json).expect("deserialize step delta event");
        match event {
            AssistantStreamEvent::ThreadRunStepDelta { data } => {
                assert_eq!(data.id, "step_d1");
                match data.delta.step_details.unwrap() {
                    StepDetailsDelta::ToolCalls { tool_calls } => {
                        let calls = tool_calls.unwrap();
                        match &calls[0] {
                            ToolCallDelta::Function { function, .. } => {
                                assert_eq!(
                                    function.as_ref().unwrap().arguments.as_deref(),
                                    Some("partial")
                                );
                            }
                            _ => panic!("expected function delta"),
                        }
                    }
                    _ => panic!("expected tool_calls"),
                }
            }
            _ => panic!("expected ThreadRunStepDelta"),
        }
    }

    #[test]
    fn annotation_delta_file_path_roundtrips() {
        let json = r#"{
            "type":"file_path",
            "index":0,
            "text":"sandbox:/file.csv",
            "file_path":{"file_id":"file-xyz"},
            "start_index":0,
            "end_index":18
        }"#;
        let ann: AnnotationDelta = serde_json::from_str(json).expect("deserialize");
        match ann {
            AnnotationDelta::FilePath {
                text,
                file_path,
                start_index,
                end_index,
                ..
            } => {
                assert_eq!(text.as_deref(), Some("sandbox:/file.csv"));
                assert_eq!(file_path.as_ref().unwrap().file_id, "file-xyz");
                assert_eq!(start_index, Some(0));
                assert_eq!(end_index, Some(18));
            }
            _ => panic!("expected file_path annotation"),
        }
    }

    #[test]
    fn tool_call_delta_file_search() {
        let json = r#"{
            "type":"file_search",
            "index":0,
            "id":"call_fs"
        }"#;
        let delta: ToolCallDelta = serde_json::from_str(json).expect("deserialize");
        match delta {
            ToolCallDelta::FileSearch { id, index, .. } => {
                assert_eq!(id.as_deref(), Some("call_fs"));
                assert_eq!(index, Some(0));
            }
            _ => panic!("expected file_search delta"),
        }
    }
}
