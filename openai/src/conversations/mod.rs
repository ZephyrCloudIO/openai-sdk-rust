//! Conversations and conversation items APIs.

use std::collections::HashMap;

use crate::{
    pagination::{ConversationCursorPage, CursorPage},
    Client, Result,
};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// The role of a message author.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    /// Unknown role.
    Unknown,
    /// User message.
    User,
    /// Assistant message.
    Assistant,
    /// System message.
    System,
    /// Critic message.
    Critic,
    /// Discriminator message.
    Discriminator,
    /// Developer message.
    Developer,
    /// Tool output message.
    Tool,
}

/// The status of a conversation item.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageStatus {
    /// Item is being processed.
    InProgress,
    /// Item completed successfully.
    Completed,
    /// Item was interrupted or left incomplete.
    Incomplete,
}

// ---------------------------------------------------------------------------
// Message content types
// ---------------------------------------------------------------------------

/// Output text annotation (e.g. file citation or URL citation).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutputTextAnnotation {
    /// Annotation type.
    #[serde(rename = "type")]
    pub annotation_type: String,
    /// Start index in the text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<i64>,
    /// End index in the text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i64>,
    /// URL for URL citations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Title for URL citations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// File ID for file citations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}

/// Content block within a conversation message.
///
/// Discriminated by the `type` field. Covers input text, output text, refusal,
/// input image, input file, and other content types produced by the model.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ConversationMessageContent {
    /// Text supplied by the user.
    #[serde(rename = "input_text")]
    InputText {
        /// The input text.
        text: String,
    },
    /// Text produced by the model.
    #[serde(rename = "output_text")]
    OutputText {
        /// The output text.
        text: String,
        /// Optional annotations (citations, etc.).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        annotations: Vec<OutputTextAnnotation>,
    },
    /// Plain text content.
    #[serde(rename = "text")]
    Text {
        /// The text.
        text: String,
    },
    /// Summary text from reasoning.
    #[serde(rename = "summary_text")]
    SummaryText {
        /// The summary text.
        text: String,
    },
    /// Reasoning text from the model.
    #[serde(rename = "reasoning_text")]
    ReasoningText {
        /// The reasoning text.
        text: String,
    },
    /// Model refusal to answer.
    #[serde(rename = "refusal")]
    Refusal {
        /// The refusal text.
        refusal: String,
    },
    /// Image input.
    #[serde(rename = "input_image")]
    InputImage {
        /// Image detail level (low, high, auto).
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
        /// File ID of an uploaded image.
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
        /// URL of the image.
        #[serde(skip_serializing_if = "Option::is_none")]
        image_url: Option<String>,
    },
    /// A screenshot of a computer session.
    #[serde(rename = "computer_screenshot")]
    ComputerScreenshot {
        /// Image detail level.
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
        /// File ID of the screenshot.
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
        /// URL of the screenshot.
        #[serde(skip_serializing_if = "Option::is_none")]
        image_url: Option<String>,
    },
    /// File input.
    #[serde(rename = "input_file")]
    InputFile {
        /// Base64-encoded file data.
        #[serde(skip_serializing_if = "Option::is_none")]
        file_data: Option<String>,
        /// URL of the file.
        #[serde(skip_serializing_if = "Option::is_none")]
        file_url: Option<String>,
        /// Filename.
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
    },
}

// ---------------------------------------------------------------------------
// Reasoning item summary
// ---------------------------------------------------------------------------

/// A summary entry within a reasoning item.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReasoningItemSummary {
    /// Summary text.
    pub text: String,
    /// Summary type.
    #[serde(rename = "type")]
    pub summary_type: String,
}

// ---------------------------------------------------------------------------
// Conversation item union
// ---------------------------------------------------------------------------

/// A conversation item, discriminated by the `type` field.
///
/// Covers messages, function calls, tool outputs, search calls, computer
/// calls, reasoning items, code interpreter calls, and more.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ConversationItemUnion {
    /// A message to or from the model.
    #[serde(rename = "message")]
    Message {
        /// Unique item ID.
        id: String,
        /// Message role.
        role: MessageRole,
        /// Message content blocks.
        content: Vec<ConversationMessageContent>,
        /// Item status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<MessageStatus>,
    },
    /// A function tool call from the model.
    #[serde(rename = "function_call")]
    FunctionCall {
        /// Unique item ID.
        id: String,
        /// Function name.
        name: String,
        /// JSON-encoded arguments.
        arguments: String,
        /// The call identifier for pairing with output.
        call_id: String,
        /// Item status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<MessageStatus>,
    },
    /// Output returned for a function tool call.
    #[serde(rename = "function_call_output")]
    FunctionCallOutput {
        /// Unique item ID.
        id: String,
        /// The call identifier this output corresponds to.
        call_id: String,
        /// The output value.
        output: serde_json::Value,
    },
    /// A file search tool call.
    #[serde(rename = "file_search_call")]
    FileSearchCall {
        /// Unique item ID.
        id: String,
        /// Queries used in the search.
        #[serde(default)]
        queries: Vec<String>,
        /// Search results.
        #[serde(default)]
        results: Vec<serde_json::Value>,
        /// Item status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<MessageStatus>,
    },
    /// A web search tool call.
    #[serde(rename = "web_search_call")]
    WebSearchCall {
        /// Unique item ID.
        id: String,
        /// Item status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<MessageStatus>,
    },
    /// An image generation tool call.
    #[serde(rename = "image_generation_call")]
    ImageGenerationCall {
        /// Unique item ID.
        id: String,
        /// Generation result.
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<String>,
        /// Item status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<MessageStatus>,
    },
    /// A computer tool call.
    #[serde(rename = "computer_call")]
    ComputerCall {
        /// Unique item ID.
        id: String,
        /// The action performed.
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<serde_json::Value>,
        /// Item status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<MessageStatus>,
    },
    /// Output from a computer tool call.
    #[serde(rename = "computer_call_output")]
    ComputerCallOutput {
        /// Unique item ID.
        id: String,
        /// Output value.
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<serde_json::Value>,
    },
    /// A tool search call.
    #[serde(rename = "tool_search_call")]
    ToolSearchCall {
        /// Unique item ID.
        id: String,
        /// Item status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<MessageStatus>,
    },
    /// Output from a tool search call.
    #[serde(rename = "tool_search_output")]
    ToolSearchOutput {
        /// Unique item ID.
        id: String,
    },
    /// A reasoning item produced by the model.
    #[serde(rename = "reasoning")]
    Reasoning {
        /// Unique item ID.
        id: String,
        /// Reasoning summary entries.
        #[serde(default)]
        summary: Vec<ReasoningItemSummary>,
    },
    /// A compaction item produced during context management.
    #[serde(rename = "compaction")]
    Compaction {
        /// Unique item ID.
        id: String,
    },
    /// A code interpreter tool call.
    #[serde(rename = "code_interpreter_call")]
    CodeInterpreterCall {
        /// Unique item ID.
        id: String,
        /// Code executed.
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
        /// Container ID used.
        #[serde(skip_serializing_if = "Option::is_none")]
        container_id: Option<String>,
        /// Item status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<MessageStatus>,
    },
    /// A local shell tool call.
    #[serde(rename = "local_shell_call")]
    LocalShellCall {
        /// Unique item ID.
        id: String,
        /// The action.
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<serde_json::Value>,
        /// Item status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<MessageStatus>,
    },
    /// Output from a local shell tool call.
    #[serde(rename = "local_shell_call_output")]
    LocalShellCallOutput {
        /// Unique item ID.
        id: String,
        /// Output value.
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<serde_json::Value>,
    },
    /// A shell tool call.
    #[serde(rename = "shell_call")]
    ShellCall {
        /// Unique item ID.
        id: String,
        /// Item status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<MessageStatus>,
    },
    /// Output from a shell tool call.
    #[serde(rename = "shell_call_output")]
    ShellCallOutput {
        /// Unique item ID.
        id: String,
        /// Output value.
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<serde_json::Value>,
    },
    /// An apply-patch tool call.
    #[serde(rename = "apply_patch_call")]
    ApplyPatchCall {
        /// Unique item ID.
        id: String,
        /// Item status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<MessageStatus>,
    },
    /// Output from an apply-patch tool call.
    #[serde(rename = "apply_patch_call_output")]
    ApplyPatchCallOutput {
        /// Unique item ID.
        id: String,
        /// Output value.
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<serde_json::Value>,
    },
    /// MCP list tools result.
    #[serde(rename = "mcp_list_tools")]
    McpListTools {
        /// Unique item ID.
        id: String,
    },
    /// MCP approval request.
    #[serde(rename = "mcp_approval_request")]
    McpApprovalRequest {
        /// Unique item ID.
        id: String,
    },
    /// MCP approval response.
    #[serde(rename = "mcp_approval_response")]
    McpApprovalResponse {
        /// Unique item ID.
        id: String,
        /// Whether the call was approved.
        #[serde(skip_serializing_if = "Option::is_none")]
        approve: Option<bool>,
        /// Reason for approval or rejection.
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    /// MCP tool call.
    #[serde(rename = "mcp_call")]
    McpCall {
        /// Unique item ID.
        id: String,
        /// Item status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<MessageStatus>,
    },
    /// A custom tool call.
    #[serde(rename = "custom_tool_call")]
    CustomToolCall {
        /// Unique item ID.
        id: String,
        /// Item status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<MessageStatus>,
    },
    /// Output from a custom tool call.
    #[serde(rename = "custom_tool_call_output")]
    CustomToolCallOutput {
        /// Unique item ID.
        id: String,
        /// Output value.
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<serde_json::Value>,
    },
}

impl ConversationItemUnion {
    /// Returns the union value as JSON for dynamic matching.
    #[must_use]
    pub fn as_any(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Returns true when this is a conversation message.
    #[must_use]
    pub fn is_message(&self) -> bool {
        matches!(self, Self::Message { .. })
    }

    /// Returns true when this is a function call.
    #[must_use]
    pub fn is_function_call(&self) -> bool {
        matches!(self, Self::FunctionCall { .. })
    }

    /// Returns true when this is a function call output.
    #[must_use]
    pub fn is_function_call_output(&self) -> bool {
        matches!(self, Self::FunctionCallOutput { .. })
    }

    /// Returns true when this is a reasoning item.
    #[must_use]
    pub fn is_reasoning(&self) -> bool {
        matches!(self, Self::Reasoning { .. })
    }

    /// Returns true when this is a tool call or tool output item.
    #[must_use]
    pub fn is_tool_item(&self) -> bool {
        !matches!(self, Self::Message { .. } | Self::Reasoning { .. })
    }
}

// ---------------------------------------------------------------------------
// Services
// ---------------------------------------------------------------------------

/// Conversation service.
#[derive(Clone)]
pub struct ConversationService {
    client: Client,
}

impl ConversationService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates a conversation.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: ConversationCreateParams) -> Result<Conversation> {
        self.client.post_json("/conversations", &params).await
    }

    /// Gets a conversation by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, conversation_id: impl AsRef<str>) -> Result<Conversation> {
        self.client
            .get_json(&format!(
                "/conversations/{}",
                urlencoding::encode(conversation_id.as_ref())
            ))
            .await
    }

    /// Updates a conversation.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn update(
        &self,
        conversation_id: impl AsRef<str>,
        params: ConversationUpdateParams,
    ) -> Result<Conversation> {
        self.client
            .post_json(
                &format!(
                    "/conversations/{}",
                    urlencoding::encode(conversation_id.as_ref())
                ),
                &params,
            )
            .await
    }

    /// Lists conversations.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(&self) -> Result<CursorPage<Conversation>> {
        self.client.get_cursor_page("/conversations").await
    }

    /// Deletes a conversation.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(&self, conversation_id: impl AsRef<str>) -> Result<ConversationDeleted> {
        self.client
            .delete_json(&format!(
                "/conversations/{}",
                urlencoding::encode(conversation_id.as_ref())
            ))
            .await
    }

    /// Returns the conversation items sub-service.
    #[must_use]
    pub fn items(&self) -> ConversationItemService {
        ConversationItemService::new(self.client.clone())
    }
}

/// Conversation items service (nested under conversations).
#[derive(Clone)]
pub struct ConversationItemService {
    client: Client,
}

impl ConversationItemService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates items in a conversation.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(
        &self,
        conversation_id: impl AsRef<str>,
        params: ConversationItemCreateParams,
    ) -> Result<ConversationItemList> {
        self.client
            .post_json(
                &format!(
                    "/conversations/{}/items",
                    urlencoding::encode(conversation_id.as_ref())
                ),
                &params,
            )
            .await
    }

    /// Gets a single item from a conversation.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(
        &self,
        conversation_id: impl AsRef<str>,
        item_id: impl AsRef<str>,
        params: Option<ConversationItemGetParams>,
    ) -> Result<ConversationItemUnion> {
        let mut path = format!(
            "/conversations/{}/items/{}",
            urlencoding::encode(conversation_id.as_ref()),
            urlencoding::encode(item_id.as_ref())
        );
        if let Some(p) = params {
            let query = p.to_query_string();
            if !query.is_empty() {
                path.push('?');
                path.push_str(&query);
            }
        }
        self.client.get_json(&path).await
    }

    /// Lists items in a conversation.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(
        &self,
        conversation_id: impl AsRef<str>,
        params: ConversationItemListParams,
    ) -> Result<ConversationCursorPage<ConversationItemUnion>> {
        let path = format!(
            "/conversations/{}/items",
            urlencoding::encode(conversation_id.as_ref())
        );
        self.client
            .get_conversation_cursor_page_query(&path, &params)
            .await
    }

    /// Deletes an item from a conversation.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(
        &self,
        conversation_id: impl AsRef<str>,
        item_id: impl AsRef<str>,
    ) -> Result<ConversationItemDeleted> {
        self.client
            .delete_json(&format!(
                "/conversations/{}/items/{}",
                urlencoding::encode(conversation_id.as_ref()),
                urlencoding::encode(item_id.as_ref())
            ))
            .await
    }
}

// ---------------------------------------------------------------------------
// Request params
// ---------------------------------------------------------------------------

/// Conversation create request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationCreateParams {
    /// Optional initial items to include in the conversation (up to 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<ConversationInputItem>>,
    /// Optional metadata key-value pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Conversation update request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationUpdateParams {
    /// Metadata key-value pairs.
    pub metadata: HashMap<String, String>,
}

/// Conversation item create request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationItemCreateParams {
    /// Items to add. Up to 20 at a time.
    pub items: Vec<ConversationInputItem>,
}

/// Input item for creating conversation items.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationInputItem {
    /// Item type (e.g. "message").
    #[serde(rename = "type")]
    pub item_type: String,
    /// Role of the message sender.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Content of the item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ConversationInputContent>>,
}

/// Content block for an input item.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationInputContent {
    /// Content type (e.g. "input_text").
    #[serde(rename = "type")]
    pub content_type: String,
    /// Text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Parameters for getting a single conversation item.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ConversationItemGetParams {
    /// Additional data to include in the response (e.g. `file_search_call.results`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}

impl ConversationItemGetParams {
    /// Serializes non-None fields into a URL query string.
    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref include) = self.include {
            for item in include {
                parts.push(format!("include[]={}", urlencoding::encode(item)));
            }
        }
        parts.join("&")
    }
}

/// Parameters for listing conversation items.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ConversationItemListParams {
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Maximum number of items to return (1-100, default 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Sort order by created_at: "asc" or "desc".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    /// Additional data to include in the response (e.g. `file_search_call.results`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}

impl ConversationItemListParams {
    /// Serializes non-None fields into a URL query string.
    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref after) = self.after {
            parts.push(format!("after={}", urlencoding::encode(after)));
        }
        if let Some(limit) = self.limit {
            parts.push(format!("limit={limit}"));
        }
        if let Some(ref order) = self.order {
            parts.push(format!("order={}", urlencoding::encode(order)));
        }
        if let Some(ref include) = self.include {
            for item in include {
                parts.push(format!("include[]={}", urlencoding::encode(item)));
            }
        }
        parts.join("&")
    }
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Conversation object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Conversation {
    /// Unique conversation ID.
    pub id: String,
    /// Object type (always "conversation").
    pub object: String,
    /// Unix timestamp when created.
    pub created_at: u64,
    /// Metadata key-value pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Conversation delete response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationDeleted {
    /// Deleted conversation ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Deletion flag.
    pub deleted: bool,
}

/// A flat conversation item (message, tool call, etc.) for backward
/// compatibility with non-union deserialization.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationItem {
    /// Unique item ID.
    pub id: String,
    /// Item type (e.g. "message", "function_call", etc.).
    #[serde(rename = "type")]
    pub item_type: String,
    /// Item status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Role for message items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Content blocks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<MessageContent>>,
}

/// Message content block (flat, untagged variant for backward compatibility).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageContent {
    /// Content type (e.g. "input_text", "output_text", "text").
    #[serde(rename = "type")]
    pub content_type: String,
    /// Text value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// A list of conversation items returned from create.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationItemList {
    /// Object type.
    pub object: String,
    /// List of items.
    pub data: Vec<ConversationItem>,
    /// Whether more items are available.
    pub has_more: bool,
    /// ID of first item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    /// ID of last item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
}

/// Conversation item delete response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationItemDeleted {
    /// Deleted item ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Deletion flag.
    pub deleted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversation_create_params_omit_optional_metadata() {
        let params = ConversationCreateParams {
            items: None,
            metadata: None,
        };
        let value = serde_json::to_value(params).expect("serialize conversation create params");
        assert!(value.get("metadata").is_none());
        assert!(value.get("items").is_none());
    }

    #[test]
    fn conversation_create_params_include_metadata() {
        let mut meta = HashMap::new();
        meta.insert("foo".to_owned(), "bar".to_owned());
        let params = ConversationCreateParams {
            items: None,
            metadata: Some(meta),
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["metadata"]["foo"], "bar");
    }

    #[test]
    fn conversation_create_params_with_items() {
        let params = ConversationCreateParams {
            items: Some(vec![ConversationInputItem {
                item_type: "message".to_owned(),
                role: Some("user".to_owned()),
                content: Some(vec![ConversationInputContent {
                    content_type: "input_text".to_owned(),
                    text: Some("Hello".to_owned()),
                }]),
            }]),
            metadata: None,
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["items"][0]["type"], "message");
    }

    #[test]
    fn conversation_deserializes() {
        let json = r#"{
            "id":"conv_1",
            "object":"conversation",
            "created_at":1234567890,
            "metadata":{"key":"value"}
        }"#;

        let conv: Conversation = serde_json::from_str(json).expect("deserialize conversation");
        assert_eq!(conv.id, "conv_1");
        assert_eq!(conv.object, "conversation");
        assert_eq!(conv.created_at, 1_234_567_890);
    }

    #[test]
    fn conversation_deleted_deserializes() {
        let json = r#"{
            "id":"conv_1",
            "object":"conversation.deleted",
            "deleted":true
        }"#;

        let deleted: ConversationDeleted =
            serde_json::from_str(json).expect("deserialize conversation deleted");
        assert_eq!(deleted.id, "conv_1");
        assert!(deleted.deleted);
    }

    #[test]
    fn conversation_item_union_message_round_trip() {
        let json = r#"{
            "type":"message",
            "id":"msg_1",
            "role":"user",
            "content":[{"type":"input_text","text":"Hello"}],
            "status":"completed"
        }"#;
        let item: ConversationItemUnion =
            serde_json::from_str(json).expect("deserialize item union message");
        match &item {
            ConversationItemUnion::Message {
                id, role, status, ..
            } => {
                assert_eq!(id, "msg_1");
                assert_eq!(*role, MessageRole::User);
                assert_eq!(*status, Some(MessageStatus::Completed));
            }
            other => panic!("expected Message, got {other:?}"),
        }
    }

    #[test]
    fn conversation_item_union_function_call_round_trip() {
        let json = r#"{
            "type":"function_call",
            "id":"fc_1",
            "name":"get_weather",
            "arguments":"{\"location\":\"London\"}",
            "call_id":"call_1",
            "status":"completed"
        }"#;
        let item: ConversationItemUnion =
            serde_json::from_str(json).expect("deserialize function_call");
        match &item {
            ConversationItemUnion::FunctionCall {
                id, name, call_id, ..
            } => {
                assert_eq!(id, "fc_1");
                assert_eq!(name, "get_weather");
                assert_eq!(call_id, "call_1");
            }
            other => panic!("expected FunctionCall, got {other:?}"),
        }
    }

    #[test]
    fn conversation_item_union_reasoning_round_trip() {
        let json = r#"{
            "type":"reasoning",
            "id":"reason_1",
            "summary":[{"text":"I think therefore I am.","type":"summary_text"}]
        }"#;
        let item: ConversationItemUnion =
            serde_json::from_str(json).expect("deserialize reasoning");
        match &item {
            ConversationItemUnion::Reasoning { id, summary } => {
                assert_eq!(id, "reason_1");
                assert_eq!(summary.len(), 1);
                assert_eq!(summary[0].text, "I think therefore I am.");
            }
            other => panic!("expected Reasoning, got {other:?}"),
        }
    }

    #[test]
    fn conversation_item_union_code_interpreter_round_trip() {
        let json = r#"{
            "type":"code_interpreter_call",
            "id":"ci_1",
            "code":"print('hello')",
            "status":"completed"
        }"#;
        let item: ConversationItemUnion =
            serde_json::from_str(json).expect("deserialize code_interpreter_call");
        match &item {
            ConversationItemUnion::CodeInterpreterCall { id, code, .. } => {
                assert_eq!(id, "ci_1");
                assert_eq!(code.as_deref(), Some("print('hello')"));
            }
            other => panic!("expected CodeInterpreterCall, got {other:?}"),
        }
    }

    #[test]
    fn message_content_input_text_round_trip() {
        let json = r#"{"type":"input_text","text":"Hello world"}"#;
        let content: ConversationMessageContent =
            serde_json::from_str(json).expect("deserialize input_text");
        match &content {
            ConversationMessageContent::InputText { text } => {
                assert_eq!(text, "Hello world");
            }
            other => panic!("expected InputText, got {other:?}"),
        }
    }

    #[test]
    fn message_content_output_text_with_annotations() {
        let json = r#"{"type":"output_text","text":"See [1].","annotations":[{"type":"url_citation","url":"https://example.com","start_index":4,"end_index":7}]}"#;
        let content: ConversationMessageContent =
            serde_json::from_str(json).expect("deserialize output_text");
        match &content {
            ConversationMessageContent::OutputText { text, annotations } => {
                assert_eq!(text, "See [1].");
                assert_eq!(annotations.len(), 1);
                assert_eq!(annotations[0].url.as_deref(), Some("https://example.com"));
            }
            other => panic!("expected OutputText, got {other:?}"),
        }
    }

    #[test]
    fn message_content_refusal_round_trip() {
        let json = r#"{"type":"refusal","refusal":"I cannot help with that."}"#;
        let content: ConversationMessageContent =
            serde_json::from_str(json).expect("deserialize refusal");
        match &content {
            ConversationMessageContent::Refusal { refusal } => {
                assert_eq!(refusal, "I cannot help with that.");
            }
            other => panic!("expected Refusal, got {other:?}"),
        }
    }

    #[test]
    fn message_content_input_image_round_trip() {
        let json = r#"{"type":"input_image","detail":"high","file_id":"file_abc"}"#;
        let content: ConversationMessageContent =
            serde_json::from_str(json).expect("deserialize input_image");
        match &content {
            ConversationMessageContent::InputImage {
                detail, file_id, ..
            } => {
                assert_eq!(detail.as_deref(), Some("high"));
                assert_eq!(file_id.as_deref(), Some("file_abc"));
            }
            other => panic!("expected InputImage, got {other:?}"),
        }
    }

    #[test]
    fn message_content_input_file_round_trip() {
        let json = r#"{"type":"input_file","filename":"data.csv","file_url":"https://example.com/data.csv"}"#;
        let content: ConversationMessageContent =
            serde_json::from_str(json).expect("deserialize input_file");
        match &content {
            ConversationMessageContent::InputFile {
                filename, file_url, ..
            } => {
                assert_eq!(filename.as_deref(), Some("data.csv"));
                assert_eq!(file_url.as_deref(), Some("https://example.com/data.csv"));
            }
            other => panic!("expected InputFile, got {other:?}"),
        }
    }

    #[test]
    fn message_role_round_trip() {
        let json = r#""developer""#;
        let role: MessageRole = serde_json::from_str(json).expect("deserialize role");
        assert_eq!(role, MessageRole::Developer);
        let serialized = serde_json::to_string(&role).expect("serialize role");
        assert_eq!(serialized, r#""developer""#);
    }

    #[test]
    fn message_status_round_trip() {
        let json = r#""in_progress""#;
        let status: MessageStatus = serde_json::from_str(json).expect("deserialize status");
        assert_eq!(status, MessageStatus::InProgress);
    }

    #[test]
    fn conversation_item_deserializes_message() {
        let json = r#"{
            "id":"msg_1",
            "type":"message",
            "status":"completed",
            "role":"user",
            "content":[{"type":"input_text","text":"Hello"}]
        }"#;

        let item: ConversationItem =
            serde_json::from_str(json).expect("deserialize conversation item");
        assert_eq!(item.id, "msg_1");
        assert_eq!(item.item_type, "message");
        assert_eq!(item.role.as_deref(), Some("user"));
        let content = item.content.as_ref().unwrap();
        assert_eq!(content.len(), 1);
        assert_eq!(content[0].text.as_deref(), Some("Hello"));
    }

    #[test]
    fn conversation_item_list_deserializes() {
        let json = r#"{
            "object":"list",
            "data":[{
                "id":"msg_1",
                "type":"message",
                "status":"completed",
                "role":"assistant",
                "content":[{"type":"output_text","text":"Hi there"}]
            }],
            "has_more":false,
            "first_id":"msg_1",
            "last_id":"msg_1"
        }"#;

        let list: ConversationItemList = serde_json::from_str(json).expect("deserialize item list");
        assert_eq!(list.data.len(), 1);
        assert!(!list.has_more);
    }

    #[test]
    fn conversation_item_list_params_query_string() {
        let params = ConversationItemListParams {
            after: Some("msg_abc".to_owned()),
            limit: Some(10),
            order: Some("asc".to_owned()),
            include: None,
        };
        let qs = params.to_query_string();
        assert!(qs.contains("after=msg_abc"));
        assert!(qs.contains("limit=10"));
        assert!(qs.contains("order=asc"));
    }

    #[test]
    fn conversation_item_list_params_with_include() {
        let params = ConversationItemListParams {
            after: None,
            limit: None,
            order: None,
            include: Some(vec!["file_search_call.results".to_owned()]),
        };
        let qs = params.to_query_string();
        assert!(qs.contains("include[]=file_search_call.results"));
    }

    #[test]
    fn conversation_item_list_params_empty_query_string() {
        let params = ConversationItemListParams::default();
        assert!(params.to_query_string().is_empty());
    }

    #[test]
    fn conversation_item_get_params_include() {
        let params = ConversationItemGetParams {
            include: Some(vec!["file_search_call.results".to_owned()]),
        };
        let qs = params.to_query_string();
        assert!(qs.contains("include[]=file_search_call.results"));
    }

    #[test]
    fn conversation_input_item_serializes() {
        let item = ConversationInputItem {
            item_type: "message".to_owned(),
            role: Some("user".to_owned()),
            content: Some(vec![ConversationInputContent {
                content_type: "input_text".to_owned(),
                text: Some("Hello".to_owned()),
            }]),
        };
        let value = serde_json::to_value(&item).expect("serialize input item");
        assert_eq!(value["type"], "message");
        assert_eq!(value["role"], "user");
    }

    #[test]
    fn conversation_item_deleted_deserializes() {
        let json = r#"{
            "id":"msg_1",
            "object":"conversation.item.deleted",
            "deleted":true
        }"#;
        let deleted: ConversationItemDeleted =
            serde_json::from_str(json).expect("deserialize item deleted");
        assert_eq!(deleted.id, "msg_1");
        assert!(deleted.deleted);
    }

    #[test]
    fn conversation_update_params_serializes_metadata() {
        let mut meta = HashMap::new();
        meta.insert("key".to_owned(), "value".to_owned());
        let params = ConversationUpdateParams { metadata: meta };
        let value = serde_json::to_value(&params).expect("serialize update params");
        assert_eq!(value["metadata"]["key"], "value");
    }
}
