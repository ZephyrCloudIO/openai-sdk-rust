//! Client and server event types for the Realtime WebSocket protocol,
//! plus all session configuration types ported from the Go SDK.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ===========================================================================
// Realtime model constants
// ===========================================================================

/// Known Realtime model identifiers.
pub mod realtime_model {
    pub const GPT_REALTIME: &str = "gpt-realtime";
    pub const GPT_REALTIME_1_5: &str = "gpt-realtime-1.5";
    pub const GPT_REALTIME_2025_08_28: &str = "gpt-realtime-2025-08-28";
    pub const GPT_4O_REALTIME_PREVIEW: &str = "gpt-4o-realtime-preview";
    pub const GPT_4O_REALTIME_PREVIEW_2024_10_01: &str = "gpt-4o-realtime-preview-2024-10-01";
    pub const GPT_4O_REALTIME_PREVIEW_2024_12_17: &str = "gpt-4o-realtime-preview-2024-12-17";
    pub const GPT_4O_REALTIME_PREVIEW_2025_06_03: &str = "gpt-4o-realtime-preview-2025-06-03";
    pub const GPT_4O_MINI_REALTIME_PREVIEW: &str = "gpt-4o-mini-realtime-preview";
    pub const GPT_4O_MINI_REALTIME_PREVIEW_2024_12_17: &str =
        "gpt-4o-mini-realtime-preview-2024-12-17";
    pub const GPT_REALTIME_MINI: &str = "gpt-realtime-mini";
    pub const GPT_REALTIME_MINI_2025_10_06: &str = "gpt-realtime-mini-2025-10-06";
    pub const GPT_REALTIME_MINI_2025_12_15: &str = "gpt-realtime-mini-2025-12-15";
    pub const GPT_AUDIO_1_5: &str = "gpt-audio-1.5";
    pub const GPT_AUDIO_MINI: &str = "gpt-audio-mini";
    pub const GPT_AUDIO_MINI_2025_10_06: &str = "gpt-audio-mini-2025-10-06";
    pub const GPT_AUDIO_MINI_2025_12_15: &str = "gpt-audio-mini-2025-12-15";
}

// ===========================================================================
// Audio transcription
// ===========================================================================

/// Model used for audio transcription.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudioTranscriptionModel {
    #[serde(rename = "whisper-1")]
    Whisper1,
    #[serde(rename = "gpt-4o-mini-transcribe")]
    Gpt4oMiniTranscribe,
    #[serde(rename = "gpt-4o-mini-transcribe-2025-12-15")]
    Gpt4oMiniTranscribe2025_12_15,
    #[serde(rename = "gpt-4o-transcribe")]
    Gpt4oTranscribe,
    #[serde(rename = "gpt-4o-transcribe-diarize")]
    Gpt4oTranscribeDiarize,
}

/// Configuration for input audio transcription.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioTranscriptionParam {
    /// The language of the input audio in ISO-639-1 format (e.g. `"en"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// The model to use for transcription.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<AudioTranscriptionModel>,
    /// An optional text to guide the model's style or continue a previous audio segment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}

// ===========================================================================
// Noise reduction
// ===========================================================================

/// Type of noise reduction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoiseReductionType {
    /// For close-talking microphones such as headphones.
    NearField,
    /// For far-field microphones such as laptop or conference room microphones.
    FarField,
}

/// Configuration for input audio noise reduction.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NoiseReductionParam {
    /// Type of noise reduction.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noise_type: Option<NoiseReductionType>,
}

// ===========================================================================
// Audio format types
// ===========================================================================

/// Audio format for input or output audio.
///
/// In the Go SDK these are represented as a union of `audio/pcm`, `audio/pcmu`,
/// and `audio/pcma` objects. Here we use an enum for idiomatic Rust.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum RealtimeAudioFormat {
    /// PCM 16-bit audio. Only 24kHz sample rate is supported.
    #[serde(rename = "audio/pcm")]
    Pcm {
        /// The sample rate. Always `24000`.
        #[serde(default = "default_pcm_rate")]
        rate: i64,
    },
    /// G.711 u-law format.
    #[serde(rename = "audio/pcmu")]
    Pcmu,
    /// G.711 A-law format.
    #[serde(rename = "audio/pcma")]
    Pcma,
}

fn default_pcm_rate() -> i64 {
    24000
}

impl Default for RealtimeAudioFormat {
    fn default() -> Self {
        Self::Pcm {
            rate: default_pcm_rate(),
        }
    }
}

// ===========================================================================
// Turn detection
// ===========================================================================

/// Eagerness level for semantic VAD.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VadEagerness {
    Low,
    Medium,
    High,
    Auto,
}

/// Turn detection configuration: Server VAD or Semantic VAD.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TurnDetection {
    /// Server-side voice activity detection based on audio volume.
    #[serde(rename = "server_vad")]
    ServerVad {
        /// Activation threshold for VAD (0.0 to 1.0). Defaults to 0.5.
        #[serde(skip_serializing_if = "Option::is_none")]
        threshold: Option<f64>,
        /// Amount of audio to include before VAD detected speech (ms). Defaults to 300.
        #[serde(skip_serializing_if = "Option::is_none")]
        prefix_padding_ms: Option<i64>,
        /// Duration of silence to detect speech stop (ms). Defaults to 500.
        #[serde(skip_serializing_if = "Option::is_none")]
        silence_duration_ms: Option<i64>,
        /// Whether to automatically generate a response on VAD stop.
        #[serde(skip_serializing_if = "Option::is_none")]
        create_response: Option<bool>,
        /// Whether to automatically interrupt ongoing response on VAD start.
        #[serde(skip_serializing_if = "Option::is_none")]
        interrupt_response: Option<bool>,
        /// Timeout (ms) after which a response is triggered automatically.
        #[serde(skip_serializing_if = "Option::is_none")]
        idle_timeout_ms: Option<i64>,
    },
    /// Semantic turn detection using a model to determine when the user is done speaking.
    #[serde(rename = "semantic_vad")]
    SemanticVad {
        /// The eagerness of the model to respond.
        #[serde(skip_serializing_if = "Option::is_none")]
        eagerness: Option<VadEagerness>,
        /// Whether to automatically generate a response on VAD stop.
        #[serde(skip_serializing_if = "Option::is_none")]
        create_response: Option<bool>,
        /// Whether to automatically interrupt ongoing response on VAD start.
        #[serde(skip_serializing_if = "Option::is_none")]
        interrupt_response: Option<bool>,
    },
}

// ===========================================================================
// Audio input / output configuration
// ===========================================================================

/// Configuration for input audio.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealtimeAudioInputConfig {
    /// Turn detection configuration. Set to `None` to disable (client must manually trigger).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_detection: Option<TurnDetection>,
    /// The format of input audio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<RealtimeAudioFormat>,
    /// Configuration for noise reduction. Set to `None` to disable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noise_reduction: Option<NoiseReductionParam>,
    /// Configuration for input audio transcription.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcription: Option<AudioTranscriptionParam>,
}

/// Built-in voice names for the Realtime API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BuiltInVoice {
    Alloy,
    Ash,
    Ballad,
    Coral,
    Echo,
    Sage,
    Shimmer,
    Verse,
    Marin,
    Cedar,
}

/// A custom voice reference with an ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeVoiceId {
    /// The custom voice ID, e.g. `"voice_1234"`.
    pub id: String,
}

/// Voice configuration: a built-in voice name, a string, or a custom voice ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeVoice {
    /// A built-in voice.
    BuiltIn(BuiltInVoice),
    /// A custom voice referenced by ID object.
    Custom(RealtimeVoiceId),
    /// An arbitrary voice string.
    String(String),
}

/// Configuration for output audio.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealtimeAudioOutputConfig {
    /// Playback speed multiplier (0.25 to 1.5). Default 1.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    /// The format of output audio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<RealtimeAudioFormat>,
    /// The voice the model uses to respond.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<RealtimeVoice>,
}

/// Configuration for input and output audio.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealtimeAudioConfig {
    /// Input audio configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<RealtimeAudioInputConfig>,
    /// Output audio configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<RealtimeAudioOutputConfig>,
}

// ===========================================================================
// Tool types
// ===========================================================================

/// A function tool available to the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeFunctionTool {
    /// The type of the tool. Always `"function"`.
    #[serde(rename = "type")]
    #[serde(default = "default_function_type")]
    pub tool_type: String,
    /// The name of the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Description of the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Parameters of the function in JSON Schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

fn default_function_type() -> String {
    "function".to_owned()
}

/// Filter for allowed MCP tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolFilter {
    /// Whether to allow only read-only tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
    /// Specific tool names to allow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_names: Option<Vec<String>>,
}

/// Allowed tools for an MCP server: a list of names or a filter object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpAllowedTools {
    /// A list of allowed tool names.
    Names(Vec<String>),
    /// A filter object.
    Filter(McpToolFilter),
}

/// Approval filter for MCP tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolApprovalFilter {
    /// Tools that always require approval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub always: Option<McpToolFilter>,
    /// Tools that never require approval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub never: Option<McpToolFilter>,
}

/// Approval configuration for MCP tools: a string setting or a filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpRequireApproval {
    /// A policy string: `"always"` or `"never"`.
    Setting(String),
    /// A filter specifying per-tool approval rules.
    Filter(McpToolApprovalFilter),
}

/// An MCP tool server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeMcpTool {
    /// The type of tool. Always `"mcp"`.
    #[serde(rename = "type")]
    #[serde(default = "default_mcp_type")]
    pub tool_type: String,
    /// A label for this MCP server.
    pub server_label: String,
    /// The URL for the MCP server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_url: Option<String>,
    /// Description of the MCP server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_description: Option<String>,
    /// Allowed tools from this server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<McpAllowedTools>,
    /// Approval policy for tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<McpRequireApproval>,
    /// Optional HTTP headers to send to the MCP server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    /// OAuth access token for the MCP server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<String>,
    /// Service connector identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_id: Option<String>,
    /// Whether this tool is deferred and discovered via tool search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defer_loading: Option<bool>,
}

fn default_mcp_type() -> String {
    "mcp".to_owned()
}

/// A tool available to the Realtime model: function or MCP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RealtimeTool {
    /// A function tool.
    #[serde(rename = "function")]
    Function {
        /// The name of the function.
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        /// Description of the function.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// Parameters of the function in JSON Schema.
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<serde_json::Value>,
    },
    /// An MCP tool server.
    #[serde(rename = "mcp")]
    Mcp {
        /// A label for this MCP server.
        server_label: String,
        /// The URL for the MCP server.
        #[serde(skip_serializing_if = "Option::is_none")]
        server_url: Option<String>,
        /// Description of the MCP server.
        #[serde(skip_serializing_if = "Option::is_none")]
        server_description: Option<String>,
        /// Allowed tools from this server.
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_tools: Option<McpAllowedTools>,
        /// Approval policy for tools.
        #[serde(skip_serializing_if = "Option::is_none")]
        require_approval: Option<McpRequireApproval>,
        /// Optional HTTP headers.
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
        /// OAuth access token.
        #[serde(skip_serializing_if = "Option::is_none")]
        authorization: Option<String>,
        /// Service connector identifier.
        #[serde(skip_serializing_if = "Option::is_none")]
        connector_id: Option<String>,
        /// Whether deferred.
        #[serde(skip_serializing_if = "Option::is_none")]
        defer_loading: Option<bool>,
    },
}

// ===========================================================================
// Tool choice
// ===========================================================================

/// How the model chooses tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeToolChoice {
    /// A mode string: `"auto"`, `"none"`, or `"required"`.
    Mode(String),
    /// Force a specific function tool.
    Function(ToolChoiceFunction),
    /// Force a specific MCP tool.
    Mcp(ToolChoiceMcp),
}

/// Force the model to use a specific function tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceFunction {
    /// The type. Always `"function"`.
    #[serde(rename = "type")]
    pub choice_type: String,
    /// Name of the function to call.
    pub name: String,
}

/// Force the model to use a specific MCP tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceMcp {
    /// The type. Always `"mcp"`.
    #[serde(rename = "type")]
    pub choice_type: String,
    /// Label of the MCP server.
    pub server_label: String,
    /// Optional specific tool name on that server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// ===========================================================================
// Max output tokens
// ===========================================================================

/// Maximum output tokens: an integer limit or `"inf"` for maximum available.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaxOutputTokens {
    /// A specific integer limit.
    Int(i64),
    /// The string `"inf"` for maximum available tokens.
    Inf(String),
}

impl Default for MaxOutputTokens {
    fn default() -> Self {
        Self::Inf("inf".to_owned())
    }
}

// ===========================================================================
// Tracing configuration
// ===========================================================================

/// Granular tracing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfiguration {
    /// The group ID for filtering in the Traces Dashboard.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    /// The workflow name for naming the trace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_name: Option<String>,
    /// Arbitrary metadata for filtering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Tracing config: `"auto"` or a detailed configuration object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeTracingConfig {
    /// Automatic tracing with default values.
    Auto(String),
    /// Granular tracing configuration.
    Configuration(TracingConfiguration),
}

// ===========================================================================
// Truncation configuration
// ===========================================================================

/// Token limits for retention-ratio truncation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruncationTokenLimits {
    /// Max tokens in the conversation after instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_instructions: Option<i64>,
}

/// Retention-ratio truncation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionRatioTruncation {
    /// The type. Always `"retention_ratio"`.
    #[serde(rename = "type")]
    #[serde(default = "default_retention_ratio_type")]
    pub truncation_type: String,
    /// Fraction of conversation tokens to retain (0.0 - 1.0).
    pub retention_ratio: f64,
    /// Optional custom token limits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_limits: Option<TruncationTokenLimits>,
}

fn default_retention_ratio_type() -> String {
    "retention_ratio".to_owned()
}

/// Truncation strategy: a string mode or a retention-ratio object.
///
/// String modes: `"auto"` (default) or `"disabled"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeTruncation {
    /// A strategy string: `"auto"` or `"disabled"`.
    Strategy(String),
    /// Retention-ratio truncation with optional token limits.
    RetentionRatio(RetentionRatioTruncation),
}

// ===========================================================================
// Prompt reference
// ===========================================================================

/// Reference to a prompt template and its variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsePromptParam {
    /// The prompt template ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Template variable values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,
    /// The version of the prompt template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

// ===========================================================================
// RealtimeSessionCreateRequestParam — the master session config type
// ===========================================================================

/// Realtime session object configuration.
///
/// This is the main parameter type for creating/updating a realtime session.
/// It matches the Go SDK's `RealtimeSessionCreateRequestParam`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealtimeSessionCreateRequestParam {
    /// The type of session. Always `"realtime"` for the Realtime API.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_type: Option<String>,

    /// The Realtime model used for this session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// The default system instructions prepended to model calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// Reference to a prompt template and its variables.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<ResponsePromptParam>,

    /// The set of modalities the model can respond with.
    /// Defaults to `["audio"]`. Use `["text"]` for text-only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_modalities: Option<Vec<String>>,

    /// Configuration for input and output audio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<RealtimeAudioConfig>,

    /// Tools available to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<RealtimeTool>>,

    /// How the model chooses tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<RealtimeToolChoice>,

    /// Maximum output tokens for a single assistant response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<MaxOutputTokens>,

    /// Tracing configuration. Set to `None` to disable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracing: Option<RealtimeTracingConfig>,

    /// Truncation configuration for managing context window.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<RealtimeTruncation>,

    /// Additional fields to include in server outputs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}

impl RealtimeSessionCreateRequestParam {
    /// Creates a new empty session configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the model.
    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Sets the instructions.
    #[must_use]
    pub fn with_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Sets the output modalities.
    #[must_use]
    pub fn with_output_modalities(mut self, modalities: Vec<String>) -> Self {
        self.output_modalities = Some(modalities);
        self
    }

    /// Sets the audio configuration.
    #[must_use]
    pub fn with_audio(mut self, audio: RealtimeAudioConfig) -> Self {
        self.audio = Some(audio);
        self
    }

    /// Sets the tools.
    #[must_use]
    pub fn with_tools(mut self, tools: Vec<RealtimeTool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Sets the tool choice.
    #[must_use]
    pub fn with_tool_choice(mut self, choice: RealtimeToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    /// Sets max output tokens to a specific integer.
    #[must_use]
    pub fn with_max_output_tokens(mut self, tokens: i64) -> Self {
        self.max_output_tokens = Some(MaxOutputTokens::Int(tokens));
        self
    }

    /// Sets max output tokens to infinite.
    #[must_use]
    pub fn with_max_output_tokens_inf(mut self) -> Self {
        self.max_output_tokens = Some(MaxOutputTokens::Inf("inf".to_owned()));
        self
    }

    /// Sets the tracing configuration.
    #[must_use]
    pub fn with_tracing(mut self, tracing: RealtimeTracingConfig) -> Self {
        self.tracing = Some(tracing);
        self
    }

    /// Sets the truncation configuration.
    #[must_use]
    pub fn with_truncation(mut self, truncation: RealtimeTruncation) -> Self {
        self.truncation = Some(truncation);
        self
    }
}

// ---------------------------------------------------------------------------
// Client Events (sent by the SDK to the server)
// ---------------------------------------------------------------------------

/// An event sent from the client to the Realtime API server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientEvent {
    /// Update session configuration.
    #[serde(rename = "session.update")]
    SessionUpdate {
        /// Session configuration object.
        session: serde_json::Value,
        /// Optional client-generated event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    /// Append audio bytes (base64-encoded) to the input buffer.
    #[serde(rename = "input_audio_buffer.append")]
    InputAudioBufferAppend {
        /// Base64-encoded audio data.
        audio: String,
        /// Optional client-generated event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    /// Commit the current input audio buffer.
    #[serde(rename = "input_audio_buffer.commit")]
    InputAudioBufferCommit {
        /// Optional client-generated event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    /// Clear the input audio buffer.
    #[serde(rename = "input_audio_buffer.clear")]
    InputAudioBufferClear {
        /// Optional client-generated event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    /// Create a conversation item.
    #[serde(rename = "conversation.item.create")]
    ConversationItemCreate {
        /// The item to create.
        item: serde_json::Value,
        /// Optional client-generated event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    /// Truncate a conversation item's audio.
    #[serde(rename = "conversation.item.truncate")]
    ConversationItemTruncate {
        /// ID of the item to truncate.
        item_id: String,
        /// Index of the content part to truncate.
        content_index: u32,
        /// Number of audio samples to keep.
        audio_end_ms: u32,
        /// Optional client-generated event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    /// Delete a conversation item.
    #[serde(rename = "conversation.item.delete")]
    ConversationItemDelete {
        /// ID of the item to delete.
        item_id: String,
        /// Optional client-generated event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    /// Trigger a model response.
    #[serde(rename = "response.create")]
    ResponseCreate {
        /// Optional response configuration.
        #[serde(skip_serializing_if = "Option::is_none")]
        response: Option<serde_json::Value>,
        /// Optional client-generated event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    /// Cancel an in-progress response.
    #[serde(rename = "response.cancel")]
    ResponseCancel {
        /// Optional client-generated event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },
}

// ---------------------------------------------------------------------------
// Server Events (received from the server)
// ---------------------------------------------------------------------------

/// An event received from the Realtime API server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerEvent {
    // -- Session events --
    /// Session was created.
    #[serde(rename = "session.created")]
    SessionCreated {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Session object.
        session: serde_json::Value,
    },

    /// Session configuration was updated.
    #[serde(rename = "session.updated")]
    SessionUpdated {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Updated session object.
        session: serde_json::Value,
    },

    // -- Conversation events --
    /// A conversation was created.
    #[serde(rename = "conversation.created")]
    ConversationCreated {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// The conversation object.
        conversation: serde_json::Value,
    },

    /// A conversation item was created.
    #[serde(rename = "conversation.item.created")]
    ConversationItemCreated {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// The created item.
        item: serde_json::Value,
    },

    /// Input audio transcription completed for a conversation item.
    #[serde(rename = "conversation.item.input_audio_transcription.completed")]
    ConversationItemInputAudioTranscriptionCompleted {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// ID of the item.
        item_id: String,
        /// Index of the content part.
        content_index: u32,
        /// The transcription text.
        transcript: String,
    },

    /// Input audio transcription failed for a conversation item.
    #[serde(rename = "conversation.item.input_audio_transcription.failed")]
    ConversationItemInputAudioTranscriptionFailed {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// ID of the item.
        item_id: String,
        /// Index of the content part.
        content_index: u32,
        /// Error details.
        error: serde_json::Value,
    },

    /// A conversation item was truncated.
    #[serde(rename = "conversation.item.truncated")]
    ConversationItemTruncated {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// ID of the item that was truncated.
        item_id: String,
        /// Index of the content part.
        content_index: u32,
        /// Audio end time in milliseconds.
        audio_end_ms: u64,
    },

    /// A conversation item was deleted.
    #[serde(rename = "conversation.item.deleted")]
    ConversationItemDeleted {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// ID of the deleted item.
        item_id: String,
    },

    // -- Input audio buffer events --
    /// Input audio buffer was committed.
    #[serde(rename = "input_audio_buffer.committed")]
    InputAudioBufferCommitted {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// ID of the previous item.
        #[serde(skip_serializing_if = "Option::is_none")]
        previous_item_id: Option<String>,
        /// ID of the new item.
        item_id: String,
    },

    /// Input audio buffer was cleared.
    #[serde(rename = "input_audio_buffer.cleared")]
    InputAudioBufferCleared {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    /// Speech was detected in the input audio buffer.
    #[serde(rename = "input_audio_buffer.speech_started")]
    InputAudioBufferSpeechStarted {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Millisecond offset where speech was detected.
        audio_start_ms: u64,
        /// ID of the item being created.
        item_id: String,
    },

    /// Speech ended in the input audio buffer.
    #[serde(rename = "input_audio_buffer.speech_stopped")]
    InputAudioBufferSpeechStopped {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Millisecond offset where speech stopped.
        audio_end_ms: u64,
        /// ID of the item.
        item_id: String,
    },

    /// Input audio buffer idle timeout was triggered.
    #[serde(rename = "input_audio_buffer.timeout_triggered")]
    InputAudioBufferTimeoutTriggered {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// ID of the item.
        item_id: String,
    },

    // -- Response lifecycle events --
    /// A response was created.
    #[serde(rename = "response.created")]
    ResponseCreated {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// The response object.
        response: serde_json::Value,
    },

    /// A response completed.
    #[serde(rename = "response.done")]
    ResponseDone {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// The completed response object.
        response: serde_json::Value,
    },

    /// An output item was added to a response.
    #[serde(rename = "response.output_item.added")]
    ResponseOutputItemAdded {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Response ID.
        response_id: String,
        /// Output index.
        output_index: u32,
        /// The item.
        item: serde_json::Value,
    },

    /// An output item in a response is done.
    #[serde(rename = "response.output_item.done")]
    ResponseOutputItemDone {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Response ID.
        response_id: String,
        /// Output index.
        output_index: u32,
        /// The completed item.
        item: serde_json::Value,
    },

    // -- Content part events --
    /// A content part was added.
    #[serde(rename = "response.content_part.added")]
    ResponseContentPartAdded {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Response ID.
        response_id: String,
        /// Item ID.
        item_id: String,
        /// Output index.
        output_index: u32,
        /// Content index.
        content_index: u32,
        /// The content part.
        part: serde_json::Value,
    },

    /// A content part is done.
    #[serde(rename = "response.content_part.done")]
    ResponseContentPartDone {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Response ID.
        response_id: String,
        /// Item ID.
        item_id: String,
        /// Output index.
        output_index: u32,
        /// Content index.
        content_index: u32,
        /// The completed content part.
        part: serde_json::Value,
    },

    // -- Text streaming events --
    /// A text delta in a response.
    #[serde(rename = "response.text.delta")]
    ResponseTextDelta {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Response ID.
        response_id: String,
        /// Item ID.
        item_id: String,
        /// Output index.
        output_index: u32,
        /// Content index.
        content_index: u32,
        /// The text delta.
        delta: String,
    },

    /// Text streaming for this content part is done.
    #[serde(rename = "response.text.done")]
    ResponseTextDone {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Response ID.
        response_id: String,
        /// Item ID.
        item_id: String,
        /// Output index.
        output_index: u32,
        /// Content index.
        content_index: u32,
        /// The complete text.
        text: String,
    },

    // -- Audio transcript streaming events --
    /// An audio transcript delta.
    #[serde(rename = "response.audio_transcript.delta")]
    ResponseAudioTranscriptDelta {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Response ID.
        response_id: String,
        /// Item ID.
        item_id: String,
        /// Output index.
        output_index: u32,
        /// Content index.
        content_index: u32,
        /// The transcript delta.
        delta: String,
    },

    /// Audio transcript streaming is done.
    #[serde(rename = "response.audio_transcript.done")]
    ResponseAudioTranscriptDone {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Response ID.
        response_id: String,
        /// Item ID.
        item_id: String,
        /// Output index.
        output_index: u32,
        /// Content index.
        content_index: u32,
        /// The complete transcript.
        transcript: String,
    },

    // -- Audio streaming events --
    /// An audio delta (base64-encoded).
    #[serde(rename = "response.audio.delta")]
    ResponseAudioDelta {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Response ID.
        response_id: String,
        /// Item ID.
        item_id: String,
        /// Output index.
        output_index: u32,
        /// Content index.
        content_index: u32,
        /// Base64-encoded audio data.
        delta: String,
    },

    /// Audio streaming is done.
    #[serde(rename = "response.audio.done")]
    ResponseAudioDone {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Response ID.
        response_id: String,
        /// Item ID.
        item_id: String,
        /// Output index.
        output_index: u32,
        /// Content index.
        content_index: u32,
    },

    // -- Function call argument events --
    /// A delta for function call arguments.
    #[serde(rename = "response.function_call_arguments.delta")]
    ResponseFunctionCallArgumentsDelta {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Response ID.
        response_id: String,
        /// Item ID.
        item_id: String,
        /// Output index.
        output_index: u32,
        /// The tool call ID.
        call_id: String,
        /// The arguments delta.
        delta: String,
    },

    /// Function call arguments are done.
    #[serde(rename = "response.function_call_arguments.done")]
    ResponseFunctionCallArgumentsDone {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Response ID.
        response_id: String,
        /// Item ID.
        item_id: String,
        /// Output index.
        output_index: u32,
        /// The tool call ID.
        call_id: String,
        /// The complete arguments string.
        arguments: String,
    },

    // -- Rate limits --
    /// Rate limits were updated.
    #[serde(rename = "rate_limits.updated")]
    RateLimitsUpdated {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Updated rate limit information.
        rate_limits: Vec<RateLimitInfo>,
    },

    // -- Error --
    /// An error occurred.
    #[serde(rename = "error")]
    Error {
        /// Server-assigned event ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Error details.
        error: serde_json::Value,
    },
}

/// Rate limit information returned in `rate_limits.updated` events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Rate limit name (e.g. "requests", "tokens").
    pub name: String,
    /// Maximum allowed.
    pub limit: u64,
    /// Remaining in the current window.
    pub remaining: u64,
    /// Seconds until the limit resets.
    pub reset_seconds: f64,
}
