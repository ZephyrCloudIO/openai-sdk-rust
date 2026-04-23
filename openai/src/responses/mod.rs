//! Responses API - the standard inference API for generating model responses.
//!
//! The Responses API replaces `/chat/completions` as the primary inference
//! endpoint. It supports text and image inputs, structured outputs, function
//! calling, built-in tools (web search, file search, code interpreter), MCP
//! tools, streaming, background execution, and conversation state management.

use std::collections::HashMap;

use futures::Stream;

use crate::{pagination::CursorPage, shared::ModelId, ssestream::SseStream, Client, Result};

// ---------------------------------------------------------------------------
// Services
// ---------------------------------------------------------------------------

/// Responses service root.
#[derive(Clone)]
pub struct ResponseService {
    client: Client,
}

impl ResponseService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Returns the input items sub-service for listing input items.
    #[must_use]
    pub fn input_items(&self) -> InputItemService {
        InputItemService {
            client: self.client.clone(),
        }
    }

    /// Returns the input token counting sub-service.
    #[must_use]
    pub fn input_tokens(&self) -> InputTokenService {
        InputTokenService {
            client: self.client.clone(),
        }
    }

    /// Creates a model response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: ResponseCreateParams) -> Result<Response> {
        self.client.post_json("/responses", &params).await
    }

    /// Creates a streaming model response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or stream decode failures.
    pub async fn create_stream(
        &self,
        mut params: ResponseCreateParams,
    ) -> Result<impl Stream<Item = Result<ResponseStreamEvent>>> {
        params.stream = Some(true);
        let response = self.client.post_raw_json("/responses", &params).await?;
        Ok(SseStream::new(response))
    }

    /// Retrieves a model response by ID.
    ///
    /// Pass `None` for `params` to retrieve with no query parameters, or pass
    /// `Some(&params)` to include filters like `include` on the request.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(
        &self,
        response_id: &str,
        params: Option<&ResponseGetParams>,
    ) -> Result<Response> {
        let path = format!("/responses/{}", response_id);
        match params {
            Some(p) => self.client.get_json_query(&path, p).await,
            None => self.client.get_json(&path).await,
        }
    }

    /// Retrieves a model response as a streaming SSE connection.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or stream decode failures.
    pub async fn get_stream(
        &self,
        response_id: &str,
        params: ResponseGetParams,
    ) -> Result<impl Stream<Item = Result<ResponseStreamEvent>>> {
        let path = format!("/responses/{}", response_id);
        #[derive(serde::Serialize)]
        struct StreamQuery {
            stream: bool,
            #[serde(flatten)]
            params: ResponseGetParams,
        }
        let query = StreamQuery {
            stream: true,
            params,
        };
        let response = self.client.get_raw_query(&path, &query).await?;
        Ok(SseStream::new(response))
    }

    /// Compacts a conversation. Returns a compacted response object.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn compact(&self, params: ResponseCompactParams) -> Result<CompactedResponse> {
        self.client.post_json("/responses/compact", &params).await
    }

    /// Deletes a model response by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(&self, response_id: &str) -> Result<DeletedResponse> {
        let path = format!("/responses/{}", response_id);
        self.client.delete_json(&path).await
    }

    /// Cancels a model response by ID. Only background responses can be cancelled.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn cancel(&self, response_id: &str) -> Result<Response> {
        let path = format!("/responses/{}/cancel", response_id);
        self.client.post_json(&path, &serde_json::Value::Null).await
    }

    /// Returns a paginated list of input items for a given response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list_input_items(
        &self,
        response_id: &str,
        params: Option<InputItemListParams>,
    ) -> Result<CursorPage<ResponseInputItem>> {
        let path = format!("/responses/{}/input_items", response_id);
        if let Some(p) = params {
            self.client.get_cursor_page_query(&path, &p).await
        } else {
            self.client.get_cursor_page(&path).await
        }
    }
}

/// Input token counting sub-service.
#[derive(Clone)]
pub struct InputTokenService {
    client: Client,
}

impl InputTokenService {
    /// Counts input tokens for the given request parameters.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn count(&self, params: InputTokenCountParams) -> Result<InputTokenCountResponse> {
        self.client
            .post_json("/responses/input_tokens", &params)
            .await
    }
}

/// Input item sub-service for listing and paginating input items.
#[derive(Clone)]
pub struct InputItemService {
    client: Client,
}

impl InputItemService {
    /// Returns a paginated list of input items for a given response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(
        &self,
        response_id: &str,
        params: Option<InputItemListParams>,
    ) -> Result<CursorPage<ResponseInputItem>> {
        let path = format!("/responses/{}/input_items", response_id);
        if let Some(p) = params {
            self.client.get_cursor_page_query(&path, &p).await
        } else {
            self.client.get_cursor_page(&path).await
        }
    }
}

// ---------------------------------------------------------------------------
// Response status
// ---------------------------------------------------------------------------

/// The status of a response generation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    /// Response completed successfully.
    Completed,
    /// Response generation failed.
    Failed,
    /// Response is currently being generated.
    InProgress,
    /// Response was cancelled.
    Cancelled,
    /// Response is queued for processing.
    Queued,
    /// Response is incomplete.
    Incomplete,
}

// ---------------------------------------------------------------------------
// Service tier
// ---------------------------------------------------------------------------

/// Processing tier for the request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTier {
    /// Use project settings.
    Auto,
    /// Standard pricing.
    Default,
    /// Flex processing.
    Flex,
    /// Scale processing.
    Scale,
    /// Priority processing.
    Priority,
}

// ---------------------------------------------------------------------------
// Truncation strategy
// ---------------------------------------------------------------------------

/// Truncation strategy for model responses.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Truncation {
    /// Automatically truncate to fit context window.
    Auto,
    /// Fail if input exceeds context window.
    Disabled,
}

// ---------------------------------------------------------------------------
// Includable fields
// ---------------------------------------------------------------------------

/// Additional fields that can be included in the response.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResponseIncludable {
    /// Include file search call results.
    #[serde(rename = "file_search_call.results")]
    FileSearchCallResults,
    /// Include web search call results.
    #[serde(rename = "web_search_call.results")]
    WebSearchCallResults,
    /// Include web search call action sources.
    #[serde(rename = "web_search_call.action.sources")]
    WebSearchCallActionSources,
    /// Include input image URLs.
    #[serde(rename = "message.input_image.image_url")]
    MessageInputImageImageUrl,
    /// Include computer call output image URLs.
    #[serde(rename = "computer_call_output.output.image_url")]
    ComputerCallOutputOutputImageUrl,
    /// Include code interpreter call outputs.
    #[serde(rename = "code_interpreter_call.outputs")]
    CodeInterpreterCallOutputs,
    /// Include encrypted reasoning content.
    #[serde(rename = "reasoning.encrypted_content")]
    ReasoningEncryptedContent,
    /// Include output text logprobs.
    #[serde(rename = "message.output_text.logprobs")]
    MessageOutputTextLogprobs,
}

// ---------------------------------------------------------------------------
// Prompt cache retention
// ---------------------------------------------------------------------------

/// Prompt cache retention policy.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PromptCacheRetention {
    /// In-memory caching.
    #[serde(rename = "in-memory")]
    InMemory,
    /// Extended 24-hour caching.
    #[serde(rename = "24h")]
    TwentyFourHours,
}

/// Prompt cache retention policy for response compaction requests.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResponseCompactPromptCacheRetention {
    /// In-memory caching.
    #[serde(rename = "in_memory")]
    InMemory,
    /// Extended 24-hour caching.
    #[serde(rename = "24h")]
    TwentyFourHours,
}

// ---------------------------------------------------------------------------
// Reasoning configuration
// ---------------------------------------------------------------------------

/// Reasoning effort level.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningEffort {
    /// No reasoning.
    None,
    /// Minimal effort reasoning.
    Minimal,
    /// Low effort reasoning.
    Low,
    /// Medium effort reasoning.
    Medium,
    /// High effort reasoning.
    High,
    /// Extra-high effort reasoning.
    Xhigh,
}

/// Reasoning summary mode.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningSummary {
    /// Automatic summary generation.
    Auto,
    /// Concise summaries.
    Concise,
    /// Detailed summaries.
    Detailed,
}

/// Configuration for reasoning models.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ReasoningConfig {
    /// Reasoning effort level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,
    /// Summary generation mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ReasoningSummary>,
    /// Generate summary configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_summary: Option<ReasoningSummary>,
}

// ---------------------------------------------------------------------------
// Text config
// ---------------------------------------------------------------------------

/// Verbosity level for text responses.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextVerbosity {
    /// Concise responses.
    Low,
    /// Default verbosity.
    Medium,
    /// Verbose responses.
    High,
}

/// Text response format type.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormatType {
    /// Plain text format.
    Text,
    /// JSON object format.
    JsonObject,
    /// JSON schema format for structured outputs.
    JsonSchema,
}

/// Response format configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseFormat {
    /// The format type.
    #[serde(rename = "type")]
    pub format_type: ResponseFormatType,
    /// JSON schema definition (required for `json_schema` type).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<JsonSchemaConfig>,
}

/// JSON schema configuration for structured outputs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsonSchemaConfig {
    /// Schema name.
    pub name: String,
    /// The JSON schema definition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    /// Whether to enforce strict schema validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    /// Description of the schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Text response configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TextConfig {
    /// Output format specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ResponseFormat>,
    /// Verbosity level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<TextVerbosity>,
}

// ---------------------------------------------------------------------------
// Tool choice
// ---------------------------------------------------------------------------

/// Tool choice mode.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoiceMode {
    /// Model decides which tool to call.
    Auto,
    /// No tool calls.
    None,
    /// Model must call at least one tool.
    Required,
}

/// Specifies a particular function the model must call.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolChoiceFunction {
    /// The type of tool choice. Always `function`.
    #[serde(rename = "type")]
    pub choice_type: String,
    /// The name of the function to call.
    pub name: String,
}

/// Constrains the tools available to the model to a pre-defined set.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolChoiceAllowed {
    /// The type. Always `allowed_tools`.
    #[serde(rename = "type")]
    pub choice_type: String,
    /// Mode: `auto` or `required`.
    pub mode: String,
    /// List of tool definitions.
    pub tools: Vec<serde_json::Value>,
}

/// Forces the model to use a built-in hosted tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolChoiceTypes {
    /// The type of hosted tool.
    #[serde(rename = "type")]
    pub choice_type: String,
}

/// Forces the model to call a specific MCP tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolChoiceMcp {
    /// The type. Always `mcp`.
    #[serde(rename = "type")]
    pub choice_type: String,
    /// The label of the MCP server.
    pub server_label: String,
    /// The name of the tool to call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Forces the model to call a specific custom tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolChoiceCustom {
    /// The type. Always `custom`.
    #[serde(rename = "type")]
    pub choice_type: String,
    /// The name of the custom tool.
    pub name: String,
}

/// Forces the model to call the apply_patch tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolChoiceApplyPatch {
    /// The type. Always `apply_patch`.
    #[serde(rename = "type")]
    pub choice_type: String,
}

/// Forces the model to call the shell tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolChoiceShell {
    /// The type. Always `shell`.
    #[serde(rename = "type")]
    pub choice_type: String,
}

/// How the model should select which tool to use.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// A mode keyword (auto, none, required).
    Mode(ToolChoiceMode),
    /// A specific function to call.
    Function(ToolChoiceFunction),
    /// Constrain the model to a pre-defined set of allowed tools.
    Allowed(ToolChoiceAllowed),
    /// Force the model to use a specific built-in tool type.
    Types(ToolChoiceTypes),
    /// Force the model to call a specific MCP tool.
    Mcp(ToolChoiceMcp),
    /// Force the model to call a specific custom tool.
    Custom(ToolChoiceCustom),
    /// Force the model to call the apply_patch tool.
    ApplyPatch(ToolChoiceApplyPatch),
    /// Force the model to call the shell tool.
    Shell(ToolChoiceShell),
}

// ---------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------

/// A function tool definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionTool {
    /// Tool type. Always `function`.
    #[serde(rename = "type")]
    pub tool_type: String,
    /// The function name.
    pub name: String,
    /// A description of what the function does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON schema for the function parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
    /// Whether to enforce strict parameter validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

/// A web search tool definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebSearchTool {
    /// Tool type. Always `web_search_preview`.
    #[serde(rename = "type")]
    pub tool_type: String,
    /// User location context for search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<WebSearchUserLocation>,
    /// Search context size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_context_size: Option<String>,
}

/// User location context for web search.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebSearchUserLocation {
    /// Location type. Always `approximate`.
    #[serde(rename = "type")]
    pub location_type: String,
    /// City name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Country code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// Region or state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// Timezone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

/// A file search tool definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileSearchTool {
    /// Tool type. Always `file_search`.
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Vector store IDs to search.
    pub vector_store_ids: Vec<String>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_num_results: Option<i64>,
    /// Ranking options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranking_options: Option<FileSearchRankingOptions>,
    /// File search filters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<serde_json::Value>,
}

/// Ranking options for file search.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileSearchRankingOptions {
    /// The ranker to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranker: Option<String>,
    /// Score threshold for results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_threshold: Option<f64>,
}

/// A code interpreter tool definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeInterpreterTool {
    /// Tool type. Always `code_interpreter`.
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Container ID to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
}

/// A computer use tool definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComputerUseTool {
    /// Tool type. Always `computer_use_preview`.
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Display width in pixels.
    pub display_width: i64,
    /// Display height in pixels.
    pub display_height: i64,
    /// Environment name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

/// An MCP (Model Context Protocol) tool definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpTool {
    /// Tool type. Always `mcp`.
    #[serde(rename = "type")]
    pub tool_type: String,
    /// The MCP server URL.
    pub server_url: String,
    /// Server label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_label: Option<String>,
    /// Allowed tools filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<serde_json::Value>,
    /// Require approval configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<serde_json::Value>,
    /// HTTP headers for the MCP connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

/// A tool the model may call while generating a response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Tool {
    /// A custom function tool.
    #[serde(rename = "function")]
    Function {
        /// The function name.
        name: String,
        /// A description of what the function does.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// JSON schema for parameters.
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<serde_json::Value>,
        /// Whether to enforce strict parameter validation.
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
    /// Web search tool.
    #[serde(rename = "web_search_preview")]
    WebSearch {
        /// User location context.
        #[serde(skip_serializing_if = "Option::is_none")]
        user_location: Option<WebSearchUserLocation>,
        /// Search context size.
        #[serde(skip_serializing_if = "Option::is_none")]
        search_context_size: Option<String>,
    },
    /// File search tool.
    #[serde(rename = "file_search")]
    FileSearch {
        /// Vector store IDs to search.
        vector_store_ids: Vec<String>,
        /// Maximum number of results.
        #[serde(skip_serializing_if = "Option::is_none")]
        max_num_results: Option<i64>,
        /// Ranking options.
        #[serde(skip_serializing_if = "Option::is_none")]
        ranking_options: Option<FileSearchRankingOptions>,
    },
    /// Code interpreter tool.
    #[serde(rename = "code_interpreter")]
    CodeInterpreter {
        /// Container ID to use.
        #[serde(skip_serializing_if = "Option::is_none")]
        container: Option<String>,
    },
    /// Computer use tool.
    #[serde(rename = "computer_use_preview")]
    ComputerUse {
        /// Display width in pixels.
        display_width: i64,
        /// Display height in pixels.
        display_height: i64,
        /// Environment name.
        #[serde(skip_serializing_if = "Option::is_none")]
        environment: Option<String>,
    },
    /// MCP tool.
    #[serde(rename = "mcp")]
    Mcp {
        /// The MCP server URL.
        server_url: String,
        /// Server label.
        #[serde(skip_serializing_if = "Option::is_none")]
        server_label: Option<String>,
        /// Allowed tools filter.
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_tools: Option<serde_json::Value>,
        /// Require approval configuration.
        #[serde(skip_serializing_if = "Option::is_none")]
        require_approval: Option<serde_json::Value>,
        /// HTTP headers for the MCP connection.
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
    },
    /// Apply patch tool for creating, deleting, or updating files.
    #[serde(rename = "apply_patch")]
    ApplyPatch,
    /// Computer tool (the new `computer` type, distinct from `computer_use_preview`).
    #[serde(rename = "computer")]
    Computer,
    /// Image generation tool.
    #[serde(rename = "image_generation")]
    ImageGeneration {
        /// Optional image generation options.
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        options: Option<ImageGenerationToolOptions>,
    },
    /// Local shell tool for executing commands locally.
    #[serde(rename = "local_shell")]
    LocalShell {
        /// Environment configuration.
        #[serde(skip_serializing_if = "Option::is_none")]
        environment: Option<serde_json::Value>,
    },
    /// Shell tool for executing commands in a container.
    #[serde(rename = "shell")]
    Shell {
        /// Environment configuration.
        #[serde(skip_serializing_if = "Option::is_none")]
        environment: Option<serde_json::Value>,
    },
    /// Tool search for deferred tool discovery.
    #[serde(rename = "tool_search")]
    ToolSearch {
        /// Description shown to the model.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// Whether tool search is executed by the server or client.
        #[serde(skip_serializing_if = "Option::is_none")]
        execution: Option<String>,
        /// Parameter schema for client-executed tool search.
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<serde_json::Value>,
    },
    /// Namespace tool that groups tools under a named scope.
    #[serde(rename = "namespace")]
    Namespace {
        /// The namespace name.
        name: String,
        /// A description of the namespace.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// The function/custom tools available inside this namespace.
        tools: Vec<serde_json::Value>,
    },
    /// Custom tool that processes input using a specified format.
    #[serde(rename = "custom")]
    Custom {
        /// The name of the custom tool.
        name: String,
        /// Optional description.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// Whether this tool should be deferred and discovered via tool search.
        #[serde(skip_serializing_if = "Option::is_none")]
        defer_loading: Option<bool>,
        /// The input format for the custom tool.
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<serde_json::Value>,
    },
    /// Web search tool (the new `web_search` type with filters support).
    #[serde(rename = "web_search")]
    WebSearchNew {
        /// Filters for the search.
        #[serde(skip_serializing_if = "Option::is_none")]
        filters: Option<WebSearchToolFilters>,
        /// Search context size.
        #[serde(skip_serializing_if = "Option::is_none")]
        search_context_size: Option<String>,
        /// User location context.
        #[serde(skip_serializing_if = "Option::is_none")]
        user_location: Option<WebSearchUserLocation>,
    },
}

/// Filters for web search results.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebSearchToolFilters {
    /// Allowed domains for the search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,
}

/// Options for the image generation tool.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ImageGenerationToolOptions {
    /// Whether to generate a new image or edit an existing one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// Background type for the generated image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    /// The image generation model to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Output format of the generated image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    /// Compression level for the output image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_compression: Option<i64>,
    /// Quality of the generated image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// Size of the generated image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Number of partial images to generate in streaming mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_images: Option<i64>,
}

// ---------------------------------------------------------------------------
// Input types
// ---------------------------------------------------------------------------

/// Text or structured input to the model.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ResponseInput {
    /// Simple text input.
    Text(String),
    /// A list of input items for structured conversation input.
    Items(Vec<ResponseInputItem>),
}

impl From<String> for ResponseInput {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for ResponseInput {
    fn from(s: &str) -> Self {
        Self::Text(s.to_owned())
    }
}

impl From<Vec<ResponseInputItem>> for ResponseInput {
    fn from(items: Vec<ResponseInputItem>) -> Self {
        Self::Items(items)
    }
}

impl ResponseInput {
    /// Returns the union value as JSON for dynamic matching.
    #[must_use]
    pub fn as_any(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Returns the text variant.
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value),
            Self::Items(_) => None,
        }
    }

    /// Returns the structured items variant.
    #[must_use]
    pub fn as_items(&self) -> Option<&[ResponseInputItem]> {
        match self {
            Self::Text(_) => None,
            Self::Items(value) => Some(value),
        }
    }

    /// Creates a text input union variant.
    #[must_use]
    pub fn param_of_text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// Creates a structured input union variant.
    #[must_use]
    pub fn param_of_items(value: Vec<ResponseInputItem>) -> Self {
        Self::Items(value)
    }
}

/// Role for input messages.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputMessageRole {
    /// User input.
    User,
    /// Assistant output.
    Assistant,
    /// System instruction.
    System,
    /// Developer instruction.
    Developer,
}

/// Detail level for file inputs.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseInputFileDetail {
    /// Default file rendering quality.
    Low,
    /// Higher-quality file rendering.
    High,
}

/// Content part for input messages.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum InputContentPart {
    /// Text content.
    #[serde(rename = "input_text")]
    InputText {
        /// The text content.
        text: String,
    },
    /// Image content from a URL.
    #[serde(rename = "input_image")]
    InputImage {
        /// Image URL.
        #[serde(skip_serializing_if = "Option::is_none")]
        image_url: Option<String>,
        /// Base64 file data.
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
        /// Detail level for vision.
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
    /// File content.
    #[serde(rename = "input_file")]
    InputFile {
        /// File ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
        /// Filename.
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
        /// File rendering detail level.
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<ResponseInputFileDetail>,
        /// Base64 file data.
        #[serde(skip_serializing_if = "Option::is_none")]
        file_data: Option<String>,
    },
}

impl InputContentPart {
    /// Returns the union value as JSON for dynamic matching.
    #[must_use]
    pub fn as_any(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Returns true when this is an `input_text` part.
    #[must_use]
    pub fn is_input_text(&self) -> bool {
        matches!(self, Self::InputText { .. })
    }

    /// Returns true when this is an `input_image` part.
    #[must_use]
    pub fn is_input_image(&self) -> bool {
        matches!(self, Self::InputImage { .. })
    }

    /// Returns true when this is an `input_file` part.
    #[must_use]
    pub fn is_input_file(&self) -> bool {
        matches!(self, Self::InputFile { .. })
    }

    /// Creates an `input_text` part.
    #[must_use]
    pub fn param_of_input_text(text: impl Into<String>) -> Self {
        Self::InputText { text: text.into() }
    }
}

/// Message content: either a simple string or structured content parts.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum InputContent {
    /// Simple text content.
    Text(String),
    /// Structured content parts.
    Parts(Vec<InputContentPart>),
}

impl From<String> for InputContent {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for InputContent {
    fn from(s: &str) -> Self {
        Self::Text(s.to_owned())
    }
}

impl InputContent {
    /// Returns the union value as JSON for dynamic matching.
    #[must_use]
    pub fn as_any(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Returns the text variant.
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value),
            Self::Parts(_) => None,
        }
    }

    /// Returns the structured parts variant.
    #[must_use]
    pub fn as_parts(&self) -> Option<&[InputContentPart]> {
        match self {
            Self::Text(_) => None,
            Self::Parts(value) => Some(value),
        }
    }

    /// Creates a text content union variant.
    #[must_use]
    pub fn param_of_text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// Creates a structured content union variant.
    #[must_use]
    pub fn param_of_parts(value: Vec<InputContentPart>) -> Self {
        Self::Parts(value)
    }
}

/// An input item in a response conversation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ResponseInputItem {
    /// A message input item.
    #[serde(rename = "message")]
    Message {
        /// The message role.
        role: InputMessageRole,
        /// The message content.
        content: InputContent,
        /// Optional message status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    /// A function call output from the user.
    #[serde(rename = "function_call_output")]
    FunctionCallOutput {
        /// The call ID this output is for.
        call_id: String,
        /// The function output value.
        output: String,
    },
    /// An item reference for conversation state.
    #[serde(rename = "item_reference")]
    ItemReference {
        /// The item ID to reference.
        id: String,
    },
    /// A computer call output from the user.
    #[serde(rename = "computer_call_output")]
    ComputerCallOutput {
        /// The call ID this output is for.
        call_id: String,
        /// The output (screenshots, etc).
        output: serde_json::Value,
        /// Acknowledged safety checks.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        acknowledged_safety_checks: Vec<AcknowledgedSafetyCheck>,
    },
    /// MCP list tools result.
    #[serde(rename = "mcp_list_tools")]
    McpListTools {
        /// Server label.
        server_label: String,
        /// Discovered tools.
        tools: Vec<McpToolInfo>,
    },
    /// MCP approval request.
    #[serde(rename = "mcp_approval_request")]
    McpApprovalRequest {
        /// The approval request ID.
        id: String,
        /// The name of the tool.
        name: String,
        /// The arguments for the tool.
        arguments: String,
        /// The MCP server label.
        server_label: String,
    },
    /// MCP approval response.
    #[serde(rename = "mcp_approval_response")]
    McpApprovalResponse {
        /// The approval request ID being answered.
        approval_request_id: String,
        /// Whether the request was approved.
        approve: bool,
        /// Optional reason for the decision.
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    /// An MCP tool call.
    #[serde(rename = "mcp_call")]
    McpCall {
        /// The unique ID.
        id: String,
        /// The tool name.
        name: String,
        /// JSON-encoded arguments.
        arguments: String,
        /// The MCP server label.
        server_label: String,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
        /// Output from the tool call.
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<String>,
        /// Error from the tool call.
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    /// A local shell call.
    #[serde(rename = "local_shell_call")]
    LocalShellCall {
        /// The unique ID.
        id: String,
        /// The call ID.
        call_id: String,
        /// The action to execute.
        action: serde_json::Value,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    /// A local shell call output.
    #[serde(rename = "local_shell_call_output")]
    LocalShellCallOutput {
        /// The unique ID.
        id: String,
        /// The output of the shell call.
        output: String,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    /// A shell call (container-based).
    #[serde(rename = "shell_call")]
    ShellCall {
        /// The unique ID.
        id: String,
        /// The call ID.
        call_id: String,
        /// The action to execute.
        action: serde_json::Value,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    /// A shell call output (container-based).
    #[serde(rename = "shell_call_output")]
    ShellCallOutput {
        /// The unique ID.
        id: String,
        /// The output of the shell call.
        output: String,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    /// An apply patch call.
    #[serde(rename = "apply_patch_call")]
    ApplyPatchCall {
        /// The unique ID.
        id: String,
        /// The call ID.
        call_id: String,
        /// The patch operations.
        #[serde(default)]
        operations: Vec<serde_json::Value>,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    /// An apply patch call output.
    #[serde(rename = "apply_patch_call_output")]
    ApplyPatchCallOutput {
        /// The unique ID.
        id: String,
        /// The output of the patch call.
        output: String,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    /// An image generation call.
    #[serde(rename = "image_generation_call")]
    ImageGenerationCall {
        /// The unique ID.
        id: String,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
        /// Generated image result.
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<String>,
    },
    /// A tool search call.
    #[serde(rename = "tool_search_call")]
    ToolSearchCall {
        /// The unique ID.
        id: String,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
}

impl ResponseInputItem {
    /// Returns the union value as JSON for dynamic matching.
    #[must_use]
    pub fn as_any(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Returns true when this is a message input item.
    #[must_use]
    pub fn is_message(&self) -> bool {
        matches!(self, Self::Message { .. })
    }

    /// Returns true when this is a function call output item.
    #[must_use]
    pub fn is_function_call_output(&self) -> bool {
        matches!(self, Self::FunctionCallOutput { .. })
    }

    /// Returns true when this is an item reference.
    #[must_use]
    pub fn is_item_reference(&self) -> bool {
        matches!(self, Self::ItemReference { .. })
    }

    /// Creates a message input item.
    #[must_use]
    pub fn param_of_message(role: InputMessageRole, content: InputContent) -> Self {
        Self::Message {
            role,
            content,
            status: None,
        }
    }

    /// Creates a function call output input item.
    #[must_use]
    pub fn param_of_function_call_output(
        call_id: impl Into<String>,
        output: impl Into<String>,
    ) -> Self {
        Self::FunctionCallOutput {
            call_id: call_id.into(),
            output: output.into(),
        }
    }

    /// Creates an item reference input item.
    #[must_use]
    pub fn param_of_item_reference(id: impl Into<String>) -> Self {
        Self::ItemReference { id: id.into() }
    }
}

/// Acknowledged safety check for computer call outputs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcknowledgedSafetyCheck {
    /// The safety check ID.
    pub id: String,
    /// The safety check code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// The safety check message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Information about a tool discovered via MCP.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpToolInfo {
    /// The tool name.
    pub name: String,
    /// The JSON schema describing the tool's input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    /// The description of the tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Additional annotations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

/// Content part in an output message.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum OutputContent {
    /// Text output.
    #[serde(rename = "output_text")]
    OutputText {
        /// The output text.
        text: String,
        /// Text annotations (citations, file paths, etc).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        annotations: Vec<Annotation>,
        /// Logprob data when requested.
        #[serde(skip_serializing_if = "Option::is_none")]
        logprobs: Option<Vec<Logprob>>,
    },
    /// A refusal from the model.
    #[serde(rename = "refusal")]
    Refusal {
        /// The refusal message.
        refusal: String,
    },
}

/// A text annotation (citation, file path, URL).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Annotation {
    /// A file citation annotation.
    #[serde(rename = "file_citation")]
    FileCitation {
        /// File ID.
        file_id: String,
        /// Index in the text.
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<i64>,
        /// File name.
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
    },
    /// A URL citation annotation.
    #[serde(rename = "url_citation")]
    UrlCitation {
        /// The URL.
        url: String,
        /// Title of the cited page.
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// Start index in the text.
        #[serde(skip_serializing_if = "Option::is_none")]
        start_index: Option<i64>,
        /// End index in the text.
        #[serde(skip_serializing_if = "Option::is_none")]
        end_index: Option<i64>,
        /// Snippet from the cited content.
        #[serde(skip_serializing_if = "Option::is_none")]
        snippet: Option<String>,
        /// Source information.
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
    },
    /// A file path annotation.
    #[serde(rename = "file_path")]
    FilePath {
        /// File ID.
        file_id: String,
        /// Index in the text.
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<i64>,
    },
    /// A container file citation annotation.
    #[serde(rename = "container_file_citation")]
    ContainerFileCitation {
        /// The container ID.
        container_id: String,
        /// The file ID.
        file_id: String,
        /// The filename.
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
        /// Start index in the text.
        #[serde(skip_serializing_if = "Option::is_none")]
        start_index: Option<i64>,
        /// End index in the text.
        #[serde(skip_serializing_if = "Option::is_none")]
        end_index: Option<i64>,
    },
}

/// Log probability data for a token.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Logprob {
    /// The token string.
    pub token: String,
    /// Log probability of the token.
    pub logprob: f64,
    /// Byte offsets of the token in the original text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
    /// Top alternative tokens and their log probabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<Vec<TopLogprob>>,
}

/// A top alternative token with its log probability.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TopLogprob {
    /// The token string.
    pub token: String,
    /// Log probability of the token.
    pub logprob: f64,
    /// Byte offsets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
}

/// Output message status.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMessageStatus {
    /// Message is being generated.
    InProgress,
    /// Message generation completed.
    Completed,
    /// Message is incomplete.
    Incomplete,
}

/// Phase of an assistant message (commentary vs final answer).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMessagePhase {
    /// Intermediate commentary.
    Commentary,
    /// The final answer.
    FinalAnswer,
}

/// An output item generated by the model.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ResponseOutputItem {
    /// An assistant message.
    #[serde(rename = "message")]
    Message {
        /// Unique ID of this output item.
        id: String,
        /// The role. Always `assistant`.
        #[serde(default = "default_assistant_role")]
        role: String,
        /// The output content.
        content: Vec<OutputContent>,
        /// Status of the message.
        status: OutputMessageStatus,
        /// Phase of the message.
        #[serde(skip_serializing_if = "Option::is_none")]
        phase: Option<OutputMessagePhase>,
    },
    /// A file search tool call.
    #[serde(rename = "file_search_call")]
    FileSearchCall {
        /// Unique ID.
        id: String,
        /// Status.
        status: String,
        /// Search queries issued.
        #[serde(default)]
        queries: Vec<String>,
        /// Search results.
        #[serde(skip_serializing_if = "Option::is_none")]
        results: Option<Vec<FileSearchResult>>,
    },
    /// A function call.
    #[serde(rename = "function_call")]
    FunctionCall {
        /// Unique ID.
        id: String,
        /// The call ID.
        call_id: String,
        /// Function name.
        name: String,
        /// JSON-encoded arguments.
        arguments: String,
        /// Status.
        status: String,
    },
    /// A function call output (echoed back).
    #[serde(rename = "function_call_output")]
    FunctionCallOutput {
        /// Unique ID.
        id: String,
        /// The call ID this output is for.
        call_id: String,
        /// The output value.
        output: String,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    /// A web search tool call.
    #[serde(rename = "web_search_call")]
    WebSearchCall {
        /// Unique ID.
        id: String,
        /// Status.
        status: String,
        /// Search action info.
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<serde_json::Value>,
    },
    /// A computer use tool call.
    #[serde(rename = "computer_call")]
    ComputerCall {
        /// Unique ID.
        id: String,
        /// The call ID.
        call_id: String,
        /// Status.
        status: String,
        /// Action details (typed as ComputerAction).
        action: ComputerAction,
        /// Pending safety checks.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pending_safety_checks: Vec<serde_json::Value>,
    },
    /// A reasoning item.
    #[serde(rename = "reasoning")]
    Reasoning {
        /// Unique ID.
        id: String,
        /// Reasoning summary items.
        #[serde(default)]
        summary: Vec<ReasoningSummaryItem>,
        /// Encrypted reasoning content.
        #[serde(skip_serializing_if = "Option::is_none")]
        encrypted_content: Option<String>,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    /// A code interpreter tool call.
    #[serde(rename = "code_interpreter_call")]
    CodeInterpreterCall {
        /// Unique ID.
        id: String,
        /// The code executed.
        code: String,
        /// Status.
        status: String,
        /// Container ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        container_id: Option<String>,
        /// Call outputs.
        #[serde(default)]
        outputs: Vec<serde_json::Value>,
    },
    /// An image generation call.
    #[serde(rename = "image_generation_call")]
    ImageGenerationCall {
        /// Unique ID.
        id: String,
        /// Status.
        status: String,
        /// Generated image result (base64 or URL).
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<String>,
    },
    /// An MCP tool call.
    #[serde(rename = "mcp_call")]
    McpCall {
        /// Unique ID.
        id: String,
        /// Status.
        status: String,
        /// Server label.
        #[serde(skip_serializing_if = "Option::is_none")]
        server_label: Option<String>,
        /// Tool name.
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        /// JSON-encoded arguments.
        #[serde(skip_serializing_if = "Option::is_none")]
        arguments: Option<String>,
        /// Tool output.
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<serde_json::Value>,
        /// Error message if failed.
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    /// An MCP list tools result.
    #[serde(rename = "mcp_list_tools")]
    McpListTools {
        /// Unique ID.
        id: String,
        /// Status.
        status: String,
        /// Server label.
        #[serde(skip_serializing_if = "Option::is_none")]
        server_label: Option<String>,
        /// Discovered tools.
        #[serde(default)]
        tools: Vec<serde_json::Value>,
    },
    /// A compaction summary item.
    #[serde(rename = "compaction")]
    Compaction {
        /// Unique ID.
        id: String,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    /// An MCP approval request output item.
    #[serde(rename = "mcp_approval_request")]
    McpApprovalRequest {
        /// Unique ID.
        id: String,
        /// The tool name.
        name: String,
        /// JSON arguments.
        arguments: String,
        /// Server label.
        server_label: String,
    },
    /// An MCP approval response output item.
    #[serde(rename = "mcp_approval_response")]
    McpApprovalResponse {
        /// Unique ID.
        id: String,
        /// The approval request ID being answered.
        approval_request_id: String,
        /// Whether the request was approved.
        approve: bool,
        /// Optional reason.
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    /// A local shell call output item.
    #[serde(rename = "local_shell_call")]
    LocalShellCall {
        /// Unique ID.
        id: String,
        /// The call ID.
        call_id: String,
        /// The action.
        action: serde_json::Value,
        /// Status.
        status: String,
    },
    /// A local shell call output result.
    #[serde(rename = "local_shell_call_output")]
    LocalShellCallOutput {
        /// Unique ID.
        id: String,
        /// The output string.
        output: String,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    /// A shell call (container-based) output item.
    #[serde(rename = "shell_call")]
    ShellCall {
        /// Unique ID.
        id: String,
        /// The call ID.
        call_id: String,
        /// The action.
        action: serde_json::Value,
        /// Status.
        status: String,
    },
    /// A shell call output result.
    #[serde(rename = "shell_call_output")]
    ShellCallOutput {
        /// Unique ID.
        id: String,
        /// The output string.
        output: String,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    /// An apply patch call output item.
    #[serde(rename = "apply_patch_call")]
    ApplyPatchCall {
        /// Unique ID.
        id: String,
        /// The call ID.
        call_id: String,
        /// The patch operations.
        #[serde(default)]
        operations: Vec<serde_json::Value>,
        /// Status.
        status: String,
    },
    /// An apply patch call output result.
    #[serde(rename = "apply_patch_call_output")]
    ApplyPatchCallOutput {
        /// Unique ID.
        id: String,
        /// The output string.
        output: String,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    /// A custom tool call output item.
    #[serde(rename = "custom_tool_call")]
    CustomToolCall {
        /// Unique ID.
        id: String,
        /// The call ID.
        call_id: String,
        /// Tool name.
        name: String,
        /// Input to the custom tool.
        #[serde(skip_serializing_if = "Option::is_none")]
        input: Option<String>,
        /// Status.
        status: String,
    },
    /// A custom tool call output result.
    #[serde(rename = "custom_tool_call_output")]
    CustomToolCallOutput {
        /// Unique ID.
        id: String,
        /// The call ID.
        call_id: String,
        /// Output from the custom tool.
        output: serde_json::Value,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    /// A tool search call output item.
    #[serde(rename = "tool_search_call")]
    ToolSearchCall {
        /// Unique ID.
        id: String,
        /// Status.
        status: String,
    },
    /// A tool search output result.
    #[serde(rename = "tool_search_output")]
    ToolSearchOutput {
        /// Unique ID.
        id: String,
        /// The tools found.
        #[serde(default)]
        tools: Vec<serde_json::Value>,
        /// Status.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
}

impl ResponseOutputItem {
    /// Returns the union value as JSON for dynamic matching.
    #[must_use]
    pub fn as_any(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Returns true when this is an assistant message.
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

    /// Creates a function call output item.
    #[must_use]
    pub fn param_of_function_call_output(
        id: impl Into<String>,
        call_id: impl Into<String>,
        output: impl Into<String>,
    ) -> Self {
        Self::FunctionCallOutput {
            id: id.into(),
            call_id: call_id.into(),
            output: output.into(),
            status: None,
        }
    }
}

fn default_assistant_role() -> String {
    "assistant".to_owned()
}

// ---------------------------------------------------------------------------
// Computer actions
// ---------------------------------------------------------------------------

/// Coordinates on the screen.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScreenCoord {
    /// X coordinate.
    pub x: i64,
    /// Y coordinate.
    pub y: i64,
}

/// Path point for drag operations.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DragPathPoint {
    /// X coordinate.
    pub x: i64,
    /// Y coordinate.
    pub y: i64,
}

/// An action performed by the computer use tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ComputerAction {
    /// Click at a position.
    #[serde(rename = "click")]
    Click {
        /// X coordinate.
        x: i64,
        /// Y coordinate.
        y: i64,
        /// Mouse button (left, right, middle, etc.).
        #[serde(skip_serializing_if = "Option::is_none")]
        button: Option<String>,
    },
    /// Double-click at a position.
    #[serde(rename = "double_click")]
    DoubleClick {
        /// X coordinate.
        x: i64,
        /// Y coordinate.
        y: i64,
    },
    /// Drag from one position to another.
    #[serde(rename = "drag")]
    Drag {
        /// Path of coordinates.
        path: Vec<DragPathPoint>,
    },
    /// Drag along a path (alternative drag action).
    #[serde(rename = "drag_path")]
    DragPath {
        /// Path of coordinates.
        path: Vec<DragPathPoint>,
    },
    /// Press one or more keys.
    #[serde(rename = "keypress")]
    Keypress {
        /// Keys to press.
        keys: Vec<String>,
    },
    /// Move the cursor to a position.
    #[serde(rename = "move")]
    Move {
        /// X coordinate.
        x: i64,
        /// Y coordinate.
        y: i64,
    },
    /// Take a screenshot.
    #[serde(rename = "screenshot")]
    Screenshot,
    /// Scroll at a position.
    #[serde(rename = "scroll")]
    Scroll {
        /// X coordinate.
        x: i64,
        /// Y coordinate.
        y: i64,
        /// Horizontal scroll amount.
        scroll_x: i64,
        /// Vertical scroll amount.
        scroll_y: i64,
    },
    /// Type text.
    #[serde(rename = "type")]
    Type {
        /// Text to type.
        text: String,
    },
    /// Wait for a specified duration.
    #[serde(rename = "wait")]
    Wait,
}

/// Reasoning summary content.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReasoningSummaryItem {
    /// Content type. Usually `summary_text`.
    #[serde(rename = "type")]
    pub item_type: String,
    /// The summary text.
    pub text: String,
}

/// A file search result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileSearchResult {
    /// File ID.
    pub file_id: String,
    /// File name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// Relevance score.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    /// The matched text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Search result attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, serde_json::Value>>,
}

// ---------------------------------------------------------------------------
// Usage
// ---------------------------------------------------------------------------

/// Token usage details.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseUsage {
    /// Number of input tokens.
    pub input_tokens: u64,
    /// Detailed breakdown of input tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens_details: Option<InputTokensDetails>,
    /// Number of output tokens.
    pub output_tokens: u64,
    /// Detailed breakdown of output tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens_details: Option<OutputTokensDetails>,
    /// Total number of tokens.
    pub total_tokens: u64,
}

/// Detailed breakdown of input tokens.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InputTokensDetails {
    /// Number of cached tokens.
    pub cached_tokens: u64,
}

/// Detailed breakdown of output tokens.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutputTokensDetails {
    /// Number of reasoning tokens.
    pub reasoning_tokens: u64,
}

// ---------------------------------------------------------------------------
// Incomplete details
// ---------------------------------------------------------------------------

/// Details about why a response is incomplete.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IncompleteDetails {
    /// The reason why the response is incomplete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Conversation
// ---------------------------------------------------------------------------

/// Conversation reference in a response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseConversation {
    /// The conversation ID.
    pub id: String,
}

// ---------------------------------------------------------------------------
// Response error
// ---------------------------------------------------------------------------

/// Error information when a response generation fails.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseError {
    /// Error code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Error message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// ---------------------------------------------------------------------------
// Prompt reference
// ---------------------------------------------------------------------------

/// Reference to a prompt template.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponsePrompt {
    /// Prompt template ID.
    pub id: String,
    /// Version identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Variable bindings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, serde_json::Value>>,
}

// ---------------------------------------------------------------------------
// Main Response type
// ---------------------------------------------------------------------------

/// A model response object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Response {
    /// Unique identifier for this response.
    pub id: String,
    /// Object type. Always `response`.
    pub object: String,
    /// Unix timestamp (in seconds) of when this response was created.
    pub created_at: f64,
    /// The status of the response generation.
    pub status: ResponseStatus,
    /// Error information when status is `failed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
    /// Details about why the response is incomplete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incomplete_details: Option<IncompleteDetails>,
    /// Instructions (system message) for this response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Model ID used to generate the response.
    pub model: ModelId,
    /// Output items generated by the model.
    pub output: Vec<ResponseOutputItem>,
    /// Whether parallel tool calls were enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    /// Sampling temperature used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// How the model selected tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    /// Tools available to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    /// Nucleus sampling parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Whether this was a background response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    /// Unix timestamp when response completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<f64>,
    /// Conversation reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<ResponseConversation>,
    /// Maximum output tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i64>,
    /// Maximum number of tool calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<i64>,
    /// Key-value metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Previous response ID for multi-turn conversations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    /// Prompt template reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<ResponsePrompt>,
    /// Prompt cache key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    /// Prompt cache retention policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<PromptCacheRetention>,
    /// Reasoning configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
    /// Safety identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,
    /// Service tier used for the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    /// Text response configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextConfig>,
    /// Number of top logprobs to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<i64>,
    /// Truncation strategy used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<Truncation>,
    /// Token usage details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ResponseUsage>,
    /// End-user identifier (deprecated, use `prompt_cache_key` instead).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl Response {
    /// Extracts the concatenated output text from all `output_text` content parts.
    #[must_use]
    pub fn output_text(&self) -> String {
        let mut text = String::new();
        for item in &self.output {
            if let ResponseOutputItem::Message { content, .. } = item {
                for part in content {
                    if let OutputContent::OutputText { text: t, .. } = part {
                        text.push_str(t);
                    }
                }
            }
        }
        text
    }
}

/// Deletion confirmation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeletedResponse {
    /// The deleted response ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Whether the response was deleted.
    pub deleted: bool,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Context management entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextManagement {
    /// Context management entry type. Currently only `compaction`.
    #[serde(rename = "type")]
    pub entry_type: String,
    /// Token threshold for compaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compact_threshold: Option<i64>,
}

/// Conversation parameter - either an ID string or a conversation object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ConversationParam {
    /// A conversation ID string.
    Id(String),
    /// A conversation object reference.
    Object(ResponseConversation),
}

/// Stream options.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StreamOptions {
    /// Whether to include obfuscation data in stream events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_obfuscation: Option<bool>,
}

/// Request payload for creating a response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseCreateParams {
    /// Model ID used to generate the response.
    pub model: ModelId,
    /// Text, image, or file inputs to the model.
    pub input: ResponseInput,

    /// Whether to run the model response in the background.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    /// Context management configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_management: Option<Vec<ContextManagement>>,
    /// Conversation reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<ConversationParam>,
    /// Additional output data to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<ResponseIncludable>>,
    /// System or developer instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Maximum number of output tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i64>,
    /// Maximum number of tool calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<i64>,
    /// Key-value metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Whether to allow parallel tool calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    /// Previous response ID for multi-turn conversations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    /// Prompt template reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<ResponsePrompt>,
    /// Prompt cache key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    /// Prompt cache retention policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<PromptCacheRetention>,
    /// Reasoning model configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
    /// Safety identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,
    /// Service tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    /// Whether to store the response for later retrieval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    /// Enables SSE streaming mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Options for streaming responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,
    /// Sampling temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Text response configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextConfig>,
    /// Tool choice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    /// Tools available to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    /// Number of top logprobs to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<i64>,
    /// Nucleus sampling parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Truncation strategy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<Truncation>,
    /// End-user identifier (deprecated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Container ID for code interpreter and shell tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
}

/// Query parameters for retrieving a response.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ResponseGetParams {
    /// Additional fields to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<ResponseIncludable>>,
    /// Whether to include obfuscation data in streaming.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_obfuscation: Option<bool>,
    /// Start streaming from after this sequence number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starting_after: Option<i64>,
}

/// Ordering for input item lists.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputItemListOrder {
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

/// Query parameters for listing input items.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct InputItemListParams {
    /// Cursor for pagination (list items after this ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Maximum number of items to return (1-100, default 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Additional fields to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<ResponseIncludable>>,
    /// Sort order (default: desc).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<InputItemListOrder>,
}

// ---------------------------------------------------------------------------
// Input token counting
// ---------------------------------------------------------------------------

/// Request payload for counting input tokens.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InputTokenCountParams {
    /// Model ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelId>,
    /// Text, image, or file inputs to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<ResponseInput>,
    /// System/developer instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Whether to allow parallel tool calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    /// Previous response ID for multi-turn conversations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    /// Conversation reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<ConversationParam>,
    /// Text response configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextConfig>,
    /// Tool choice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    /// Tools available to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    /// Reasoning configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
    /// Truncation strategy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<Truncation>,
}

/// Response from the input token counting endpoint.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InputTokenCountResponse {
    /// Number of input tokens.
    pub input_tokens: i64,
    /// Object type. Always `response.input_tokens`.
    pub object: String,
}

// ---------------------------------------------------------------------------
// Compaction
// ---------------------------------------------------------------------------

/// Request payload for compacting a conversation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseCompactParams {
    /// Model ID.
    pub model: ModelId,
    /// System/developer instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Previous response ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    /// Prompt cache key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    /// Text, image, or file inputs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<ResponseInput>,
    /// Prompt cache retention policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<ResponseCompactPromptCacheRetention>,
}

/// A compacted response object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompactedResponse {
    /// The unique identifier.
    pub id: String,
    /// Unix timestamp (in seconds) when the compaction was created.
    pub created_at: f64,
    /// Object type. Always `response.compaction`.
    pub object: String,
    /// The compacted list of output items.
    pub output: Vec<ResponseOutputItem>,
    /// Token usage for the compaction.
    pub usage: ResponseUsage,
}

// ---------------------------------------------------------------------------
// Stream events
// ---------------------------------------------------------------------------

/// A server-sent event from a streaming response.
///
/// The event `type` field discriminates the variant. The `ResponseStreamEvent`
/// is deserialized as a tagged enum where the `type` JSON field determines
/// which variant is active.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ResponseStreamEvent {
    // -- Lifecycle events --
    /// Emitted when the response is first created.
    #[serde(rename = "response.created")]
    ResponseCreated {
        /// The response object.
        response: Response,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when the response begins processing.
    #[serde(rename = "response.in_progress")]
    ResponseInProgress {
        /// The response object.
        response: Response,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when the response is completed.
    #[serde(rename = "response.completed")]
    ResponseCompleted {
        /// The completed response object.
        response: Response,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when the response fails.
    #[serde(rename = "response.failed")]
    ResponseFailed {
        /// The failed response object.
        response: Response,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when the response is incomplete.
    #[serde(rename = "response.incomplete")]
    ResponseIncomplete {
        /// The incomplete response object.
        response: Response,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when the response is queued.
    #[serde(rename = "response.queued")]
    ResponseQueued {
        /// The queued response object.
        response: Response,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },

    // -- Output item events --
    /// Emitted when a new output item is added.
    #[serde(rename = "response.output_item.added")]
    OutputItemAdded {
        /// The output item.
        item: ResponseOutputItem,
        /// Index in the output array.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when an output item is completed.
    #[serde(rename = "response.output_item.done")]
    OutputItemDone {
        /// The completed output item.
        item: ResponseOutputItem,
        /// Index in the output array.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },

    // -- Content part events --
    /// Emitted when a content part is added.
    #[serde(rename = "response.content_part.added")]
    ContentPartAdded {
        /// The content part.
        part: OutputContent,
        /// Index of the output item.
        output_index: i64,
        /// Index of the content part within the item.
        content_index: i64,
        /// Item ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        item_id: Option<String>,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when a content part is completed.
    #[serde(rename = "response.content_part.done")]
    ContentPartDone {
        /// The completed content part.
        part: OutputContent,
        /// Index of the output item.
        output_index: i64,
        /// Index of the content part within the item.
        content_index: i64,
        /// Item ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        item_id: Option<String>,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },

    // -- Text delta events --
    /// Emitted for incremental text output.
    #[serde(rename = "response.output_text.delta")]
    OutputTextDelta {
        /// The text delta.
        delta: String,
        /// Index of the output item.
        output_index: i64,
        /// Index of the content part.
        content_index: i64,
        /// Item ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        item_id: Option<String>,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when text output is complete.
    #[serde(rename = "response.output_text.done")]
    OutputTextDone {
        /// The full text.
        text: String,
        /// Index of the output item.
        output_index: i64,
        /// Index of the content part.
        content_index: i64,
        /// Item ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        item_id: Option<String>,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when a text annotation is added.
    #[serde(rename = "response.output_text.annotation.added")]
    OutputTextAnnotationAdded {
        /// The annotation.
        annotation: serde_json::Value,
        /// Annotation index.
        annotation_index: i64,
        /// Index of the output item.
        output_index: i64,
        /// Index of the content part.
        content_index: i64,
        /// Item ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        item_id: Option<String>,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },

    // -- Refusal events --
    /// Emitted for incremental refusal output.
    #[serde(rename = "response.refusal.delta")]
    RefusalDelta {
        /// The refusal delta text.
        delta: String,
        /// Index of the output item.
        output_index: i64,
        /// Index of the content part.
        content_index: i64,
        /// Item ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        item_id: Option<String>,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when refusal output is complete.
    #[serde(rename = "response.refusal.done")]
    RefusalDone {
        /// The full refusal text.
        refusal: String,
        /// Index of the output item.
        output_index: i64,
        /// Index of the content part.
        content_index: i64,
        /// Item ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        item_id: Option<String>,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },

    // -- Function call events --
    /// Emitted for incremental function call arguments.
    #[serde(rename = "response.function_call_arguments.delta")]
    FunctionCallArgumentsDelta {
        /// The arguments delta.
        delta: String,
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when function call arguments are complete.
    #[serde(rename = "response.function_call_arguments.done")]
    FunctionCallArgumentsDone {
        /// The complete arguments JSON string.
        arguments: String,
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// The function name.
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },

    // -- File search events --
    /// Emitted when a file search call begins.
    #[serde(rename = "response.file_search_call.in_progress")]
    FileSearchCallInProgress {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted while a file search call is searching.
    #[serde(rename = "response.file_search_call.searching")]
    FileSearchCallSearching {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when a file search call completes.
    #[serde(rename = "response.file_search_call.completed")]
    FileSearchCallCompleted {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },

    // -- Web search events --
    /// Emitted when a web search call begins.
    #[serde(rename = "response.web_search_call.in_progress")]
    WebSearchCallInProgress {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted while a web search call is searching.
    #[serde(rename = "response.web_search_call.searching")]
    WebSearchCallSearching {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when a web search call completes.
    #[serde(rename = "response.web_search_call.completed")]
    WebSearchCallCompleted {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },

    // -- Reasoning events --
    /// Emitted for incremental reasoning text.
    #[serde(rename = "response.reasoning_text.delta")]
    ReasoningTextDelta {
        /// The reasoning text delta.
        delta: String,
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when reasoning text is complete.
    #[serde(rename = "response.reasoning_text.done")]
    ReasoningTextDone {
        /// The full reasoning text.
        text: String,
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },

    // -- Reasoning summary events --
    /// Emitted when a reasoning summary part is added.
    #[serde(rename = "response.reasoning_summary_part.added")]
    ReasoningSummaryPartAdded {
        /// The summary part.
        part: serde_json::Value,
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Summary index.
        summary_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when a reasoning summary part is completed.
    #[serde(rename = "response.reasoning_summary_part.done")]
    ReasoningSummaryPartDone {
        /// The completed summary part.
        part: serde_json::Value,
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Summary index.
        summary_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted for incremental reasoning summary text.
    #[serde(rename = "response.reasoning_summary_text.delta")]
    ReasoningSummaryTextDelta {
        /// The summary text delta.
        delta: String,
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Summary index.
        summary_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when reasoning summary text is complete.
    #[serde(rename = "response.reasoning_summary_text.done")]
    ReasoningSummaryTextDone {
        /// The full summary text.
        text: String,
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Summary index.
        summary_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },

    // -- Code interpreter events --
    /// Emitted for incremental code interpreter code.
    #[serde(rename = "response.code_interpreter_call_code.delta")]
    CodeInterpreterCallCodeDelta {
        /// The code delta.
        delta: String,
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when code interpreter code is complete.
    #[serde(rename = "response.code_interpreter_call_code.done")]
    CodeInterpreterCallCodeDone {
        /// The full code.
        code: String,
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when a code interpreter call begins.
    #[serde(rename = "response.code_interpreter_call.in_progress")]
    CodeInterpreterCallInProgress {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted while a code interpreter call is interpreting.
    #[serde(rename = "response.code_interpreter_call.interpreting")]
    CodeInterpreterCallInterpreting {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when a code interpreter call completes.
    #[serde(rename = "response.code_interpreter_call.completed")]
    CodeInterpreterCallCompleted {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },

    // -- Image generation events --
    /// Emitted when an image generation call begins.
    #[serde(rename = "response.image_generation_call.in_progress")]
    ImageGenerationCallInProgress {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted while an image is being generated.
    #[serde(rename = "response.image_generation_call.generating")]
    ImageGenerationCallGenerating {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted with partial image data during generation.
    #[serde(rename = "response.image_generation_call.partial_image")]
    ImageGenerationCallPartialImage {
        /// Partial image base64 data.
        partial_image_b64: String,
        /// Partial image index.
        partial_image_index: i64,
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when image generation completes.
    #[serde(rename = "response.image_generation_call.completed")]
    ImageGenerationCallCompleted {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },

    // -- MCP events --
    /// Emitted for incremental MCP call arguments.
    #[serde(rename = "response.mcp_call_arguments.delta")]
    McpCallArgumentsDelta {
        /// The arguments delta.
        delta: String,
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when MCP call arguments are complete.
    #[serde(rename = "response.mcp_call_arguments.done")]
    McpCallArgumentsDone {
        /// The complete arguments.
        arguments: String,
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when an MCP call begins.
    #[serde(rename = "response.mcp_call.in_progress")]
    McpCallInProgress {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when an MCP call completes.
    #[serde(rename = "response.mcp_call.completed")]
    McpCallCompleted {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when an MCP call fails.
    #[serde(rename = "response.mcp_call.failed")]
    McpCallFailed {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when MCP list tools begins.
    #[serde(rename = "response.mcp_list_tools.in_progress")]
    McpListToolsInProgress {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when MCP list tools completes.
    #[serde(rename = "response.mcp_list_tools.completed")]
    McpListToolsCompleted {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when MCP list tools fails.
    #[serde(rename = "response.mcp_list_tools.failed")]
    McpListToolsFailed {
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },

    // -- Audio events --
    /// Emitted for incremental audio data.
    #[serde(rename = "response.audio.delta")]
    AudioDelta {
        /// Base64-encoded audio delta.
        delta: String,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when audio output is complete.
    #[serde(rename = "response.audio.done")]
    AudioDone {
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted for incremental audio transcript text.
    #[serde(rename = "response.audio.transcript.delta")]
    AudioTranscriptDelta {
        /// The transcript delta.
        delta: String,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when audio transcript is complete.
    #[serde(rename = "response.audio.transcript.done")]
    AudioTranscriptDone {
        /// The full transcript.
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },

    // -- Custom tool call events --
    /// Emitted for incremental custom tool call input.
    #[serde(rename = "response.custom_tool_call_input.delta")]
    CustomToolCallInputDelta {
        /// The input delta.
        delta: String,
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
    /// Emitted when custom tool call input is complete.
    #[serde(rename = "response.custom_tool_call_input.done")]
    CustomToolCallInputDone {
        /// The complete input.
        input: String,
        /// Item ID.
        item_id: String,
        /// Index of the output item.
        output_index: i64,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },

    // -- Error event --
    /// Emitted when an error occurs during streaming.
    #[serde(rename = "error")]
    Error {
        /// Error code.
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
        /// Error message.
        message: String,
        /// Parameter that caused the error.
        #[serde(skip_serializing_if = "Option::is_none")]
        param: Option<String>,
        /// Event sequence number.
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<i64>,
    },
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ModelId;

    #[test]
    fn union_helpers_match_response_input_variants() {
        let input = ResponseInput::param_of_text("hello");
        assert_eq!(input.as_text(), Some("hello"));
        assert_eq!(input.as_any(), serde_json::json!("hello"));

        let item = ResponseInputItem::param_of_message(
            InputMessageRole::User,
            InputContent::param_of_text("hi"),
        );
        assert!(item.is_message());
        assert_eq!(item.as_any()["type"], "message");

        let content =
            InputContent::param_of_parts(vec![InputContentPart::param_of_input_text("part")]);
        assert_eq!(content.as_parts().expect("parts").len(), 1);
    }

    #[test]
    fn create_params_serializes_text_input() {
        let params = ResponseCreateParams {
            model: ModelId::from("gpt-4o"),
            input: ResponseInput::Text("Hello, world!".to_owned()),
            background: None,
            context_management: None,
            conversation: None,
            include: None,
            instructions: None,
            max_output_tokens: None,
            max_tool_calls: None,
            metadata: None,
            parallel_tool_calls: None,
            previous_response_id: None,
            prompt: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            reasoning: None,
            safety_identifier: None,
            service_tier: None,
            store: None,
            stream: None,
            stream_options: None,
            temperature: None,
            text: None,
            tool_choice: None,
            tools: None,
            top_logprobs: None,
            top_p: None,
            truncation: None,
            user: None,
            container: None,
        };

        let value = serde_json::to_value(&params).expect("serialize create params");
        assert_eq!(value["model"], "gpt-4o");
        assert_eq!(value["input"], "Hello, world!");
        assert!(value.get("stream").is_none());
        assert!(value.get("temperature").is_none());
        assert!(value.get("tools").is_none());
    }

    #[test]
    fn create_params_serializes_item_input() {
        let params = ResponseCreateParams {
            model: ModelId::from("gpt-4o"),
            input: ResponseInput::Items(vec![ResponseInputItem::Message {
                role: InputMessageRole::User,
                content: InputContent::Text("What is Rust?".to_owned()),
                status: None,
            }]),
            background: None,
            context_management: None,
            conversation: None,
            include: None,
            instructions: None,
            max_output_tokens: None,
            max_tool_calls: None,
            metadata: None,
            parallel_tool_calls: None,
            previous_response_id: None,
            prompt: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            reasoning: None,
            safety_identifier: None,
            service_tier: None,
            store: None,
            stream: None,
            stream_options: None,
            temperature: None,
            text: None,
            tool_choice: None,
            tools: None,
            top_logprobs: None,
            top_p: None,
            truncation: None,
            user: None,
            container: None,
        };

        let value = serde_json::to_value(&params).expect("serialize create params");
        assert!(value["input"].is_array());
        assert_eq!(value["input"][0]["type"], "message");
        assert_eq!(value["input"][0]["role"], "user");
        assert_eq!(value["input"][0]["content"], "What is Rust?");
    }

    #[test]
    fn create_params_omits_optional_fields() {
        let params = ResponseCreateParams {
            model: ModelId::from("gpt-4o-mini"),
            input: ResponseInput::Text("test".to_owned()),
            background: None,
            context_management: None,
            conversation: None,
            include: None,
            instructions: Some("Be helpful".to_owned()),
            max_output_tokens: None,
            max_tool_calls: None,
            metadata: None,
            parallel_tool_calls: None,
            previous_response_id: None,
            prompt: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            reasoning: None,
            safety_identifier: None,
            service_tier: None,
            store: None,
            stream: None,
            stream_options: None,
            temperature: Some(0.7),
            text: None,
            tool_choice: None,
            tools: None,
            top_logprobs: None,
            top_p: None,
            truncation: None,
            user: None,
            container: None,
        };

        let value = serde_json::to_value(&params).expect("serialize create params");
        assert_eq!(value["instructions"], "Be helpful");
        assert_eq!(value["temperature"], 0.7);
        assert!(value.get("max_output_tokens").is_none());
        assert!(value.get("tools").is_none());
        assert!(value.get("background").is_none());
        assert!(value.get("store").is_none());
    }

    #[test]
    fn create_params_with_function_tool() {
        let params = ResponseCreateParams {
            model: ModelId::from("gpt-4o"),
            input: ResponseInput::Text("What's the weather?".to_owned()),
            background: None,
            context_management: None,
            conversation: None,
            include: None,
            instructions: None,
            max_output_tokens: None,
            max_tool_calls: None,
            metadata: None,
            parallel_tool_calls: None,
            previous_response_id: None,
            prompt: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            reasoning: None,
            safety_identifier: None,
            service_tier: None,
            store: None,
            stream: None,
            stream_options: None,
            temperature: None,
            text: None,
            tool_choice: None,
            tools: Some(vec![Tool::Function {
                name: "get_weather".to_owned(),
                description: Some("Get the current weather".to_owned()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    },
                    "required": ["location"]
                })),
                strict: Some(true),
            }]),
            top_logprobs: None,
            top_p: None,
            truncation: None,
            user: None,
            container: None,
        };

        let value = serde_json::to_value(&params).expect("serialize params with tools");
        assert!(value["tools"].is_array());
        assert_eq!(value["tools"][0]["type"], "function");
        assert_eq!(value["tools"][0]["name"], "get_weather");
        assert_eq!(value["tools"][0]["strict"], true);
    }

    #[test]
    fn response_deserializes_from_json() {
        let json = r#"{
            "id": "resp_abc123",
            "object": "response",
            "created_at": 1700000000.0,
            "status": "completed",
            "model": "gpt-4o",
            "output": [
                {
                    "type": "message",
                    "id": "msg_abc123",
                    "role": "assistant",
                    "content": [
                        {
                            "type": "output_text",
                            "text": "Hello! How can I help you?",
                            "annotations": []
                        }
                    ],
                    "status": "completed"
                }
            ],
            "usage": {
                "input_tokens": 10,
                "output_tokens": 8,
                "total_tokens": 18
            }
        }"#;

        let response: Response = serde_json::from_str(json).expect("deserialize response");
        assert_eq!(response.id, "resp_abc123");
        assert_eq!(response.status, ResponseStatus::Completed);
        assert_eq!(response.model.as_ref(), "gpt-4o");
        assert_eq!(response.output.len(), 1);
        assert_eq!(response.output_text(), "Hello! How can I help you?");
        assert!(response.usage.is_some());
        let usage = response.usage.unwrap();
        assert_eq!(usage.input_tokens, 10);
        assert_eq!(usage.output_tokens, 8);
        assert_eq!(usage.total_tokens, 18);
    }

    #[test]
    fn response_with_function_call_output() {
        let json = r#"{
            "id": "resp_xyz789",
            "object": "response",
            "created_at": 1700000000.0,
            "status": "completed",
            "model": "gpt-4o",
            "output": [
                {
                    "type": "function_call",
                    "id": "fc_abc",
                    "call_id": "call_123",
                    "name": "get_weather",
                    "arguments": "{\"location\":\"Paris\"}",
                    "status": "completed"
                }
            ]
        }"#;

        let response: Response = serde_json::from_str(json).expect("deserialize response");
        assert_eq!(response.output.len(), 1);
        match &response.output[0] {
            ResponseOutputItem::FunctionCall {
                name, arguments, ..
            } => {
                assert_eq!(name, "get_weather");
                assert!(arguments.contains("Paris"));
            }
            other => panic!("expected FunctionCall, got {:?}", other),
        }
    }

    #[test]
    fn response_status_serializes_correctly() {
        assert_eq!(
            serde_json::to_string(&ResponseStatus::Completed).unwrap(),
            "\"completed\""
        );
        assert_eq!(
            serde_json::to_string(&ResponseStatus::InProgress).unwrap(),
            "\"in_progress\""
        );
        assert_eq!(
            serde_json::to_string(&ResponseStatus::Cancelled).unwrap(),
            "\"cancelled\""
        );

        let decoded: ResponseStatus =
            serde_json::from_str("\"queued\"").expect("deserialize status");
        assert_eq!(decoded, ResponseStatus::Queued);
    }

    #[test]
    fn tool_choice_mode_serializes_correctly() {
        assert_eq!(
            serde_json::to_string(&ToolChoiceMode::Auto).unwrap(),
            "\"auto\""
        );
        assert_eq!(
            serde_json::to_string(&ToolChoiceMode::Required).unwrap(),
            "\"required\""
        );
    }

    #[test]
    fn tool_choice_function_serializes() {
        let tc = ToolChoice::Function(ToolChoiceFunction {
            choice_type: "function".to_owned(),
            name: "get_weather".to_owned(),
        });
        let value = serde_json::to_value(&tc).expect("serialize tool choice");
        assert_eq!(value["type"], "function");
        assert_eq!(value["name"], "get_weather");
    }

    #[test]
    fn service_tier_serializes_correctly() {
        assert_eq!(
            serde_json::to_string(&ServiceTier::Auto).unwrap(),
            "\"auto\""
        );
        assert_eq!(
            serde_json::to_string(&ServiceTier::Flex).unwrap(),
            "\"flex\""
        );
        assert_eq!(
            serde_json::to_string(&ServiceTier::Priority).unwrap(),
            "\"priority\""
        );
    }

    #[test]
    fn truncation_serializes_correctly() {
        assert_eq!(
            serde_json::to_string(&Truncation::Auto).unwrap(),
            "\"auto\""
        );
        assert_eq!(
            serde_json::to_string(&Truncation::Disabled).unwrap(),
            "\"disabled\""
        );
    }

    #[test]
    fn response_includable_serializes_correctly() {
        assert_eq!(
            serde_json::to_string(&ResponseIncludable::FileSearchCallResults).unwrap(),
            "\"file_search_call.results\""
        );
        assert_eq!(
            serde_json::to_string(&ResponseIncludable::MessageOutputTextLogprobs).unwrap(),
            "\"message.output_text.logprobs\""
        );
    }

    #[test]
    fn prompt_cache_retention_serializes() {
        assert_eq!(
            serde_json::to_string(&PromptCacheRetention::InMemory).unwrap(),
            "\"in-memory\""
        );
        assert_eq!(
            serde_json::to_string(&PromptCacheRetention::TwentyFourHours).unwrap(),
            "\"24h\""
        );
    }

    #[test]
    fn compact_prompt_cache_retention_serializes() {
        assert_eq!(
            serde_json::to_string(&ResponseCompactPromptCacheRetention::InMemory).unwrap(),
            "\"in_memory\""
        );
        assert_eq!(
            serde_json::to_string(&ResponseCompactPromptCacheRetention::TwentyFourHours).unwrap(),
            "\"24h\""
        );

        let params = ResponseCompactParams {
            model: ModelId::from("gpt-4o"),
            instructions: None,
            previous_response_id: Some("resp_previous".to_owned()),
            prompt_cache_key: Some("cache-key".to_owned()),
            input: Some(ResponseInput::Text("Summarize the conversation".to_owned())),
            prompt_cache_retention: Some(ResponseCompactPromptCacheRetention::InMemory),
        };

        let value = serde_json::to_value(params).expect("serialize compact params");
        assert_eq!(value["prompt_cache_retention"], "in_memory");
        assert_eq!(value["prompt_cache_key"], "cache-key");
        assert_eq!(value["previous_response_id"], "resp_previous");
    }

    #[test]
    fn input_item_list_params_serializes() {
        let params = InputItemListParams {
            after: Some("item_abc".to_owned()),
            limit: Some(50),
            include: None,
            order: Some(InputItemListOrder::Asc),
        };

        let qs = serde_urlencoded::to_string(&params).expect("serialize query params");
        assert!(qs.contains("after=item_abc"));
        assert!(qs.contains("limit=50"));
        assert!(qs.contains("order=asc"));
    }

    #[test]
    fn deleted_response_deserializes() {
        let json = r#"{"id":"resp_123","object":"response","deleted":true}"#;
        let deleted: DeletedResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(deleted.id, "resp_123");
        assert!(deleted.deleted);
    }

    #[test]
    fn stream_event_response_created_deserializes() {
        let json = r#"{
            "type": "response.created",
            "response": {
                "id": "resp_abc",
                "object": "response",
                "created_at": 1700000000.0,
                "status": "in_progress",
                "model": "gpt-4o",
                "output": []
            },
            "sequence_number": 0
        }"#;

        let event: ResponseStreamEvent =
            serde_json::from_str(json).expect("deserialize stream event");
        match event {
            ResponseStreamEvent::ResponseCreated {
                response,
                sequence_number,
            } => {
                assert_eq!(response.id, "resp_abc");
                assert_eq!(response.status, ResponseStatus::InProgress);
                assert_eq!(sequence_number, Some(0));
            }
            other => panic!("expected ResponseCreated, got {:?}", other),
        }
    }

    #[test]
    fn stream_event_output_text_delta_deserializes() {
        let json = r#"{
            "type": "response.output_text.delta",
            "delta": "Hello",
            "output_index": 0,
            "content_index": 0,
            "item_id": "msg_123",
            "sequence_number": 5
        }"#;

        let event: ResponseStreamEvent =
            serde_json::from_str(json).expect("deserialize text delta event");
        match event {
            ResponseStreamEvent::OutputTextDelta {
                delta,
                output_index,
                content_index,
                ..
            } => {
                assert_eq!(delta, "Hello");
                assert_eq!(output_index, 0);
                assert_eq!(content_index, 0);
            }
            other => panic!("expected OutputTextDelta, got {:?}", other),
        }
    }

    #[test]
    fn stream_event_function_call_arguments_done_deserializes() {
        let json = r#"{
            "type": "response.function_call_arguments.done",
            "arguments": "{\"location\":\"London\"}",
            "item_id": "fc_abc",
            "output_index": 0,
            "name": "get_weather",
            "sequence_number": 10
        }"#;

        let event: ResponseStreamEvent =
            serde_json::from_str(json).expect("deserialize function call done event");
        match event {
            ResponseStreamEvent::FunctionCallArgumentsDone {
                arguments, name, ..
            } => {
                assert!(arguments.contains("London"));
                assert_eq!(name, Some("get_weather".to_owned()));
            }
            other => panic!("expected FunctionCallArgumentsDone, got {:?}", other),
        }
    }

    #[test]
    fn stream_event_response_completed_deserializes() {
        let json = r#"{
            "type": "response.completed",
            "response": {
                "id": "resp_done",
                "object": "response",
                "created_at": 1700000000.0,
                "status": "completed",
                "model": "gpt-4o",
                "output": [
                    {
                        "type": "message",
                        "id": "msg_1",
                        "role": "assistant",
                        "content": [
                            {"type": "output_text", "text": "Done!", "annotations": []}
                        ],
                        "status": "completed"
                    }
                ],
                "usage": {
                    "input_tokens": 5,
                    "output_tokens": 3,
                    "total_tokens": 8
                }
            },
            "sequence_number": 20
        }"#;

        let event: ResponseStreamEvent =
            serde_json::from_str(json).expect("deserialize completed event");
        match event {
            ResponseStreamEvent::ResponseCompleted { response, .. } => {
                assert_eq!(response.id, "resp_done");
                assert_eq!(response.status, ResponseStatus::Completed);
                assert_eq!(response.output_text(), "Done!");
            }
            other => panic!("expected ResponseCompleted, got {:?}", other),
        }
    }

    #[test]
    fn stream_event_error_deserializes() {
        let json = r#"{
            "type": "error",
            "message": "rate limit exceeded",
            "code": "rate_limit_exceeded",
            "sequence_number": 1
        }"#;

        let event: ResponseStreamEvent =
            serde_json::from_str(json).expect("deserialize error event");
        match event {
            ResponseStreamEvent::Error { message, code, .. } => {
                assert_eq!(message, "rate limit exceeded");
                assert_eq!(code, Some("rate_limit_exceeded".to_owned()));
            }
            other => panic!("expected Error, got {:?}", other),
        }
    }

    #[test]
    fn response_input_text_from_str() {
        let input: ResponseInput = "Hello".into();
        let value = serde_json::to_value(&input).expect("serialize");
        assert_eq!(value, "Hello");
    }

    #[test]
    fn response_input_items_serializes() {
        let input = ResponseInput::Items(vec![
            ResponseInputItem::Message {
                role: InputMessageRole::User,
                content: InputContent::Text("Hi".to_owned()),
                status: None,
            },
            ResponseInputItem::FunctionCallOutput {
                call_id: "call_1".to_owned(),
                output: "result".to_owned(),
            },
        ]);

        let value = serde_json::to_value(&input).expect("serialize");
        assert!(value.is_array());
        assert_eq!(value[0]["type"], "message");
        assert_eq!(value[1]["type"], "function_call_output");
        assert_eq!(value[1]["call_id"], "call_1");
    }

    #[test]
    fn tool_function_serializes_tagged() {
        let tool = Tool::Function {
            name: "calculate".to_owned(),
            description: Some("Perform a calculation".to_owned()),
            parameters: Some(serde_json::json!({"type": "object"})),
            strict: None,
        };

        let value = serde_json::to_value(&tool).expect("serialize tool");
        assert_eq!(value["type"], "function");
        assert_eq!(value["name"], "calculate");
        assert_eq!(value["description"], "Perform a calculation");
    }

    #[test]
    fn tool_web_search_serializes_tagged() {
        let tool = Tool::WebSearch {
            user_location: None,
            search_context_size: Some("medium".to_owned()),
        };

        let value = serde_json::to_value(&tool).expect("serialize tool");
        assert_eq!(value["type"], "web_search_preview");
        assert_eq!(value["search_context_size"], "medium");
    }

    #[test]
    fn output_content_output_text_deserializes() {
        let json = r#"{"type":"output_text","text":"hello","annotations":[]}"#;
        let content: OutputContent = serde_json::from_str(json).expect("deserialize");
        match content {
            OutputContent::OutputText {
                text, annotations, ..
            } => {
                assert_eq!(text, "hello");
                assert!(annotations.is_empty());
            }
            other => panic!("expected OutputText, got {:?}", other),
        }
    }

    #[test]
    fn output_content_refusal_deserializes() {
        let json = r#"{"type":"refusal","refusal":"I cannot help with that"}"#;
        let content: OutputContent = serde_json::from_str(json).expect("deserialize");
        match content {
            OutputContent::Refusal { refusal } => {
                assert_eq!(refusal, "I cannot help with that");
            }
            other => panic!("expected Refusal, got {:?}", other),
        }
    }

    #[test]
    fn reasoning_config_serializes() {
        let config = ReasoningConfig {
            effort: Some(ReasoningEffort::High),
            summary: Some(ReasoningSummary::Auto),
            generate_summary: None,
        };
        let value = serde_json::to_value(&config).expect("serialize");
        assert_eq!(value["effort"], "high");
        assert_eq!(value["summary"], "auto");
        assert!(value.get("generate_summary").is_none());
    }

    #[test]
    fn response_output_item_reasoning_deserializes() {
        let json = r#"{
            "type": "reasoning",
            "id": "rs_123",
            "summary": [
                {"type": "summary_text", "text": "Thinking about the problem..."}
            ]
        }"#;
        let item: ResponseOutputItem = serde_json::from_str(json).expect("deserialize");
        match item {
            ResponseOutputItem::Reasoning { id, summary, .. } => {
                assert_eq!(id, "rs_123");
                assert_eq!(summary.len(), 1);
                assert_eq!(summary[0].text, "Thinking about the problem...");
            }
            other => panic!("expected Reasoning, got {:?}", other),
        }
    }

    #[test]
    fn response_output_item_web_search_deserializes() {
        let json = r#"{"type":"web_search_call","id":"ws_123","status":"completed"}"#;
        let item: ResponseOutputItem = serde_json::from_str(json).expect("deserialize");
        match item {
            ResponseOutputItem::WebSearchCall { id, status, .. } => {
                assert_eq!(id, "ws_123");
                assert_eq!(status, "completed");
            }
            other => panic!("expected WebSearchCall, got {:?}", other),
        }
    }

    #[test]
    fn response_conversation_param_string_serializes() {
        let conv = ConversationParam::Id("conv_abc".to_owned());
        let value = serde_json::to_value(&conv).expect("serialize");
        assert_eq!(value, "conv_abc");
    }

    #[test]
    fn response_conversation_param_object_serializes() {
        let conv = ConversationParam::Object(ResponseConversation {
            id: "conv_abc".to_owned(),
        });
        let value = serde_json::to_value(&conv).expect("serialize");
        assert_eq!(value["id"], "conv_abc");
    }

    #[test]
    fn response_format_json_schema_serializes() {
        let format = ResponseFormat {
            format_type: ResponseFormatType::JsonSchema,
            json_schema: Some(JsonSchemaConfig {
                name: "my_schema".to_owned(),
                schema: Some(serde_json::json!({"type": "object"})),
                strict: Some(true),
                description: None,
            }),
        };
        let value = serde_json::to_value(&format).expect("serialize");
        assert_eq!(value["type"], "json_schema");
        assert_eq!(value["json_schema"]["name"], "my_schema");
        assert_eq!(value["json_schema"]["strict"], true);
    }

    #[test]
    fn input_content_part_text_serializes() {
        let part = InputContentPart::InputText {
            text: "hello".to_owned(),
        };
        let value = serde_json::to_value(&part).expect("serialize");
        assert_eq!(value["type"], "input_text");
        assert_eq!(value["text"], "hello");
    }

    #[test]
    fn input_content_part_file_serializes_detail() {
        let part = InputContentPart::InputFile {
            file_id: Some("file_abc".to_owned()),
            filename: None,
            detail: Some(ResponseInputFileDetail::High),
            file_data: None,
        };
        let value = serde_json::to_value(&part).expect("serialize input file");
        assert_eq!(value["type"], "input_file");
        assert_eq!(value["file_id"], "file_abc");
        assert_eq!(value["detail"], "high");
    }

    #[test]
    fn annotation_url_citation_serializes() {
        let annotation = Annotation::UrlCitation {
            url: "https://example.com".to_owned(),
            title: Some("Example".to_owned()),
            start_index: Some(0),
            end_index: Some(10),
            snippet: None,
            source: None,
        };
        let value = serde_json::to_value(&annotation).expect("serialize");
        assert_eq!(value["type"], "url_citation");
        assert_eq!(value["url"], "https://example.com");
    }

    #[test]
    fn response_error_deserializes() {
        let json = r#"{"code":"server_error","message":"Internal error"}"#;
        let err: ResponseError = serde_json::from_str(json).expect("deserialize");
        assert_eq!(err.code, Some("server_error".to_owned()));
        assert_eq!(err.message, Some("Internal error".to_owned()));
    }

    #[test]
    fn usage_with_details_deserializes() {
        let json = r#"{
            "input_tokens": 100,
            "input_tokens_details": {"cached_tokens": 50},
            "output_tokens": 200,
            "output_tokens_details": {"reasoning_tokens": 30},
            "total_tokens": 300
        }"#;
        let usage: ResponseUsage = serde_json::from_str(json).expect("deserialize");
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 200);
        assert_eq!(usage.total_tokens, 300);
        assert_eq!(usage.input_tokens_details.unwrap().cached_tokens, 50);
        assert_eq!(usage.output_tokens_details.unwrap().reasoning_tokens, 30);
    }

    #[test]
    fn output_text_extracts_text_across_items() {
        let response = Response {
            id: "resp_1".to_owned(),
            object: "response".to_owned(),
            created_at: 0.0,
            status: ResponseStatus::Completed,
            error: None,
            incomplete_details: None,
            instructions: None,
            model: ModelId::from("gpt-4o"),
            output: vec![ResponseOutputItem::Message {
                id: "msg_1".to_owned(),
                role: "assistant".to_owned(),
                content: vec![
                    OutputContent::OutputText {
                        text: "Hello ".to_owned(),
                        annotations: vec![],
                        logprobs: None,
                    },
                    OutputContent::OutputText {
                        text: "world!".to_owned(),
                        annotations: vec![],
                        logprobs: None,
                    },
                ],
                status: OutputMessageStatus::Completed,
                phase: None,
            }],
            parallel_tool_calls: None,
            temperature: None,
            tool_choice: None,
            tools: None,
            top_p: None,
            background: None,
            completed_at: None,
            conversation: None,
            max_output_tokens: None,
            max_tool_calls: None,
            metadata: None,
            previous_response_id: None,
            prompt: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            reasoning: None,
            safety_identifier: None,
            service_tier: None,
            text: None,
            top_logprobs: None,
            truncation: None,
            usage: None,
            user: None,
        };

        assert_eq!(response.output_text(), "Hello world!");
    }

    #[test]
    fn stream_event_web_search_completed_deserializes() {
        let json = r#"{
            "type": "response.web_search_call.completed",
            "item_id": "ws_abc",
            "output_index": 1,
            "sequence_number": 15
        }"#;

        let event: ResponseStreamEvent =
            serde_json::from_str(json).expect("deserialize web search event");
        match event {
            ResponseStreamEvent::WebSearchCallCompleted {
                item_id,
                output_index,
                ..
            } => {
                assert_eq!(item_id, "ws_abc");
                assert_eq!(output_index, 1);
            }
            other => panic!("expected WebSearchCallCompleted, got {:?}", other),
        }
    }

    #[test]
    fn stream_event_reasoning_text_delta_deserializes() {
        let json = r#"{
            "type": "response.reasoning_text.delta",
            "delta": "Let me think...",
            "item_id": "rs_1",
            "output_index": 0,
            "sequence_number": 3
        }"#;

        let event: ResponseStreamEvent =
            serde_json::from_str(json).expect("deserialize reasoning delta");
        match event {
            ResponseStreamEvent::ReasoningTextDelta { delta, item_id, .. } => {
                assert_eq!(delta, "Let me think...");
                assert_eq!(item_id, "rs_1");
            }
            other => panic!("expected ReasoningTextDelta, got {:?}", other),
        }
    }

    #[test]
    fn create_params_with_multiple_tools_and_options() {
        let params = ResponseCreateParams {
            model: ModelId::from("gpt-4o"),
            input: ResponseInput::Text("Search for Rust docs".to_owned()),
            background: Some(true),
            context_management: None,
            conversation: None,
            include: Some(vec![ResponseIncludable::WebSearchCallActionSources]),
            instructions: Some("You are a helpful assistant.".to_owned()),
            max_output_tokens: Some(4096),
            max_tool_calls: Some(10),
            metadata: Some({
                let mut m = HashMap::new();
                m.insert("session".to_owned(), "test_123".to_owned());
                m
            }),
            parallel_tool_calls: Some(true),
            previous_response_id: None,
            prompt: None,
            prompt_cache_key: Some("cache-key-1".to_owned()),
            prompt_cache_retention: Some(PromptCacheRetention::TwentyFourHours),
            reasoning: Some(ReasoningConfig {
                effort: Some(ReasoningEffort::Medium),
                summary: None,
                generate_summary: None,
            }),
            safety_identifier: Some("user-hash-abc".to_owned()),
            service_tier: Some(ServiceTier::Auto),
            store: Some(true),
            stream: None,
            stream_options: None,
            temperature: Some(0.5),
            text: Some(TextConfig {
                format: None,
                verbosity: Some(TextVerbosity::Low),
            }),
            tool_choice: Some(ToolChoice::Mode(ToolChoiceMode::Auto)),
            tools: Some(vec![
                Tool::WebSearch {
                    user_location: None,
                    search_context_size: Some("medium".to_owned()),
                },
                Tool::Function {
                    name: "lookup".to_owned(),
                    description: Some("Look up info".to_owned()),
                    parameters: None,
                    strict: None,
                },
            ]),
            top_logprobs: None,
            top_p: None,
            truncation: Some(Truncation::Auto),
            user: None,
            container: None,
        };

        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["background"], true);
        assert_eq!(value["max_output_tokens"], 4096);
        assert_eq!(value["max_tool_calls"], 10);
        assert_eq!(value["parallel_tool_calls"], true);
        assert_eq!(value["service_tier"], "auto");
        assert_eq!(value["store"], true);
        assert_eq!(value["temperature"], 0.5);
        assert_eq!(value["truncation"], "auto");
        assert_eq!(value["prompt_cache_key"], "cache-key-1");
        assert_eq!(value["prompt_cache_retention"], "24h");
        assert_eq!(value["safety_identifier"], "user-hash-abc");
        assert_eq!(value["reasoning"]["effort"], "medium");
        assert_eq!(value["text"]["verbosity"], "low");
        assert_eq!(value["tool_choice"], "auto");
        assert_eq!(value["tools"].as_array().unwrap().len(), 2);
        assert_eq!(value["include"][0], "web_search_call.action.sources");
        assert_eq!(value["metadata"]["session"], "test_123");
    }
}
