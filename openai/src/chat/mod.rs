//! Chat APIs.

use std::collections::HashMap;

use futures::Stream;

use crate::{
    pagination::CursorPage,
    shared::{FinishReason, ModelId},
    ssestream::SseStream,
    Client, Result,
};

// ---------------------------------------------------------------------------
// Services
// ---------------------------------------------------------------------------

/// Chat service root.
#[derive(Clone)]
pub struct ChatService {
    client: Client,
}

impl ChatService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Returns chat completions service.
    #[must_use]
    pub fn completions(&self) -> ChatCompletionsService {
        ChatCompletionsService {
            client: self.client.clone(),
        }
    }
}

/// Chat completion endpoints.
#[derive(Clone)]
pub struct ChatCompletionsService {
    client: Client,
}

impl ChatCompletionsService {
    /// Creates a chat completion.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: ChatCompletionCreateParams) -> Result<ChatCompletion> {
        self.client.post_json("/chat/completions", &params).await
    }

    /// Creates a streaming chat completion.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or stream decode failures.
    pub async fn create_stream(
        &self,
        mut params: ChatCompletionCreateParams,
    ) -> Result<impl Stream<Item = Result<ChatCompletionChunk>>> {
        params.stream = Some(true);
        let response = self
            .client
            .post_raw_json("/chat/completions", &params)
            .await?;
        Ok(SseStream::new(response))
    }

    /// Gets a stored chat completion by ID.
    ///
    /// Only completions created with `store: true` will be returned.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, completion_id: impl AsRef<str>) -> Result<ChatCompletion> {
        self.client
            .get_json(&format!(
                "/chat/completions/{}",
                urlencoding::encode(completion_id.as_ref())
            ))
            .await
    }

    /// Updates a stored chat completion (currently only metadata).
    ///
    /// Only completions created with `store: true` can be modified.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn update(
        &self,
        completion_id: impl AsRef<str>,
        params: ChatCompletionUpdateParams,
    ) -> Result<ChatCompletion> {
        self.client
            .post_json(
                &format!(
                    "/chat/completions/{}",
                    urlencoding::encode(completion_id.as_ref())
                ),
                &params,
            )
            .await
    }

    /// Deletes a stored chat completion.
    ///
    /// Only completions created with `store: true` can be deleted.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(&self, completion_id: impl AsRef<str>) -> Result<ChatCompletionDeleted> {
        self.client
            .delete_json(&format!(
                "/chat/completions/{}",
                urlencoding::encode(completion_id.as_ref())
            ))
            .await
    }

    /// Lists stored chat completions.
    ///
    /// Only completions created with `store: true` will be returned.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(
        &self,
        params: ChatCompletionListParams,
    ) -> Result<CursorPage<ChatCompletion>> {
        self.client
            .get_cursor_page_query("/chat/completions", &params)
            .await
    }

    /// Returns a sub-service for accessing messages of stored completions.
    #[must_use]
    pub fn messages(&self) -> ChatCompletionMessageService {
        ChatCompletionMessageService {
            client: self.client.clone(),
        }
    }
}

/// Service for listing messages within a stored chat completion.
#[derive(Clone)]
pub struct ChatCompletionMessageService {
    client: Client,
}

impl ChatCompletionMessageService {
    /// Lists messages in a stored chat completion.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(
        &self,
        completion_id: impl AsRef<str>,
        params: ChatCompletionMessageListParams,
    ) -> Result<CursorPage<ChatCompletionStoreMessage>> {
        let path = format!(
            "/chat/completions/{}/messages",
            urlencoding::encode(completion_id.as_ref())
        );
        self.client.get_cursor_page_query(&path, &params).await
    }
}

// ---------------------------------------------------------------------------
// Typed enums
// ---------------------------------------------------------------------------

/// Service tier for processing chat completions.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatCompletionServiceTier {
    /// Automatic selection based on project settings.
    Auto,
    /// Standard pricing and performance.
    Default,
    /// Flex processing tier.
    Flex,
    /// Scale processing tier.
    Scale,
    /// Priority processing tier.
    Priority,
}

/// Reasoning effort level for reasoning models.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatCompletionReasoningEffort {
    /// No reasoning (gpt-5.1 default).
    None,
    /// Minimal reasoning effort.
    Minimal,
    /// Low reasoning effort.
    Low,
    /// Medium reasoning effort (default for models before gpt-5.1).
    Medium,
    /// High reasoning effort.
    High,
    /// Extra-high reasoning effort.
    Xhigh,
}

/// Prompt cache retention policy.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ChatCompletionPromptCacheRetention {
    /// In-memory caching.
    #[serde(rename = "in-memory")]
    InMemory,
    /// 24-hour extended caching.
    #[serde(rename = "24h")]
    TwentyFourHours,
}

/// Verbosity level for model responses.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatCompletionVerbosity {
    /// Concise responses.
    Low,
    /// Standard verbosity.
    Medium,
    /// Verbose responses.
    High,
}

// ---------------------------------------------------------------------------
// Audio types
// ---------------------------------------------------------------------------

/// Audio output format for chat completions.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatCompletionAudioFormat {
    /// WAV format.
    Wav,
    /// AAC format.
    Aac,
    /// MP3 format.
    Mp3,
    /// FLAC format.
    Flac,
    /// Opus format.
    Opus,
    /// PCM 16-bit format.
    Pcm16,
}

/// Built-in voice for audio output.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatCompletionAudioVoiceString {
    /// Alloy voice.
    Alloy,
    /// Ash voice.
    Ash,
    /// Ballad voice.
    Ballad,
    /// Coral voice.
    Coral,
    /// Echo voice.
    Echo,
    /// Sage voice.
    Sage,
    /// Shimmer voice.
    Shimmer,
    /// Verse voice.
    Verse,
    /// Marin voice.
    Marin,
    /// Cedar voice.
    Cedar,
}

/// Custom voice reference by ID.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionAudioVoiceID {
    /// The custom voice ID, e.g. `voice_1234`.
    pub id: String,
}

/// Voice union: either a built-in voice name or a custom voice ID.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionAudioVoice {
    /// A built-in voice string.
    BuiltIn(ChatCompletionAudioVoiceString),
    /// A custom voice ID object.
    Custom(ChatCompletionAudioVoiceID),
    /// An arbitrary voice string.
    Other(String),
}

/// Parameters for audio output in chat completions.
/// Required when audio output is requested with `modalities: ["audio"]`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionAudioParam {
    /// Specifies the output audio format.
    pub format: ChatCompletionAudioFormat,
    /// The voice the model uses to respond.
    pub voice: ChatCompletionAudioVoice,
}

/// Audio response data from the model.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionAudio {
    /// Unique identifier for this audio response.
    pub id: String,
    /// Base64 encoded audio bytes generated by the model.
    pub data: String,
    /// Unix timestamp when this audio response expires.
    pub expires_at: i64,
    /// Transcript of the audio generated by the model.
    pub transcript: String,
}

// ---------------------------------------------------------------------------
// Web search types
// ---------------------------------------------------------------------------

/// Approximate user location for web search.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct WebSearchUserLocationApproximate {
    /// Free text city, e.g. "San Francisco".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Two-letter ISO country code, e.g. "US".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// Free text region, e.g. "California".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// IANA timezone, e.g. "America/Los_Angeles".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

/// User location for web search, currently only supports approximate.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebSearchUserLocation {
    /// Approximate location parameters.
    pub approximate: WebSearchUserLocationApproximate,
    /// The type of location approximation, always "approximate".
    #[serde(rename = "type", default = "default_approximate_type")]
    pub location_type: String,
}

fn default_approximate_type() -> String {
    "approximate".to_owned()
}

/// Web search options for chat completions.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct WebSearchOptions {
    /// Approximate location parameters for the search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<WebSearchUserLocation>,
    /// Context window size guidance: "low", "medium", or "high".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_context_size: Option<String>,
}

/// A URL citation annotation from web search results.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionMessageAnnotationURLCitation {
    /// Index of the last character of the citation in the message.
    pub end_index: i64,
    /// Index of the first character of the citation in the message.
    pub start_index: i64,
    /// Title of the web resource.
    pub title: String,
    /// URL of the web resource.
    pub url: String,
}

/// Annotation on a chat completion message (e.g. URL citation from web search).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionMessageAnnotation {
    /// The type of annotation, always "url_citation".
    #[serde(rename = "type")]
    pub annotation_type: String,
    /// The URL citation data.
    pub url_citation: ChatCompletionMessageAnnotationURLCitation,
}

// ---------------------------------------------------------------------------
// Prediction types
// ---------------------------------------------------------------------------

/// Content union for prediction: either a string or array of text parts.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionPredictionContent {
    /// A plain text string.
    Text(String),
    /// An array of text content parts.
    Parts(Vec<ChatCompletionContentPartTextParam>),
}

/// Static predicted output content for faster generation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionPredictionContentParam {
    /// The content that should be matched when generating a model response.
    pub content: ChatCompletionPredictionContent,
    /// The type of predicted content, always "content".
    #[serde(rename = "type", default = "default_content_type")]
    pub prediction_type: String,
}

fn default_content_type() -> String {
    "content".to_owned()
}

// ---------------------------------------------------------------------------
// Custom tool types
// ---------------------------------------------------------------------------

/// Grammar syntax for custom tools.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GrammarSyntax {
    /// Lark grammar syntax.
    Lark,
    /// Regex grammar syntax.
    Regex,
}

/// Grammar definition for custom tool input format.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomToolGrammar {
    /// The grammar definition string.
    pub definition: String,
    /// The syntax of the grammar definition.
    pub syntax: GrammarSyntax,
}

/// Grammar format for custom tool input.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomToolFormatGrammar {
    /// The grammar specification.
    pub grammar: CustomToolGrammar,
    /// Format type, always "grammar".
    #[serde(rename = "type", default = "default_grammar_type")]
    pub format_type: String,
}

fn default_grammar_type() -> String {
    "grammar".to_owned()
}

/// Text format for custom tool input (unconstrained).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomToolFormatText {
    /// Format type, always "text".
    #[serde(rename = "type", default = "default_text_type")]
    pub format_type: String,
}

fn default_text_type() -> String {
    "text".to_owned()
}

/// Input format union for custom tools: text or grammar.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CustomToolFormat {
    /// Unconstrained free-form text.
    Text,
    /// Grammar-constrained input.
    Grammar {
        /// The grammar specification.
        grammar: CustomToolGrammar,
    },
}

/// Properties of a custom tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionCustomToolCustomParam {
    /// The name of the custom tool.
    pub name: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The input format for the custom tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<CustomToolFormat>,
}

/// A custom tool that processes input using a specified format.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionCustomToolParam {
    /// Properties of the custom tool.
    pub custom: ChatCompletionCustomToolCustomParam,
    /// The type of the tool, always "custom".
    #[serde(rename = "type", default = "default_custom_type")]
    pub tool_type: String,
}

fn default_custom_type() -> String {
    "custom".to_owned()
}

/// A custom tool call from the model response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionMessageCustomToolCallCustom {
    /// The input for the custom tool call generated by the model.
    pub input: String,
    /// The name of the custom tool to call.
    pub name: String,
}

/// A custom tool call in the assistant's response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionMessageCustomToolCall {
    /// Unique tool call ID.
    pub id: String,
    /// The custom tool details.
    pub custom: ChatCompletionMessageCustomToolCallCustom,
    /// Type of the tool call, always "custom".
    #[serde(rename = "type")]
    pub call_type: String,
}

/// Allowed tools mode for tool choice.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatCompletionAllowedToolsMode {
    /// Model can pick among allowed tools or generate a message.
    Auto,
    /// Model must call one or more allowed tools.
    Required,
}

/// Configuration for constraining available tools.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionAllowedToolsParam {
    /// The mode: auto or required.
    pub mode: ChatCompletionAllowedToolsMode,
    /// A list of tool definitions the model is allowed to call.
    pub tools: Vec<serde_json::Value>,
}

/// Allowed tool choice option.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionAllowedToolChoiceParam {
    /// The allowed tools configuration.
    pub allowed_tools: ChatCompletionAllowedToolsParam,
    /// Type, always "allowed_tools".
    #[serde(rename = "type", default = "default_allowed_tools_type")]
    pub choice_type: String,
}

fn default_allowed_tools_type() -> String {
    "allowed_tools".to_owned()
}

// ---------------------------------------------------------------------------
// File content part
// ---------------------------------------------------------------------------

/// File parameters for file content parts.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionContentPartFileData {
    /// Base64 encoded file data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data: Option<String>,
    /// The ID of an uploaded file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    /// The name of the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

/// File content part parameter.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionContentPartFileParam {
    /// The type of content part, always "file".
    #[serde(rename = "type")]
    pub part_type: String,
    /// File data.
    pub file: ChatCompletionContentPartFileData,
}

impl ChatCompletionContentPartFileParam {
    /// Creates a new file content part from a file ID.
    #[must_use]
    pub fn from_file_id(file_id: impl Into<String>) -> Self {
        Self {
            part_type: "file".to_owned(),
            file: ChatCompletionContentPartFileData {
                file_data: None,
                file_id: Some(file_id.into()),
                filename: None,
            },
        }
    }

    /// Creates a new file content part from base64 data.
    #[must_use]
    pub fn from_data(data: impl Into<String>, filename: impl Into<String>) -> Self {
        Self {
            part_type: "file".to_owned(),
            file: ChatCompletionContentPartFileData {
                file_data: Some(data.into()),
                file_id: None,
                filename: Some(filename.into()),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Roles
// ---------------------------------------------------------------------------

/// Role-tagged chat message.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    /// Developer instruction role (newer models).
    Developer,
    /// System instruction role.
    System,
    /// End-user input role.
    User,
    /// Assistant output role.
    Assistant,
    /// Tool response role.
    Tool,
    /// Legacy function response role.
    Function,
}

// ---------------------------------------------------------------------------
// Content parts (vision, audio, file)
// ---------------------------------------------------------------------------

/// Image detail level for vision inputs.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    /// Automatic detail level.
    Auto,
    /// Low detail level.
    Low,
    /// High detail level.
    High,
}

/// Image URL for vision content parts.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionContentPartImageUrl {
    /// Either a URL of the image or base64-encoded image data.
    pub url: String,
    /// Detail level of the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<ImageDetail>,
}

/// Image content part for vision inputs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionContentPartImageParam {
    /// The type of the content part, always `"image_url"`.
    #[serde(rename = "type")]
    pub part_type: String,
    /// Image URL descriptor.
    pub image_url: ChatCompletionContentPartImageUrl,
}

impl ChatCompletionContentPartImageParam {
    /// Creates a new image content part.
    #[must_use]
    pub fn new(url: impl Into<String>, detail: Option<ImageDetail>) -> Self {
        Self {
            part_type: "image_url".to_owned(),
            image_url: ChatCompletionContentPartImageUrl {
                url: url.into(),
                detail,
            },
        }
    }
}

/// Input audio data for audio content parts.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionContentPartInputAudioData {
    /// Base64-encoded audio data.
    pub data: String,
    /// Audio format ("wav" or "mp3").
    pub format: String,
}

/// Audio content part for audio inputs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionContentPartInputAudioParam {
    /// The type of the content part, always `"input_audio"`.
    #[serde(rename = "type")]
    pub part_type: String,
    /// Audio data.
    pub input_audio: ChatCompletionContentPartInputAudioData,
}

impl ChatCompletionContentPartInputAudioParam {
    /// Creates a new audio input content part.
    #[must_use]
    pub fn new(data: impl Into<String>, format: impl Into<String>) -> Self {
        Self {
            part_type: "input_audio".to_owned(),
            input_audio: ChatCompletionContentPartInputAudioData {
                data: data.into(),
                format: format.into(),
            },
        }
    }
}

/// Text content part.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionContentPartTextParam {
    /// The type of the content part, always `"text"`.
    #[serde(rename = "type")]
    pub part_type: String,
    /// Text content.
    pub text: String,
}

impl ChatCompletionContentPartTextParam {
    /// Creates a new text content part.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            part_type: "text".to_owned(),
            text: text.into(),
        }
    }
}

/// Union of content part types that can appear in a user message.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatCompletionContentPart {
    /// Text content.
    Text {
        /// Text content.
        text: String,
    },
    /// Image URL content (vision).
    ImageUrl {
        /// Image URL descriptor.
        image_url: ChatCompletionContentPartImageUrl,
    },
    /// Audio input content.
    InputAudio {
        /// Audio data.
        input_audio: ChatCompletionContentPartInputAudioData,
    },
    /// File content.
    File {
        /// File data.
        file: ChatCompletionContentPartFileData,
    },
}

// ---------------------------------------------------------------------------
// User message content union (string or parts array)
// ---------------------------------------------------------------------------

/// User message content: either a plain string or an array of content parts.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionUserMessageContent {
    /// Plain text string content.
    Text(String),
    /// Array of content parts (text, image_url, input_audio).
    Parts(Vec<ChatCompletionContentPart>),
}

// ---------------------------------------------------------------------------
// Tool calling types
// ---------------------------------------------------------------------------

/// Function definition for a callable tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionDefinition {
    /// Function name.
    pub name: String,
    /// Description of what the function does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema object describing the function's parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
    /// Whether to enable strict schema adherence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

/// A tool the model may call. Currently only function tools are supported.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionTool {
    /// Tool type, always `"function"`.
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function definition.
    pub function: FunctionDefinition,
}

impl ChatCompletionTool {
    /// Creates a new function tool.
    #[must_use]
    pub fn function(definition: FunctionDefinition) -> Self {
        Self {
            tool_type: "function".to_owned(),
            function: definition,
        }
    }
}

/// A specific named function for tool_choice.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionNamedToolChoiceFunction {
    /// The function name.
    pub name: String,
}

/// Specifies a tool the model should use.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionNamedToolChoice {
    /// Type of the tool choice, always `"function"`.
    #[serde(rename = "type")]
    pub choice_type: String,
    /// Function to call.
    pub function: ChatCompletionNamedToolChoiceFunction,
}

/// Controls which tool is called by the model.
///
/// - `"none"`: model will not call any tool
/// - `"auto"`: model can pick between generating a message or calling tools
/// - `"required"`: model must call one or more tools
/// - Specific function: forces a particular function call
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionToolChoiceOption {
    /// String mode: "none", "auto", or "required".
    Mode(String),
    /// Specific named tool choice.
    Named(ChatCompletionNamedToolChoice),
    /// Constrained allowed tools choice.
    AllowedTools(ChatCompletionAllowedToolChoiceParam),
}

/// A function call made by the model (response side).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionMessageToolCallFunction {
    /// The function name.
    pub name: String,
    /// JSON arguments string generated by the model.
    pub arguments: String,
}

/// A tool call in the assistant's response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionMessageToolCall {
    /// Unique tool call ID.
    pub id: String,
    /// Type of the tool call, always `"function"`.
    #[serde(rename = "type")]
    pub call_type: String,
    /// The function call details.
    pub function: ChatCompletionMessageToolCallFunction,
}

/// Legacy function_call field in the assistant response (deprecated).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionMessageFunctionCall {
    /// The function name.
    pub name: String,
    /// JSON arguments string generated by the model.
    pub arguments: String,
}

// ---------------------------------------------------------------------------
// Structured outputs / response format
// ---------------------------------------------------------------------------

/// JSON Schema definition for structured outputs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseFormatJsonSchemaDefinition {
    /// Schema name.
    pub name: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    /// Whether to enforce strict schema adherence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

/// Response format options for chat completions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    /// Plain text output (default).
    Text,
    /// JSON object output mode.
    JsonObject,
    /// Structured outputs with a JSON schema.
    JsonSchema {
        /// The JSON schema definition.
        json_schema: ResponseFormatJsonSchemaDefinition,
    },
}

// ---------------------------------------------------------------------------
// Message param types (request side)
// ---------------------------------------------------------------------------

/// System message parameter.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionSystemMessageParam {
    /// Role, always `"system"`.
    pub role: ChatRole,
    /// Message content.
    pub content: String,
    /// Optional participant name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Developer message parameter (newer models, replaces system).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionDeveloperMessageParam {
    /// Role, always `"developer"`.
    pub role: ChatRole,
    /// Message content.
    pub content: String,
    /// Optional participant name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// User message parameter, with text or multimodal content.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionUserMessageParam {
    /// Role, always `"user"`.
    pub role: ChatRole,
    /// Message content (plain text string or array of content parts).
    pub content: ChatCompletionUserMessageContent,
    /// Optional participant name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Assistant message parameter, with optional tool_calls.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionAssistantMessageParam {
    /// Role, always `"assistant"`.
    pub role: ChatRole,
    /// Message content (may be absent when tool_calls is present).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Optional participant name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Refusal message from the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    /// Tool calls generated by the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ChatCompletionMessageToolCall>>,
    /// Legacy function_call field (deprecated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<ChatCompletionMessageFunctionCall>,
}

/// Tool message parameter (response to a tool call).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionToolMessageParam {
    /// Role, always `"tool"`.
    pub role: ChatRole,
    /// The tool response content.
    pub content: String,
    /// The tool call ID this message responds to.
    pub tool_call_id: String,
}

/// Legacy function message parameter (deprecated).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionFunctionMessageParam {
    /// Role, always `"function"`.
    pub role: ChatRole,
    /// Function response content.
    pub content: String,
    /// The function name.
    pub name: String,
}

/// Union of all chat completion message types for request payloads.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum ChatCompletionMessageParam {
    /// Developer instruction message.
    Developer {
        /// Message content.
        content: String,
        /// Optional participant name.
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    /// System instruction message.
    System {
        /// Message content.
        content: String,
        /// Optional participant name.
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    /// User input message (text or multimodal).
    User {
        /// Message content (string or parts array).
        content: ChatCompletionUserMessageContent,
        /// Optional participant name.
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    /// Assistant output message.
    Assistant {
        /// Message content (may be absent when tool_calls is present).
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        /// Optional participant name.
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        /// Refusal message from the model.
        #[serde(skip_serializing_if = "Option::is_none")]
        refusal: Option<String>,
        /// Tool calls generated by the model.
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ChatCompletionMessageToolCall>>,
        /// Legacy function_call field (deprecated).
        #[serde(skip_serializing_if = "Option::is_none")]
        function_call: Option<ChatCompletionMessageFunctionCall>,
    },
    /// Tool response message.
    Tool {
        /// Tool response content.
        content: String,
        /// The tool call ID this responds to.
        tool_call_id: String,
    },
    /// Legacy function response message (deprecated).
    Function {
        /// Function response content.
        content: String,
        /// Function name.
        name: String,
    },
}

// ---------------------------------------------------------------------------
// Log probabilities
// ---------------------------------------------------------------------------

/// Top log probability entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TopLogprob {
    /// The token.
    pub token: String,
    /// Log probability of this token.
    pub logprob: f64,
    /// UTF-8 byte representation of the token; null if unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<i64>>,
}

/// Token log probability information.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionTokenLogprob {
    /// The token.
    pub token: String,
    /// Log probability of this token.
    pub logprob: f64,
    /// UTF-8 byte representation of the token; null if unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<i64>>,
    /// Most likely tokens at this position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<Vec<TopLogprob>>,
}

/// Log probability information for a choice.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionChoiceLogprobs {
    /// Content token log probabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ChatCompletionTokenLogprob>>,
    /// Refusal token log probabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<Vec<ChatCompletionTokenLogprob>>,
}

// ---------------------------------------------------------------------------
// Usage types
// ---------------------------------------------------------------------------

/// Breakdown of completion tokens.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompletionTokensDetails {
    /// Tokens for reasoning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,
    /// Audio output tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
    /// Accepted prediction tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_prediction_tokens: Option<u32>,
    /// Rejected prediction tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_prediction_tokens: Option<u32>,
}

/// Breakdown of prompt tokens.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptTokensDetails {
    /// Audio input tokens in the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
    /// Cached tokens in the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u32>,
}

/// Detailed token usage for chat completion responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompletionUsage {
    /// Prompt tokens billed.
    pub prompt_tokens: u32,
    /// Completion tokens billed.
    pub completion_tokens: u32,
    /// Total token count.
    pub total_tokens: u32,
    /// Detailed breakdown of completion tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
    /// Detailed breakdown of prompt tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
}

// ---------------------------------------------------------------------------
// Response message (returned by the API)
// ---------------------------------------------------------------------------

/// A chat completion message generated by the model (response side).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionMessage {
    /// The role of the author, typically `"assistant"`.
    pub role: ChatRole,
    /// The message content, may be null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Refusal message from the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    /// Tool calls generated by the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ChatCompletionMessageToolCall>>,
    /// Legacy function_call field (deprecated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<ChatCompletionMessageFunctionCall>,
    /// Audio response from the model (when audio output modality is requested).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<ChatCompletionAudio>,
    /// Annotations for the message (e.g. URL citations from web search).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<ChatCompletionMessageAnnotation>>,
}

// ---------------------------------------------------------------------------
// Request payload
// ---------------------------------------------------------------------------

/// Request payload for chat completion creation.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionCreateParams {
    /// Model ID.
    pub model: ModelId,
    /// Message list.
    pub messages: Vec<ChatCompletionMessageParam>,
    /// Enables SSE streaming mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Optional temperature value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Frequency penalty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    /// Presence penalty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    /// Maximum number of completion tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<i64>,
    /// Maximum number of tokens (deprecated, use max_completion_tokens).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i64>,
    /// How many choices to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,
    /// Top-p nucleus sampling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Seed for deterministic sampling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Whether to store the completion for distillation/evals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    /// Whether to return log probabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    /// Number of most likely tokens to return at each position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<i64>,
    /// Up to 4 stop sequences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<serde_json::Value>,
    /// Response format (text, json_object, or json_schema).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    /// Tools the model may call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ChatCompletionTool>>,
    /// Controls which tool is called.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ChatCompletionToolChoiceOption>,
    /// Whether to enable parallel function calling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    /// End-user identifier (deprecated, use safety_identifier + prompt_cache_key).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Service tier for processing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ChatCompletionServiceTier>,
    /// Metadata key-value pairs (up to 16).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Reasoning effort for reasoning models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ChatCompletionReasoningEffort>,
    /// Stream options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<ChatCompletionStreamOptions>,
    /// Parameters for audio output (required with `modalities: ["audio"]`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<ChatCompletionAudioParam>,
    /// Modify likelihood of specified tokens. Maps token IDs to bias values (-100 to 100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i64>>,
    /// Output modalities: "text", "audio", or both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<String>>,
    /// Stable key for prompt caching.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    /// Prompt cache retention policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<ChatCompletionPromptCacheRetention>,
    /// Stable identifier for detecting abusive users.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,
    /// Verbosity level for model responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<ChatCompletionVerbosity>,
    /// Static predicted output content for faster generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<ChatCompletionPredictionContentParam>,
    /// Web search tool options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search_options: Option<WebSearchOptions>,
    /// Deprecated: Controls which (if any) function is called by the model.
    /// `"none"` means the model will not call a function. `"auto"` means the
    /// model can pick between generating a message or calling a function.
    /// Specifying `{ "name": "my_function" }` forces the model to call that function.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated(note = "Use tools and tool_choice instead")]
    pub function_call: Option<serde_json::Value>,
    /// Deprecated: A list of functions the model may generate JSON inputs for.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated(note = "Use tools instead")]
    pub functions: Option<Vec<FunctionDefinition>>,
}

/// Options for streaming responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionStreamOptions {
    /// Whether to include usage in the final chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>,
}

/// Parameters for updating a stored chat completion.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionUpdateParams {
    /// Metadata key-value pairs.
    pub metadata: HashMap<String, String>,
}

/// Parameters for listing stored chat completions.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionListParams {
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Maximum number of items to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Filter by model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Sort order ("asc" or "desc").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
}

/// Parameters for listing messages in a stored completion.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionMessageListParams {
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Maximum number of items to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Sort order ("asc" or "desc").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Chat completion response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletion {
    /// Response ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Unix timestamp of creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<i64>,
    /// Model used.
    pub model: ModelId,
    /// Choices list.
    pub choices: Vec<ChatCompletionChoice>,
    /// Token usage details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<CompletionUsage>,
    /// Service tier used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ChatCompletionServiceTier>,
    /// System fingerprint for determinism tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// Chat completion choice.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionChoice {
    /// Choice index.
    pub index: i64,
    /// Output message.
    pub message: ChatCompletionMessage,
    /// Stop reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
    /// Log probability information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<ChatCompletionChoiceLogprobs>,
}

/// Deleted chat completion response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionDeleted {
    /// The ID of the deleted completion.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Whether the completion was deleted.
    pub deleted: bool,
}

/// A stored message within a chat completion.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionStoreMessage {
    /// The message identifier.
    pub id: String,
    /// The role of the message author.
    pub role: ChatRole,
    /// The message content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

// ---------------------------------------------------------------------------
// Streaming types
// ---------------------------------------------------------------------------

/// Streaming chat chunk.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionChunk {
    /// Chunk ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Unix timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<i64>,
    /// Model used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelId>,
    /// Streaming choices.
    pub choices: Vec<ChatCompletionChunkChoice>,
    /// Token usage (only in last chunk when stream_options.include_usage is true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<CompletionUsage>,
    /// Service tier used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ChatCompletionServiceTier>,
    /// System fingerprint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// A tool call delta in a streaming chunk.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionChunkDeltaToolCall {
    /// Index of the tool call.
    pub index: i64,
    /// Tool call ID (only in first chunk for this tool call).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Tool type.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_type: Option<String>,
    /// Function details delta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<ChatCompletionChunkDeltaToolCallFunction>,
}

/// Function delta within a streaming tool call.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionChunkDeltaToolCallFunction {
    /// Function name (only in first chunk for this tool call).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Accumulated arguments fragment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// Delta content within a stream chunk.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ChatCompletionChunkDelta {
    /// Optional role update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<ChatRole>,
    /// Optional token fragment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Refusal fragment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    /// Tool call deltas.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ChatCompletionChunkDeltaToolCall>>,
    /// Legacy function call delta (deprecated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<ChatCompletionMessageFunctionCall>,
}

/// Chunk choice delta.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionChunkChoice {
    /// Choice index.
    pub index: i64,
    /// Delta payload.
    pub delta: ChatCompletionChunkDelta,
    /// Optional finish reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
    /// Log probability information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<ChatCompletionChoiceLogprobs>,
}

// ---------------------------------------------------------------------------
// Stream accumulator
// ---------------------------------------------------------------------------

/// Accumulates `ChatCompletionChunk`s into a full `ChatCompletion`.
///
/// Usage:
/// ```ignore
/// let mut acc = ChatCompletionAccumulator::new();
/// while let Some(chunk) = stream.next().await {
///     acc.add_chunk(&chunk?);
/// }
/// let completion = acc.into_completion();
/// ```
#[derive(Debug, Clone, Default)]
pub struct ChatCompletionAccumulator {
    id: Option<String>,
    object: Option<String>,
    created: Option<i64>,
    model: Option<ModelId>,
    service_tier: Option<ChatCompletionServiceTier>,
    system_fingerprint: Option<String>,
    usage: Option<CompletionUsage>,
    choices: Vec<AccumulatedChoice>,
}

#[derive(Debug, Clone, Default)]
struct AccumulatedChoice {
    index: i64,
    role: Option<ChatRole>,
    content: Option<String>,
    refusal: Option<String>,
    tool_calls: HashMap<i64, AccumulatedToolCall>,
    finish_reason: Option<FinishReason>,
    logprobs: Option<ChatCompletionChoiceLogprobs>,
}

#[derive(Debug, Clone, Default)]
struct AccumulatedToolCall {
    id: Option<String>,
    call_type: Option<String>,
    function_name: Option<String>,
    function_arguments: String,
}

impl ChatCompletionAccumulator {
    /// Creates a new empty accumulator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a streaming chunk to the accumulator.
    pub fn add_chunk(&mut self, chunk: &ChatCompletionChunk) {
        if self.id.is_none() {
            self.id = Some(chunk.id.clone());
        }
        if self.object.is_none() {
            self.object = Some("chat.completion".to_owned());
        }
        if self.created.is_none() {
            self.created = chunk.created;
        }
        if self.model.is_none() {
            self.model = chunk.model.clone();
        }
        if self.service_tier.is_none() {
            self.service_tier = chunk.service_tier.clone();
        }
        if self.system_fingerprint.is_none() && chunk.system_fingerprint.is_some() {
            self.system_fingerprint = chunk.system_fingerprint.clone();
        }
        if chunk.usage.is_some() {
            self.usage = chunk.usage.clone();
        }

        for choice in &chunk.choices {
            let idx = choice.index as usize;
            while self.choices.len() <= idx {
                self.choices.push(AccumulatedChoice {
                    index: self.choices.len() as i64,
                    ..Default::default()
                });
            }
            let acc_choice = &mut self.choices[idx];

            if let Some(ref role) = choice.delta.role {
                acc_choice.role = Some(role.clone());
            }
            if let Some(ref content) = choice.delta.content {
                acc_choice
                    .content
                    .get_or_insert_with(String::new)
                    .push_str(content);
            }
            if let Some(ref refusal) = choice.delta.refusal {
                acc_choice
                    .refusal
                    .get_or_insert_with(String::new)
                    .push_str(refusal);
            }
            if let Some(ref finish) = choice.finish_reason {
                acc_choice.finish_reason = Some(finish.clone());
            }
            if let Some(ref tool_calls) = choice.delta.tool_calls {
                for tc in tool_calls {
                    let entry = acc_choice.tool_calls.entry(tc.index).or_default();
                    if let Some(ref id) = tc.id {
                        entry.id = Some(id.clone());
                    }
                    if let Some(ref ct) = tc.call_type {
                        entry.call_type = Some(ct.clone());
                    }
                    if let Some(ref func) = tc.function {
                        if let Some(ref name) = func.name {
                            entry.function_name = Some(name.clone());
                        }
                        if let Some(ref args) = func.arguments {
                            entry.function_arguments.push_str(args);
                        }
                    }
                }
            }
        }
    }

    /// Consumes the accumulator and returns the assembled `ChatCompletion`.
    #[must_use]
    pub fn into_completion(self) -> ChatCompletion {
        let choices = self
            .choices
            .into_iter()
            .map(|c| {
                let mut tool_calls_vec: Vec<ChatCompletionMessageToolCall> = c
                    .tool_calls
                    .into_iter()
                    .map(|(_, tc)| ChatCompletionMessageToolCall {
                        id: tc.id.unwrap_or_default(),
                        call_type: tc.call_type.unwrap_or_else(|| "function".to_owned()),
                        function: ChatCompletionMessageToolCallFunction {
                            name: tc.function_name.unwrap_or_default(),
                            arguments: tc.function_arguments,
                        },
                    })
                    .collect();
                tool_calls_vec.sort_by_key(|tc| tc.id.clone());

                ChatCompletionChoice {
                    index: c.index,
                    message: ChatCompletionMessage {
                        role: c.role.unwrap_or(ChatRole::Assistant),
                        content: c.content,
                        refusal: c.refusal,
                        tool_calls: if tool_calls_vec.is_empty() {
                            None
                        } else {
                            Some(tool_calls_vec)
                        },
                        function_call: None,
                        audio: None,
                        annotations: None,
                    },
                    finish_reason: c.finish_reason,
                    logprobs: c.logprobs,
                }
            })
            .collect();

        ChatCompletion {
            id: self.id.unwrap_or_default(),
            object: self.object.unwrap_or_else(|| "chat.completion".to_owned()),
            created: self.created,
            model: self.model.unwrap_or_default(),
            choices,
            usage: self.usage,
            service_tier: self.service_tier,
            system_fingerprint: self.system_fingerprint,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ModelId;

    // --- Basic create params ---

    #[test]
    fn create_params_omit_optional_fields() {
        let params = ChatCompletionCreateParams {
            model: ModelId::from("gpt-4o-mini"),
            messages: vec![ChatCompletionMessageParam::User {
                content: ChatCompletionUserMessageContent::Text("hello".to_owned()),
                name: None,
            }],
            stream: None,
            temperature: None,
            frequency_penalty: None,
            presence_penalty: None,
            max_completion_tokens: None,
            max_tokens: None,
            n: None,
            top_p: None,
            seed: None,
            store: None,
            logprobs: None,
            top_logprobs: None,
            stop: None,
            response_format: None,
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            user: None,
            service_tier: None,
            metadata: None,
            reasoning_effort: None,
            stream_options: None,
            audio: None,
            logit_bias: None,
            modalities: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            safety_identifier: None,
            verbosity: None,
            prediction: None,
            web_search_options: None,
            function_call: None,
            functions: None,
        };

        let value = serde_json::to_value(&params).expect("serialize create params");
        assert_eq!(
            value.get("model"),
            Some(&serde_json::Value::String("gpt-4o-mini".to_owned()))
        );
        assert!(value.get("stream").is_none());
        assert!(value.get("temperature").is_none());
        assert!(value.get("tools").is_none());
        assert!(value.get("tool_choice").is_none());
        assert!(value.get("response_format").is_none());
        assert!(value.get("metadata").is_none());
    }

    // --- Chunk deserialization ---

    #[test]
    fn chunk_deserializes_finish_reason() {
        let json = r#"{
            "id":"chatcmpl_123",
            "object":"chat.completion.chunk",
            "choices":[
                {"index":0,"delta":{"role":"assistant","content":"ok"},"finish_reason":"stop"}
            ]
        }"#;

        let chunk: ChatCompletionChunk = serde_json::from_str(json).expect("deserialize chunk");
        assert_eq!(chunk.choices.len(), 1);
        assert_eq!(
            serde_json::to_string(&chunk.choices[0].finish_reason)
                .expect("serialize finish reason option"),
            "\"stop\""
        );
    }

    // --- Message param union round-trips ---

    #[test]
    fn system_message_param_round_trip() {
        let msg = ChatCompletionMessageParam::System {
            content: "You are helpful.".to_owned(),
            name: None,
        };
        let json = serde_json::to_string(&msg).expect("serialize system message");
        assert!(json.contains("\"role\":\"system\""));
        assert!(json.contains("\"content\":\"You are helpful.\""));
        assert!(!json.contains("\"name\""));

        let decoded: ChatCompletionMessageParam =
            serde_json::from_str(&json).expect("deserialize system message");
        match decoded {
            ChatCompletionMessageParam::System { content, name } => {
                assert_eq!(content, "You are helpful.");
                assert!(name.is_none());
            }
            _ => panic!("expected System variant"),
        }
    }

    #[test]
    fn user_message_text_round_trip() {
        let msg = ChatCompletionMessageParam::User {
            content: ChatCompletionUserMessageContent::Text("hi".to_owned()),
            name: Some("alice".to_owned()),
        };
        let json = serde_json::to_string(&msg).expect("serialize user message");
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"hi\""));
        assert!(json.contains("\"name\":\"alice\""));

        let decoded: ChatCompletionMessageParam =
            serde_json::from_str(&json).expect("deserialize user message");
        match decoded {
            ChatCompletionMessageParam::User { content, name } => {
                match content {
                    ChatCompletionUserMessageContent::Text(t) => assert_eq!(t, "hi"),
                    _ => panic!("expected Text content"),
                }
                assert_eq!(name, Some("alice".to_owned()));
            }
            _ => panic!("expected User variant"),
        }
    }

    #[test]
    fn user_message_parts_round_trip() {
        let msg = ChatCompletionMessageParam::User {
            content: ChatCompletionUserMessageContent::Parts(vec![
                ChatCompletionContentPart::Text {
                    text: "What is in this image?".to_owned(),
                },
                ChatCompletionContentPart::ImageUrl {
                    image_url: ChatCompletionContentPartImageUrl {
                        url: "https://example.com/cat.jpg".to_owned(),
                        detail: Some(ImageDetail::High),
                    },
                },
            ]),
            name: None,
        };
        let json = serde_json::to_string(&msg).expect("serialize user parts message");
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("\"type\":\"image_url\""));
        assert!(json.contains("\"detail\":\"high\""));

        let decoded: ChatCompletionMessageParam =
            serde_json::from_str(&json).expect("deserialize user parts message");
        match decoded {
            ChatCompletionMessageParam::User { content, .. } => match content {
                ChatCompletionUserMessageContent::Parts(parts) => {
                    assert_eq!(parts.len(), 2);
                }
                _ => panic!("expected Parts content"),
            },
            _ => panic!("expected User variant"),
        }
    }

    #[test]
    fn assistant_message_with_tool_calls_round_trip() {
        let msg = ChatCompletionMessageParam::Assistant {
            content: None,
            name: None,
            refusal: None,
            tool_calls: Some(vec![ChatCompletionMessageToolCall {
                id: "call_abc".to_owned(),
                call_type: "function".to_owned(),
                function: ChatCompletionMessageToolCallFunction {
                    name: "get_weather".to_owned(),
                    arguments: r#"{"location":"NYC"}"#.to_owned(),
                },
            }]),
            function_call: None,
        };
        let json = serde_json::to_string(&msg).expect("serialize assistant with tool calls");
        assert!(json.contains("\"role\":\"assistant\""));
        assert!(json.contains("\"call_abc\""));
        assert!(json.contains("\"get_weather\""));
        assert!(!json.contains("\"content\""));

        let decoded: ChatCompletionMessageParam =
            serde_json::from_str(&json).expect("deserialize assistant with tool calls");
        match decoded {
            ChatCompletionMessageParam::Assistant {
                tool_calls,
                content,
                ..
            } => {
                assert!(content.is_none());
                let tc = tool_calls.expect("expected tool_calls");
                assert_eq!(tc.len(), 1);
                assert_eq!(tc[0].id, "call_abc");
                assert_eq!(tc[0].function.name, "get_weather");
            }
            _ => panic!("expected Assistant variant"),
        }
    }

    #[test]
    fn tool_message_round_trip() {
        let msg = ChatCompletionMessageParam::Tool {
            content: r#"{"temp":"72F"}"#.to_owned(),
            tool_call_id: "call_abc".to_owned(),
        };
        let json = serde_json::to_string(&msg).expect("serialize tool message");
        assert!(json.contains("\"role\":\"tool\""));
        assert!(json.contains("\"tool_call_id\":\"call_abc\""));

        let decoded: ChatCompletionMessageParam =
            serde_json::from_str(&json).expect("deserialize tool message");
        match decoded {
            ChatCompletionMessageParam::Tool {
                content,
                tool_call_id,
            } => {
                assert_eq!(tool_call_id, "call_abc");
                assert!(content.contains("72F"));
            }
            _ => panic!("expected Tool variant"),
        }
    }

    #[test]
    fn function_message_round_trip() {
        let msg = ChatCompletionMessageParam::Function {
            content: "result_data".to_owned(),
            name: "my_func".to_owned(),
        };
        let json = serde_json::to_string(&msg).expect("serialize function message");
        assert!(json.contains("\"role\":\"function\""));
        assert!(json.contains("\"name\":\"my_func\""));

        let decoded: ChatCompletionMessageParam =
            serde_json::from_str(&json).expect("deserialize function message");
        match decoded {
            ChatCompletionMessageParam::Function { content, name } => {
                assert_eq!(content, "result_data");
                assert_eq!(name, "my_func");
            }
            _ => panic!("expected Function variant"),
        }
    }

    #[test]
    fn developer_message_round_trip() {
        let msg = ChatCompletionMessageParam::Developer {
            content: "Be concise.".to_owned(),
            name: None,
        };
        let json = serde_json::to_string(&msg).expect("serialize developer message");
        assert!(json.contains("\"role\":\"developer\""));
        let decoded: ChatCompletionMessageParam =
            serde_json::from_str(&json).expect("deserialize developer message");
        match decoded {
            ChatCompletionMessageParam::Developer { content, .. } => {
                assert_eq!(content, "Be concise.");
            }
            _ => panic!("expected Developer variant"),
        }
    }

    // --- Tool calling types ---

    #[test]
    fn chat_completion_tool_serializes_correctly() {
        let tool = ChatCompletionTool::function(FunctionDefinition {
            name: "get_weather".to_owned(),
            description: Some("Get the current weather".to_owned()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "location": { "type": "string" }
                },
                "required": ["location"]
            })),
            strict: None,
        });

        let json = serde_json::to_value(&tool).expect("serialize tool");
        assert_eq!(json["type"], "function");
        assert_eq!(json["function"]["name"], "get_weather");
        assert!(json["function"]["parameters"]["properties"]["location"].is_object());
    }

    #[test]
    fn tool_choice_mode_round_trip() {
        let auto = ChatCompletionToolChoiceOption::Mode("auto".to_owned());
        let json = serde_json::to_string(&auto).expect("serialize auto");
        assert_eq!(json, "\"auto\"");

        let decoded: ChatCompletionToolChoiceOption =
            serde_json::from_str(&json).expect("deserialize auto");
        match decoded {
            ChatCompletionToolChoiceOption::Mode(m) => assert_eq!(m, "auto"),
            _ => panic!("expected Mode variant"),
        }
    }

    #[test]
    fn tool_choice_named_round_trip() {
        let named = ChatCompletionToolChoiceOption::Named(ChatCompletionNamedToolChoice {
            choice_type: "function".to_owned(),
            function: ChatCompletionNamedToolChoiceFunction {
                name: "get_weather".to_owned(),
            },
        });
        let json = serde_json::to_string(&named).expect("serialize named tool choice");
        assert!(json.contains("\"function\""));
        assert!(json.contains("\"get_weather\""));

        let decoded: ChatCompletionToolChoiceOption =
            serde_json::from_str(&json).expect("deserialize named tool choice");
        match decoded {
            ChatCompletionToolChoiceOption::Named(n) => {
                assert_eq!(n.function.name, "get_weather");
            }
            _ => panic!("expected Named variant"),
        }
    }

    // --- Response format ---

    #[test]
    fn response_format_text_round_trip() {
        let fmt = ResponseFormat::Text;
        let json = serde_json::to_string(&fmt).expect("serialize text format");
        assert!(json.contains("\"type\":\"text\""));

        let decoded: ResponseFormat = serde_json::from_str(&json).expect("deserialize text format");
        assert!(matches!(decoded, ResponseFormat::Text));
    }

    #[test]
    fn response_format_json_object_round_trip() {
        let fmt = ResponseFormat::JsonObject;
        let json = serde_json::to_string(&fmt).expect("serialize json_object format");
        assert!(json.contains("\"type\":\"json_object\""));

        let decoded: ResponseFormat =
            serde_json::from_str(&json).expect("deserialize json_object format");
        assert!(matches!(decoded, ResponseFormat::JsonObject));
    }

    #[test]
    fn response_format_json_schema_round_trip() {
        let fmt = ResponseFormat::JsonSchema {
            json_schema: ResponseFormatJsonSchemaDefinition {
                name: "my_schema".to_owned(),
                description: Some("A test schema".to_owned()),
                schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "answer": { "type": "string" }
                    }
                })),
                strict: Some(true),
            },
        };
        let json = serde_json::to_string(&fmt).expect("serialize json_schema format");
        assert!(json.contains("\"type\":\"json_schema\""));
        assert!(json.contains("\"name\":\"my_schema\""));
        assert!(json.contains("\"strict\":true"));

        let decoded: ResponseFormat =
            serde_json::from_str(&json).expect("deserialize json_schema format");
        match decoded {
            ResponseFormat::JsonSchema { json_schema } => {
                assert_eq!(json_schema.name, "my_schema");
                assert_eq!(json_schema.strict, Some(true));
            }
            _ => panic!("expected JsonSchema variant"),
        }
    }

    // --- Content parts ---

    #[test]
    fn content_part_text_round_trip() {
        let part = ChatCompletionContentPart::Text {
            text: "hello".to_owned(),
        };
        let json = serde_json::to_string(&part).expect("serialize text part");
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("\"text\":\"hello\""));

        let decoded: ChatCompletionContentPart =
            serde_json::from_str(&json).expect("deserialize text part");
        match decoded {
            ChatCompletionContentPart::Text { text } => assert_eq!(text, "hello"),
            _ => panic!("expected Text variant"),
        }
    }

    #[test]
    fn content_part_image_url_round_trip() {
        let part = ChatCompletionContentPart::ImageUrl {
            image_url: ChatCompletionContentPartImageUrl {
                url: "https://example.com/img.png".to_owned(),
                detail: Some(ImageDetail::Low),
            },
        };
        let json = serde_json::to_string(&part).expect("serialize image_url part");
        assert!(json.contains("\"type\":\"image_url\""));
        assert!(json.contains("\"detail\":\"low\""));

        let decoded: ChatCompletionContentPart =
            serde_json::from_str(&json).expect("deserialize image_url part");
        match decoded {
            ChatCompletionContentPart::ImageUrl { image_url } => {
                assert_eq!(image_url.url, "https://example.com/img.png");
                assert_eq!(image_url.detail, Some(ImageDetail::Low));
            }
            _ => panic!("expected ImageUrl variant"),
        }
    }

    #[test]
    fn content_part_input_audio_round_trip() {
        let part = ChatCompletionContentPart::InputAudio {
            input_audio: ChatCompletionContentPartInputAudioData {
                data: "base64data".to_owned(),
                format: "wav".to_owned(),
            },
        };
        let json = serde_json::to_string(&part).expect("serialize input_audio part");
        assert!(json.contains("\"type\":\"input_audio\""));

        let decoded: ChatCompletionContentPart =
            serde_json::from_str(&json).expect("deserialize input_audio part");
        match decoded {
            ChatCompletionContentPart::InputAudio { input_audio } => {
                assert_eq!(input_audio.data, "base64data");
                assert_eq!(input_audio.format, "wav");
            }
            _ => panic!("expected InputAudio variant"),
        }
    }

    // --- Image detail ---

    #[test]
    fn image_detail_serializes_lowercase() {
        assert_eq!(
            serde_json::to_string(&ImageDetail::Auto).unwrap(),
            "\"auto\""
        );
        assert_eq!(serde_json::to_string(&ImageDetail::Low).unwrap(), "\"low\"");
        assert_eq!(
            serde_json::to_string(&ImageDetail::High).unwrap(),
            "\"high\""
        );
    }

    // --- Logprobs ---

    #[test]
    fn token_logprob_deserializes() {
        let json = r#"{
            "token":"hello",
            "logprob":-0.5,
            "bytes":[104,101,108,108,111],
            "top_logprobs":[
                {"token":"hello","logprob":-0.5,"bytes":[104,101,108,108,111]},
                {"token":"hi","logprob":-1.2,"bytes":[104,105]}
            ]
        }"#;

        let logprob: ChatCompletionTokenLogprob =
            serde_json::from_str(json).expect("deserialize token logprob");
        assert_eq!(logprob.token, "hello");
        assert!((logprob.logprob - (-0.5)).abs() < f64::EPSILON);
        assert_eq!(logprob.top_logprobs.as_ref().unwrap().len(), 2);
    }

    // --- Usage ---

    #[test]
    fn completion_usage_basic_round_trip() {
        let usage = CompletionUsage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
            completion_tokens_details: None,
            prompt_tokens_details: None,
        };
        let json = serde_json::to_string(&usage).expect("serialize usage");
        assert!(!json.contains("completion_tokens_details"));
        assert!(!json.contains("prompt_tokens_details"));

        let decoded: CompletionUsage = serde_json::from_str(&json).expect("deserialize usage");
        assert_eq!(decoded.total_tokens, 30);
    }

    #[test]
    fn completion_usage_detailed_round_trip() {
        let usage = CompletionUsage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
            completion_tokens_details: Some(CompletionTokensDetails {
                reasoning_tokens: Some(5),
                audio_tokens: None,
                accepted_prediction_tokens: Some(2),
                rejected_prediction_tokens: Some(1),
            }),
            prompt_tokens_details: Some(PromptTokensDetails {
                audio_tokens: Some(3),
                cached_tokens: Some(7),
            }),
        };
        let json = serde_json::to_string(&usage).expect("serialize detailed usage");
        assert!(json.contains("\"reasoning_tokens\":5"));
        assert!(json.contains("\"cached_tokens\":7"));

        let decoded: CompletionUsage =
            serde_json::from_str(&json).expect("deserialize detailed usage");
        assert_eq!(
            decoded
                .completion_tokens_details
                .as_ref()
                .unwrap()
                .reasoning_tokens,
            Some(5)
        );
        assert_eq!(
            decoded
                .prompt_tokens_details
                .as_ref()
                .unwrap()
                .cached_tokens,
            Some(7)
        );
    }

    // --- ChatCompletion response ---

    #[test]
    fn chat_completion_response_deserializes() {
        let json = r#"{
            "id": "chatcmpl-abc123",
            "object": "chat.completion",
            "created": 1700000000,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 5,
                "completion_tokens": 3,
                "total_tokens": 8
            }
        }"#;

        let completion: ChatCompletion =
            serde_json::from_str(json).expect("deserialize chat completion");
        assert_eq!(completion.id, "chatcmpl-abc123");
        assert_eq!(completion.choices.len(), 1);
        assert_eq!(
            completion.choices[0]
                .message
                .content
                .as_deref()
                .unwrap_or(""),
            "Hello!"
        );
        assert_eq!(completion.usage.as_ref().unwrap().total_tokens, 8);
    }

    #[test]
    fn chat_completion_response_with_tool_calls() {
        let json = r#"{
            "id": "chatcmpl-xyz",
            "object": "chat.completion",
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "tool_calls": [{
                        "id": "call_123",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"location\":\"NYC\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }]
        }"#;

        let completion: ChatCompletion =
            serde_json::from_str(json).expect("deserialize completion with tool calls");
        let msg = &completion.choices[0].message;
        assert!(msg.content.is_none());
        let tc = msg.tool_calls.as_ref().expect("expected tool_calls");
        assert_eq!(tc.len(), 1);
        assert_eq!(tc[0].id, "call_123");
        assert_eq!(tc[0].function.name, "get_weather");
    }

    #[test]
    fn chat_completion_response_with_logprobs() {
        let json = r#"{
            "id": "chatcmpl-lp",
            "object": "chat.completion",
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": { "role": "assistant", "content": "Hi" },
                "finish_reason": "stop",
                "logprobs": {
                    "content": [{
                        "token": "Hi",
                        "logprob": -0.1,
                        "bytes": [72, 105],
                        "top_logprobs": [
                            { "token": "Hi", "logprob": -0.1, "bytes": [72, 105] }
                        ]
                    }]
                }
            }]
        }"#;

        let completion: ChatCompletion =
            serde_json::from_str(json).expect("deserialize completion with logprobs");
        let lp = completion.choices[0]
            .logprobs
            .as_ref()
            .expect("expected logprobs");
        let content_lp = lp.content.as_ref().expect("expected content logprobs");
        assert_eq!(content_lp.len(), 1);
        assert_eq!(content_lp[0].token, "Hi");
    }

    // --- Deleted completion ---

    #[test]
    fn chat_completion_deleted_round_trip() {
        let deleted = ChatCompletionDeleted {
            id: "chatcmpl-abc".to_owned(),
            object: "chat.completion.deleted".to_owned(),
            deleted: true,
        };
        let json = serde_json::to_string(&deleted).expect("serialize deleted");
        let decoded: ChatCompletionDeleted =
            serde_json::from_str(&json).expect("deserialize deleted");
        assert_eq!(decoded.id, "chatcmpl-abc");
        assert!(decoded.deleted);
    }

    // --- Stored message ---

    #[test]
    fn store_message_deserializes() {
        let json = r#"{
            "id": "msg_abc",
            "role": "user",
            "content": "Hello"
        }"#;

        let msg: ChatCompletionStoreMessage =
            serde_json::from_str(json).expect("deserialize store message");
        assert_eq!(msg.id, "msg_abc");
        assert_eq!(msg.role, ChatRole::User);
        assert_eq!(msg.content, Some("Hello".to_owned()));
    }

    // --- List params ---

    #[test]
    fn list_params_defaults_are_none() {
        let params = ChatCompletionListParams::default();
        assert!(params.after.is_none());
        assert!(params.limit.is_none());
        assert!(params.model.is_none());
        assert!(params.order.is_none());
    }

    // --- Update params ---

    #[test]
    fn update_params_serializes_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_owned(), "value1".to_owned());
        let params = ChatCompletionUpdateParams { metadata };
        let json = serde_json::to_value(&params).expect("serialize update params");
        assert_eq!(json["metadata"]["key1"], "value1");
    }

    // --- Create params with tools ---

    #[test]
    fn create_params_with_tools_round_trip() {
        let params = ChatCompletionCreateParams {
            model: ModelId::from("gpt-4o"),
            messages: vec![ChatCompletionMessageParam::User {
                content: ChatCompletionUserMessageContent::Text("What is the weather?".to_owned()),
                name: None,
            }],
            stream: None,
            temperature: None,
            frequency_penalty: None,
            presence_penalty: None,
            max_completion_tokens: None,
            max_tokens: None,
            n: None,
            top_p: None,
            seed: None,
            store: None,
            logprobs: None,
            top_logprobs: None,
            stop: None,
            response_format: None,
            tools: Some(vec![ChatCompletionTool::function(FunctionDefinition {
                name: "get_weather".to_owned(),
                description: Some("Get weather for a location".to_owned()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "location": { "type": "string" }
                    },
                    "required": ["location"]
                })),
                strict: None,
            })]),
            tool_choice: Some(ChatCompletionToolChoiceOption::Mode("auto".to_owned())),
            parallel_tool_calls: None,
            user: None,
            service_tier: None,
            metadata: None,
            reasoning_effort: None,
            stream_options: None,
            audio: None,
            logit_bias: None,
            modalities: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            safety_identifier: None,
            verbosity: None,
            prediction: None,
            web_search_options: None,
            function_call: None,
            functions: None,
        };

        let json = serde_json::to_value(&params).expect("serialize params with tools");
        assert_eq!(json["tools"][0]["type"], "function");
        assert_eq!(json["tools"][0]["function"]["name"], "get_weather");
        assert_eq!(json["tool_choice"], "auto");
    }

    // --- Create params with response format ---

    #[test]
    fn create_params_with_json_schema_response_format() {
        let params = ChatCompletionCreateParams {
            model: ModelId::from("gpt-4o"),
            messages: vec![ChatCompletionMessageParam::User {
                content: ChatCompletionUserMessageContent::Text("list colors".to_owned()),
                name: None,
            }],
            stream: None,
            temperature: None,
            frequency_penalty: None,
            presence_penalty: None,
            max_completion_tokens: None,
            max_tokens: None,
            n: None,
            top_p: None,
            seed: None,
            store: None,
            logprobs: None,
            top_logprobs: None,
            stop: None,
            response_format: Some(ResponseFormat::JsonSchema {
                json_schema: ResponseFormatJsonSchemaDefinition {
                    name: "colors".to_owned(),
                    description: None,
                    schema: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "colors": {
                                "type": "array",
                                "items": { "type": "string" }
                            }
                        }
                    })),
                    strict: Some(true),
                },
            }),
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            user: None,
            service_tier: None,
            metadata: None,
            reasoning_effort: None,
            stream_options: None,
            audio: None,
            logit_bias: None,
            modalities: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            safety_identifier: None,
            verbosity: None,
            prediction: None,
            web_search_options: None,
            function_call: None,
            functions: None,
        };

        let json = serde_json::to_value(&params).expect("serialize with response_format");
        assert_eq!(json["response_format"]["type"], "json_schema");
        assert_eq!(json["response_format"]["json_schema"]["name"], "colors");
        assert_eq!(json["response_format"]["json_schema"]["strict"], true);
    }

    // --- Streaming chunk with tool calls ---

    #[test]
    fn chunk_with_tool_calls_deserializes() {
        let json = r#"{
            "id":"chatcmpl-stream",
            "object":"chat.completion.chunk",
            "choices":[{
                "index":0,
                "delta":{
                    "tool_calls":[{
                        "index":0,
                        "id":"call_456",
                        "type":"function",
                        "function":{"name":"get_weather","arguments":"{\"loc"}
                    }]
                },
                "finish_reason":null
            }]
        }"#;

        let chunk: ChatCompletionChunk =
            serde_json::from_str(json).expect("deserialize chunk with tool calls");
        let delta = &chunk.choices[0].delta;
        let tc = delta.tool_calls.as_ref().expect("expected tool_calls");
        assert_eq!(tc.len(), 1);
        assert_eq!(tc[0].id, Some("call_456".to_owned()));
        assert_eq!(
            tc[0].function.as_ref().unwrap().name,
            Some("get_weather".to_owned())
        );
    }

    // --- ChatCompletionMessage response with function_call ---

    #[test]
    fn response_message_with_function_call() {
        let json = r#"{
            "role": "assistant",
            "content": null,
            "function_call": {
                "name": "old_func",
                "arguments": "{}"
            }
        }"#;

        let msg: ChatCompletionMessage =
            serde_json::from_str(json).expect("deserialize message with function_call");
        assert!(msg.content.is_none());
        let fc = msg.function_call.as_ref().expect("expected function_call");
        assert_eq!(fc.name, "old_func");
    }

    // --- Full tool-calling round-trip scenario ---

    #[test]
    fn full_tool_calling_scenario() {
        // 1. Build request with tool
        let request = ChatCompletionCreateParams {
            model: ModelId::from("gpt-4o"),
            messages: vec![ChatCompletionMessageParam::User {
                content: ChatCompletionUserMessageContent::Text(
                    "What is the weather in NYC?".to_owned(),
                ),
                name: None,
            }],
            tools: Some(vec![ChatCompletionTool::function(FunctionDefinition {
                name: "get_weather".to_owned(),
                description: Some("Get the current weather".to_owned()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": { "location": { "type": "string" } },
                    "required": ["location"]
                })),
                strict: None,
            })]),
            tool_choice: Some(ChatCompletionToolChoiceOption::Mode("auto".to_owned())),
            stream: None,
            temperature: None,
            frequency_penalty: None,
            presence_penalty: None,
            max_completion_tokens: None,
            max_tokens: None,
            n: None,
            top_p: None,
            seed: None,
            store: None,
            logprobs: None,
            top_logprobs: None,
            stop: None,
            response_format: None,
            parallel_tool_calls: None,
            user: None,
            service_tier: None,
            metadata: None,
            reasoning_effort: None,
            stream_options: None,
            audio: None,
            logit_bias: None,
            modalities: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            safety_identifier: None,
            verbosity: None,
            prediction: None,
            web_search_options: None,
            function_call: None,
            functions: None,
        };

        // Serialize to JSON and back
        let json = serde_json::to_string(&request).expect("serialize request");
        let decoded: ChatCompletionCreateParams =
            serde_json::from_str(&json).expect("deserialize request");
        assert_eq!(decoded.tools.as_ref().unwrap().len(), 1);

        // 2. Simulate response with tool call
        let response_json = r#"{
            "id": "chatcmpl-test",
            "object": "chat.completion",
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "tool_calls": [{
                        "id": "call_abc",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"location\":\"NYC\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 15,
                "total_tokens": 25
            }
        }"#;

        let response: ChatCompletion =
            serde_json::from_str(response_json).expect("deserialize response");
        let tool_calls = response.choices[0]
            .message
            .tool_calls
            .as_ref()
            .expect("expected tool_calls");
        assert_eq!(tool_calls[0].function.name, "get_weather");

        // 3. Build follow-up with tool result
        let follow_up = ChatCompletionCreateParams {
            model: ModelId::from("gpt-4o"),
            messages: vec![
                ChatCompletionMessageParam::User {
                    content: ChatCompletionUserMessageContent::Text(
                        "What is the weather in NYC?".to_owned(),
                    ),
                    name: None,
                },
                ChatCompletionMessageParam::Assistant {
                    content: None,
                    name: None,
                    refusal: None,
                    tool_calls: Some(vec![ChatCompletionMessageToolCall {
                        id: "call_abc".to_owned(),
                        call_type: "function".to_owned(),
                        function: ChatCompletionMessageToolCallFunction {
                            name: "get_weather".to_owned(),
                            arguments: r#"{"location":"NYC"}"#.to_owned(),
                        },
                    }]),
                    function_call: None,
                },
                ChatCompletionMessageParam::Tool {
                    content: r#"{"temperature":"72F","condition":"sunny"}"#.to_owned(),
                    tool_call_id: "call_abc".to_owned(),
                },
            ],
            tools: decoded.tools,
            tool_choice: None,
            stream: None,
            temperature: None,
            frequency_penalty: None,
            presence_penalty: None,
            max_completion_tokens: None,
            max_tokens: None,
            n: None,
            top_p: None,
            seed: None,
            store: None,
            logprobs: None,
            top_logprobs: None,
            stop: None,
            response_format: None,
            parallel_tool_calls: None,
            user: None,
            service_tier: None,
            metadata: None,
            reasoning_effort: None,
            stream_options: None,
            audio: None,
            logit_bias: None,
            modalities: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            safety_identifier: None,
            verbosity: None,
            prediction: None,
            web_search_options: None,
            function_call: None,
            functions: None,
        };

        let follow_up_json = serde_json::to_string(&follow_up).expect("serialize follow up");
        assert!(follow_up_json.contains("\"tool_call_id\":\"call_abc\""));
    }

    // --- Content part param constructors ---

    #[test]
    fn content_part_text_param_constructor() {
        let part = ChatCompletionContentPartTextParam::new("hello world");
        assert_eq!(part.part_type, "text");
        assert_eq!(part.text, "hello world");
    }

    #[test]
    fn content_part_image_param_constructor() {
        let part = ChatCompletionContentPartImageParam::new(
            "https://example.com/img.png",
            Some(ImageDetail::High),
        );
        assert_eq!(part.part_type, "image_url");
        assert_eq!(part.image_url.url, "https://example.com/img.png");
        assert_eq!(part.image_url.detail, Some(ImageDetail::High));
    }

    #[test]
    fn content_part_input_audio_param_constructor() {
        let part = ChatCompletionContentPartInputAudioParam::new("base64abc", "mp3");
        assert_eq!(part.part_type, "input_audio");
        assert_eq!(part.input_audio.data, "base64abc");
        assert_eq!(part.input_audio.format, "mp3");
    }

    // --- Message list params ---

    #[test]
    fn message_list_params_defaults() {
        let params = ChatCompletionMessageListParams::default();
        assert!(params.after.is_none());
        assert!(params.limit.is_none());
        assert!(params.order.is_none());
    }

    // --- Stream options ---

    #[test]
    fn stream_options_round_trip() {
        let opts = ChatCompletionStreamOptions {
            include_usage: Some(true),
        };
        let json = serde_json::to_value(&opts).expect("serialize stream options");
        assert_eq!(json["include_usage"], true);
    }

    // --- FunctionDefinition with strict ---

    #[test]
    fn function_definition_strict_round_trip() {
        let def = FunctionDefinition {
            name: "strict_fn".to_owned(),
            description: None,
            parameters: Some(serde_json::json!({"type": "object"})),
            strict: Some(true),
        };
        let json = serde_json::to_value(&def).expect("serialize function definition");
        assert_eq!(json["strict"], true);
        assert!(json.get("description").is_none());

        let decoded: FunctionDefinition =
            serde_json::from_value(json).expect("deserialize function definition");
        assert_eq!(decoded.strict, Some(true));
        assert!(decoded.description.is_none());
    }

    // --- New typed enums ---

    #[test]
    fn service_tier_enum_serializes() {
        assert_eq!(
            serde_json::to_string(&ChatCompletionServiceTier::Auto).unwrap(),
            "\"auto\""
        );
        assert_eq!(
            serde_json::to_string(&ChatCompletionServiceTier::Flex).unwrap(),
            "\"flex\""
        );
        assert_eq!(
            serde_json::to_string(&ChatCompletionServiceTier::Priority).unwrap(),
            "\"priority\""
        );
        let decoded: ChatCompletionServiceTier =
            serde_json::from_str("\"scale\"").expect("deserialize service tier");
        assert_eq!(decoded, ChatCompletionServiceTier::Scale);
    }

    #[test]
    fn reasoning_effort_enum_serializes() {
        assert_eq!(
            serde_json::to_string(&ChatCompletionReasoningEffort::Medium).unwrap(),
            "\"medium\""
        );
        assert_eq!(
            serde_json::to_string(&ChatCompletionReasoningEffort::High).unwrap(),
            "\"high\""
        );
        let decoded: ChatCompletionReasoningEffort =
            serde_json::from_str("\"low\"").expect("deserialize reasoning effort");
        assert_eq!(decoded, ChatCompletionReasoningEffort::Low);
    }

    #[test]
    fn prompt_cache_retention_round_trip() {
        let val = ChatCompletionPromptCacheRetention::TwentyFourHours;
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "\"24h\"");
        let decoded: ChatCompletionPromptCacheRetention =
            serde_json::from_str("\"in-memory\"").unwrap();
        assert_eq!(decoded, ChatCompletionPromptCacheRetention::InMemory);
    }

    #[test]
    fn verbosity_enum_round_trip() {
        assert_eq!(
            serde_json::to_string(&ChatCompletionVerbosity::Low).unwrap(),
            "\"low\""
        );
        let decoded: ChatCompletionVerbosity = serde_json::from_str("\"high\"").unwrap();
        assert_eq!(decoded, ChatCompletionVerbosity::High);
    }

    // --- Audio types ---

    #[test]
    fn audio_param_serializes() {
        let param = ChatCompletionAudioParam {
            format: ChatCompletionAudioFormat::Mp3,
            voice: ChatCompletionAudioVoice::BuiltIn(ChatCompletionAudioVoiceString::Alloy),
        };
        let json = serde_json::to_value(&param).expect("serialize audio param");
        assert_eq!(json["format"], "mp3");
        assert_eq!(json["voice"], "alloy");
    }

    #[test]
    fn audio_param_custom_voice() {
        let param = ChatCompletionAudioParam {
            format: ChatCompletionAudioFormat::Wav,
            voice: ChatCompletionAudioVoice::Custom(ChatCompletionAudioVoiceID {
                id: "voice_1234".to_owned(),
            }),
        };
        let json = serde_json::to_value(&param).expect("serialize custom voice");
        assert_eq!(json["voice"]["id"], "voice_1234");
    }

    #[test]
    fn audio_response_deserializes() {
        let json = r#"{
            "id": "audio_123",
            "data": "base64audiodata",
            "expires_at": 1700000000,
            "transcript": "Hello world"
        }"#;
        let audio: ChatCompletionAudio =
            serde_json::from_str(json).expect("deserialize audio response");
        assert_eq!(audio.id, "audio_123");
        assert_eq!(audio.transcript, "Hello world");
    }

    // --- Web search types ---

    #[test]
    fn web_search_options_serializes() {
        let opts = WebSearchOptions {
            search_context_size: Some("medium".to_owned()),
            user_location: None,
        };
        let json = serde_json::to_value(&opts).expect("serialize web search options");
        assert_eq!(json["search_context_size"], "medium");
        assert!(json.get("user_location").is_none());
    }

    #[test]
    fn message_annotation_deserializes() {
        let json = r#"{
            "type": "url_citation",
            "url_citation": {
                "start_index": 0,
                "end_index": 10,
                "title": "Example",
                "url": "https://example.com"
            }
        }"#;
        let ann: ChatCompletionMessageAnnotation =
            serde_json::from_str(json).expect("deserialize annotation");
        assert_eq!(ann.url_citation.title, "Example");
        assert_eq!(ann.url_citation.start_index, 0);
        assert_eq!(ann.url_citation.end_index, 10);
    }

    #[test]
    fn message_with_annotations_deserializes() {
        let json = r#"{
            "role": "assistant",
            "content": "According to sources",
            "annotations": [{
                "type": "url_citation",
                "url_citation": {
                    "start_index": 0,
                    "end_index": 20,
                    "title": "Source",
                    "url": "https://example.com"
                }
            }]
        }"#;
        let msg: ChatCompletionMessage =
            serde_json::from_str(json).expect("deserialize message with annotations");
        let anns = msg.annotations.as_ref().expect("expected annotations");
        assert_eq!(anns.len(), 1);
        assert_eq!(anns[0].url_citation.url, "https://example.com");
    }

    // --- Prediction types ---

    #[test]
    fn prediction_content_param_serializes() {
        let pred = ChatCompletionPredictionContentParam {
            content: ChatCompletionPredictionContent::Text("predicted output".to_owned()),
            prediction_type: "content".to_owned(),
        };
        let json = serde_json::to_value(&pred).expect("serialize prediction");
        assert_eq!(json["content"], "predicted output");
        assert_eq!(json["type"], "content");
    }

    // --- Custom tool types ---

    #[test]
    fn custom_tool_param_serializes() {
        let tool = ChatCompletionCustomToolParam {
            custom: ChatCompletionCustomToolCustomParam {
                name: "my_tool".to_owned(),
                description: Some("A custom tool".to_owned()),
                format: Some(CustomToolFormat::Text),
            },
            tool_type: "custom".to_owned(),
        };
        let json = serde_json::to_value(&tool).expect("serialize custom tool");
        assert_eq!(json["type"], "custom");
        assert_eq!(json["custom"]["name"], "my_tool");
    }

    #[test]
    fn custom_tool_grammar_format_serializes() {
        let tool = ChatCompletionCustomToolParam {
            custom: ChatCompletionCustomToolCustomParam {
                name: "grammar_tool".to_owned(),
                description: None,
                format: Some(CustomToolFormat::Grammar {
                    grammar: CustomToolGrammar {
                        definition: "start: expr".to_owned(),
                        syntax: GrammarSyntax::Lark,
                    },
                }),
            },
            tool_type: "custom".to_owned(),
        };
        let json = serde_json::to_value(&tool).expect("serialize grammar tool");
        assert_eq!(json["custom"]["format"]["type"], "grammar");
        assert_eq!(json["custom"]["format"]["grammar"]["syntax"], "lark");
    }

    // --- File content part ---

    #[test]
    fn file_content_part_from_file_id() {
        let part = ChatCompletionContentPartFileParam::from_file_id("file-abc123");
        assert_eq!(part.part_type, "file");
        assert_eq!(part.file.file_id, Some("file-abc123".to_owned()));
        assert!(part.file.file_data.is_none());
    }

    #[test]
    fn file_content_part_round_trip() {
        let part = ChatCompletionContentPart::File {
            file: ChatCompletionContentPartFileData {
                file_data: None,
                file_id: Some("file-xyz".to_owned()),
                filename: Some("doc.pdf".to_owned()),
            },
        };
        let json = serde_json::to_string(&part).expect("serialize file content part");
        assert!(json.contains("\"type\":\"file\""));
        assert!(json.contains("\"file_id\":\"file-xyz\""));

        let decoded: ChatCompletionContentPart =
            serde_json::from_str(&json).expect("deserialize file content part");
        match decoded {
            ChatCompletionContentPart::File { file } => {
                assert_eq!(file.file_id, Some("file-xyz".to_owned()));
            }
            _ => panic!("expected File variant"),
        }
    }

    // --- Accumulator ---

    #[test]
    fn accumulator_assembles_simple_completion() {
        let mut acc = ChatCompletionAccumulator::new();

        // First chunk: role
        let chunk1 = ChatCompletionChunk {
            id: "chatcmpl-test".to_owned(),
            object: "chat.completion.chunk".to_owned(),
            created: Some(1700000000),
            model: Some(ModelId::from("gpt-4o")),
            choices: vec![ChatCompletionChunkChoice {
                index: 0,
                delta: ChatCompletionChunkDelta {
                    role: Some(ChatRole::Assistant),
                    content: Some("Hello".to_owned()),
                    ..Default::default()
                },
                finish_reason: None,
                logprobs: None,
            }],
            usage: None,
            service_tier: None,
            system_fingerprint: None,
        };
        acc.add_chunk(&chunk1);

        // Second chunk: more content
        let chunk2 = ChatCompletionChunk {
            id: "chatcmpl-test".to_owned(),
            object: "chat.completion.chunk".to_owned(),
            created: Some(1700000000),
            model: Some(ModelId::from("gpt-4o")),
            choices: vec![ChatCompletionChunkChoice {
                index: 0,
                delta: ChatCompletionChunkDelta {
                    content: Some(" world!".to_owned()),
                    ..Default::default()
                },
                finish_reason: Some(FinishReason::Stop),
                logprobs: None,
            }],
            usage: None,
            service_tier: None,
            system_fingerprint: None,
        };
        acc.add_chunk(&chunk2);

        let completion = acc.into_completion();
        assert_eq!(completion.id, "chatcmpl-test");
        assert_eq!(completion.choices.len(), 1);
        assert_eq!(
            completion.choices[0].message.content.as_deref(),
            Some("Hello world!")
        );
        assert_eq!(
            completion.choices[0].finish_reason,
            Some(FinishReason::Stop)
        );
        assert_eq!(completion.choices[0].message.role, ChatRole::Assistant);
    }

    #[test]
    fn accumulator_with_tool_calls() {
        let mut acc = ChatCompletionAccumulator::new();

        let chunk1 = ChatCompletionChunk {
            id: "chatcmpl-tc".to_owned(),
            object: "chat.completion.chunk".to_owned(),
            created: Some(1700000000),
            model: Some(ModelId::from("gpt-4o")),
            choices: vec![ChatCompletionChunkChoice {
                index: 0,
                delta: ChatCompletionChunkDelta {
                    role: Some(ChatRole::Assistant),
                    tool_calls: Some(vec![ChatCompletionChunkDeltaToolCall {
                        index: 0,
                        id: Some("call_abc".to_owned()),
                        call_type: Some("function".to_owned()),
                        function: Some(ChatCompletionChunkDeltaToolCallFunction {
                            name: Some("get_weather".to_owned()),
                            arguments: Some("{\"loc".to_owned()),
                        }),
                    }]),
                    ..Default::default()
                },
                finish_reason: None,
                logprobs: None,
            }],
            usage: None,
            service_tier: None,
            system_fingerprint: None,
        };
        acc.add_chunk(&chunk1);

        let chunk2 = ChatCompletionChunk {
            id: "chatcmpl-tc".to_owned(),
            object: "chat.completion.chunk".to_owned(),
            created: Some(1700000000),
            model: Some(ModelId::from("gpt-4o")),
            choices: vec![ChatCompletionChunkChoice {
                index: 0,
                delta: ChatCompletionChunkDelta {
                    tool_calls: Some(vec![ChatCompletionChunkDeltaToolCall {
                        index: 0,
                        id: None,
                        call_type: None,
                        function: Some(ChatCompletionChunkDeltaToolCallFunction {
                            name: None,
                            arguments: Some("ation\":\"NYC\"}".to_owned()),
                        }),
                    }]),
                    ..Default::default()
                },
                finish_reason: Some(FinishReason::ToolCalls),
                logprobs: None,
            }],
            usage: None,
            service_tier: None,
            system_fingerprint: None,
        };
        acc.add_chunk(&chunk2);

        let completion = acc.into_completion();
        let tc = completion.choices[0]
            .message
            .tool_calls
            .as_ref()
            .expect("tool calls");
        assert_eq!(tc.len(), 1);
        assert_eq!(tc[0].id, "call_abc");
        assert_eq!(tc[0].function.name, "get_weather");
        assert_eq!(tc[0].function.arguments, "{\"location\":\"NYC\"}");
    }

    // --- Accumulator edge cases ---

    #[test]
    fn accumulator_empty_choices_chunk_does_not_panic() {
        let mut acc = ChatCompletionAccumulator::new();

        // A chunk with zero choices should be safely handled.
        let chunk = ChatCompletionChunk {
            id: "chatcmpl-empty".to_owned(),
            object: "chat.completion.chunk".to_owned(),
            created: Some(1700000000),
            model: Some(ModelId::from("gpt-4o")),
            choices: vec![],
            usage: None,
            service_tier: None,
            system_fingerprint: None,
        };
        acc.add_chunk(&chunk);

        let completion = acc.into_completion();
        assert_eq!(completion.id, "chatcmpl-empty");
        assert!(completion.choices.is_empty());
    }

    #[test]
    fn accumulator_chunk_with_usage_details() {
        let mut acc = ChatCompletionAccumulator::new();

        // First chunk with content
        let chunk1 = ChatCompletionChunk {
            id: "chatcmpl-usage".to_owned(),
            object: "chat.completion.chunk".to_owned(),
            created: Some(1700000000),
            model: Some(ModelId::from("gpt-4o")),
            choices: vec![ChatCompletionChunkChoice {
                index: 0,
                delta: ChatCompletionChunkDelta {
                    role: Some(ChatRole::Assistant),
                    content: Some("Hi".to_owned()),
                    ..Default::default()
                },
                finish_reason: Some(FinishReason::Stop),
                logprobs: None,
            }],
            usage: None,
            service_tier: None,
            system_fingerprint: None,
        };
        acc.add_chunk(&chunk1);

        // Final chunk with usage including completion_tokens_details
        let chunk2 = ChatCompletionChunk {
            id: "chatcmpl-usage".to_owned(),
            object: "chat.completion.chunk".to_owned(),
            created: Some(1700000000),
            model: Some(ModelId::from("gpt-4o")),
            choices: vec![],
            usage: Some(CompletionUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
                completion_tokens_details: Some(CompletionTokensDetails {
                    reasoning_tokens: Some(3),
                    audio_tokens: None,
                    accepted_prediction_tokens: Some(1),
                    rejected_prediction_tokens: Some(0),
                }),
                prompt_tokens_details: Some(PromptTokensDetails {
                    audio_tokens: None,
                    cached_tokens: Some(4),
                }),
            }),
            service_tier: None,
            system_fingerprint: None,
        };
        acc.add_chunk(&chunk2);

        let completion = acc.into_completion();
        let usage = completion.usage.expect("expected usage");
        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.completion_tokens, 5);
        assert_eq!(usage.total_tokens, 15);

        let details = usage
            .completion_tokens_details
            .expect("expected completion_tokens_details");
        assert_eq!(details.reasoning_tokens, Some(3));
        assert_eq!(details.accepted_prediction_tokens, Some(1));
        assert_eq!(details.rejected_prediction_tokens, Some(0));
        assert!(details.audio_tokens.is_none());

        let prompt_details = usage
            .prompt_tokens_details
            .expect("expected prompt_tokens_details");
        assert_eq!(prompt_details.cached_tokens, Some(4));
    }

    #[test]
    fn accumulator_into_completion_on_empty() {
        let acc = ChatCompletionAccumulator::new();
        let completion = acc.into_completion();

        // Empty accumulator should produce sensible defaults, not panic.
        assert_eq!(completion.id, "");
        assert_eq!(completion.object, "chat.completion");
        assert!(completion.choices.is_empty());
        assert!(completion.usage.is_none());
        assert!(completion.created.is_none());
        assert!(completion.service_tier.is_none());
        assert!(completion.system_fingerprint.is_none());
    }

    // --- CreateParams with new fields ---

    #[test]
    fn create_params_with_new_fields_serializes() {
        let params = ChatCompletionCreateParams {
            model: ModelId::from("gpt-4o"),
            messages: vec![],
            service_tier: Some(ChatCompletionServiceTier::Flex),
            reasoning_effort: Some(ChatCompletionReasoningEffort::High),
            logit_bias: Some({
                let mut m = HashMap::new();
                m.insert("50256".to_owned(), -100);
                m
            }),
            modalities: Some(vec!["text".to_owned(), "audio".to_owned()]),
            verbosity: Some(ChatCompletionVerbosity::Low),
            ..Default::default()
        };
        let json = serde_json::to_value(&params).expect("serialize with new fields");
        assert_eq!(json["service_tier"], "flex");
        assert_eq!(json["reasoning_effort"], "high");
        assert_eq!(json["logit_bias"]["50256"], -100);
        assert_eq!(json["modalities"][0], "text");
        assert_eq!(json["modalities"][1], "audio");
        assert_eq!(json["verbosity"], "low");
    }
}
