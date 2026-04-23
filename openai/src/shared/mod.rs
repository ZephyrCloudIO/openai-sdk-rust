//! Shared reusable SDK types.

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// ModelId newtype
// ---------------------------------------------------------------------------

/// Newtype wrapper for model IDs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ModelId(pub String);

impl ModelId {
    /// Creates a model ID from any owned or borrowed string input.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for ModelId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for ModelId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for ModelId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

/// Arbitrary string key-value metadata attached to API objects.
pub type Metadata = HashMap<String, String>;

// ---------------------------------------------------------------------------
// OAuthErrorCode
// ---------------------------------------------------------------------------

/// OAuth error codes returned by token exchange flows.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OAuthErrorCode {
    /// The grant is invalid.
    InvalidGrant,
    /// The subject token is invalid.
    InvalidSubjectToken,
    /// An unrecognized OAuth error code.
    Other(String),
}

impl OAuthErrorCode {
    /// Returns the wire string for this OAuth error code.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::InvalidGrant => "invalid_grant",
            Self::InvalidSubjectToken => "invalid_subject_token",
            Self::Other(value) => value.as_str(),
        }
    }
}

impl std::fmt::Display for OAuthErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl serde::Serialize for OAuthErrorCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for OAuthErrorCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "invalid_grant" => Self::InvalidGrant,
            "invalid_subject_token" => Self::InvalidSubjectToken,
            _ => Self::Other(value),
        })
    }
}

// ---------------------------------------------------------------------------
// FinishReason
// ---------------------------------------------------------------------------

/// Common completion finish reasons.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Natural stop.
    Stop,
    /// Max tokens hit.
    Length,
    /// Content policy filter.
    ContentFilter,
    /// Tool call stop.
    ToolCalls,
    /// Legacy function call stop.
    FunctionCall,
}

// ---------------------------------------------------------------------------
// Model constants
// ---------------------------------------------------------------------------

/// Chat model ID constants matching the Go SDK's `ChatModel` values.
pub mod chat_model {
    // GPT-4o family
    pub const GPT_4O: &str = "gpt-4o";
    pub const GPT_4O_2024_11_20: &str = "gpt-4o-2024-11-20";
    pub const GPT_4O_2024_08_06: &str = "gpt-4o-2024-08-06";
    pub const GPT_4O_2024_05_13: &str = "gpt-4o-2024-05-13";
    pub const GPT_4O_AUDIO_PREVIEW: &str = "gpt-4o-audio-preview";
    pub const GPT_4O_AUDIO_PREVIEW_2024_10_01: &str = "gpt-4o-audio-preview-2024-10-01";
    pub const GPT_4O_AUDIO_PREVIEW_2024_12_17: &str = "gpt-4o-audio-preview-2024-12-17";
    pub const GPT_4O_AUDIO_PREVIEW_2025_06_03: &str = "gpt-4o-audio-preview-2025-06-03";
    pub const GPT_4O_MINI_AUDIO_PREVIEW: &str = "gpt-4o-mini-audio-preview";
    pub const GPT_4O_MINI_AUDIO_PREVIEW_2024_12_17: &str = "gpt-4o-mini-audio-preview-2024-12-17";
    pub const GPT_4O_SEARCH_PREVIEW: &str = "gpt-4o-search-preview";
    pub const GPT_4O_MINI_SEARCH_PREVIEW: &str = "gpt-4o-mini-search-preview";
    pub const GPT_4O_SEARCH_PREVIEW_2025_03_11: &str = "gpt-4o-search-preview-2025-03-11";
    pub const GPT_4O_MINI_SEARCH_PREVIEW_2025_03_11: &str = "gpt-4o-mini-search-preview-2025-03-11";
    pub const CHATGPT_4O_LATEST: &str = "chatgpt-4o-latest";
    pub const GPT_4O_MINI: &str = "gpt-4o-mini";
    pub const GPT_4O_MINI_2024_07_18: &str = "gpt-4o-mini-2024-07-18";

    // GPT-4.1 family
    pub const GPT_4_1: &str = "gpt-4.1";
    pub const GPT_4_1_MINI: &str = "gpt-4.1-mini";
    pub const GPT_4_1_NANO: &str = "gpt-4.1-nano";
    pub const GPT_4_1_2025_04_14: &str = "gpt-4.1-2025-04-14";
    pub const GPT_4_1_MINI_2025_04_14: &str = "gpt-4.1-mini-2025-04-14";
    pub const GPT_4_1_NANO_2025_04_14: &str = "gpt-4.1-nano-2025-04-14";

    // GPT-4 Turbo
    pub const GPT_4_TURBO: &str = "gpt-4-turbo";
    pub const GPT_4_TURBO_2024_04_09: &str = "gpt-4-turbo-2024-04-09";
    pub const GPT_4_0125_PREVIEW: &str = "gpt-4-0125-preview";
    pub const GPT_4_TURBO_PREVIEW: &str = "gpt-4-turbo-preview";
    pub const GPT_4_1106_PREVIEW: &str = "gpt-4-1106-preview";
    pub const GPT_4_VISION_PREVIEW: &str = "gpt-4-vision-preview";

    // GPT-4
    pub const GPT_4: &str = "gpt-4";
    pub const GPT_4_0314: &str = "gpt-4-0314";
    pub const GPT_4_0613: &str = "gpt-4-0613";
    pub const GPT_4_32K: &str = "gpt-4-32k";
    pub const GPT_4_32K_0314: &str = "gpt-4-32k-0314";
    pub const GPT_4_32K_0613: &str = "gpt-4-32k-0613";

    // GPT-3.5
    pub const GPT_3_5_TURBO: &str = "gpt-3.5-turbo";
    pub const GPT_3_5_TURBO_16K: &str = "gpt-3.5-turbo-16k";
    pub const GPT_3_5_TURBO_0301: &str = "gpt-3.5-turbo-0301";
    pub const GPT_3_5_TURBO_0613: &str = "gpt-3.5-turbo-0613";
    pub const GPT_3_5_TURBO_1106: &str = "gpt-3.5-turbo-1106";
    pub const GPT_3_5_TURBO_0125: &str = "gpt-3.5-turbo-0125";
    pub const GPT_3_5_TURBO_16K_0613: &str = "gpt-3.5-turbo-16k-0613";

    // GPT-5 family
    pub const GPT_5: &str = "gpt-5";
    pub const GPT_5_MINI: &str = "gpt-5-mini";
    pub const GPT_5_NANO: &str = "gpt-5-nano";
    pub const GPT_5_2025_08_07: &str = "gpt-5-2025-08-07";
    pub const GPT_5_MINI_2025_08_07: &str = "gpt-5-mini-2025-08-07";
    pub const GPT_5_NANO_2025_08_07: &str = "gpt-5-nano-2025-08-07";
    pub const GPT_5_CHAT_LATEST: &str = "gpt-5-chat-latest";
    pub const GPT_5_1: &str = "gpt-5.1";
    pub const GPT_5_1_2025_11_13: &str = "gpt-5.1-2025-11-13";
    pub const GPT_5_1_CODEX: &str = "gpt-5.1-codex";
    pub const GPT_5_1_MINI: &str = "gpt-5.1-mini";
    pub const GPT_5_1_CHAT_LATEST: &str = "gpt-5.1-chat-latest";
    pub const GPT_5_2: &str = "gpt-5.2";
    pub const GPT_5_2_2025_12_11: &str = "gpt-5.2-2025-12-11";
    pub const GPT_5_2_CHAT_LATEST: &str = "gpt-5.2-chat-latest";
    pub const GPT_5_2_PRO: &str = "gpt-5.2-pro";
    pub const GPT_5_2_PRO_2025_12_11: &str = "gpt-5.2-pro-2025-12-11";
    pub const GPT_5_4: &str = "gpt-5.4";
    pub const GPT_5_4_MINI: &str = "gpt-5.4-mini";
    pub const GPT_5_4_NANO: &str = "gpt-5.4-nano";
    pub const GPT_5_4_MINI_2026_03_17: &str = "gpt-5.4-mini-2026-03-17";
    pub const GPT_5_4_NANO_2026_03_17: &str = "gpt-5.4-nano-2026-03-17";
    pub const GPT_5_3_CHAT_LATEST: &str = "gpt-5.3-chat-latest";

    // o-series
    pub const O1: &str = "o1";
    pub const O1_2024_12_17: &str = "o1-2024-12-17";
    pub const O1_PREVIEW: &str = "o1-preview";
    pub const O1_PREVIEW_2024_09_12: &str = "o1-preview-2024-09-12";
    pub const O1_MINI: &str = "o1-mini";
    pub const O1_MINI_2024_09_12: &str = "o1-mini-2024-09-12";
    pub const O3: &str = "o3";
    pub const O3_2025_04_16: &str = "o3-2025-04-16";
    pub const O3_MINI: &str = "o3-mini";
    pub const O3_MINI_2025_01_31: &str = "o3-mini-2025-01-31";
    pub const O4_MINI: &str = "o4-mini";
    pub const O4_MINI_2025_04_16: &str = "o4-mini-2025-04-16";

    // Codex
    pub const CODEX_MINI_LATEST: &str = "codex-mini-latest";
}

/// Responses API model ID constants matching the Go SDK's `ResponsesModel` values.
pub mod responses_model {
    pub const O1_PRO: &str = "o1-pro";
    pub const O1_PRO_2025_03_19: &str = "o1-pro-2025-03-19";
    pub const O3_PRO: &str = "o3-pro";
    pub const O3_PRO_2025_06_10: &str = "o3-pro-2025-06-10";
    pub const O3_DEEP_RESEARCH: &str = "o3-deep-research";
    pub const O3_DEEP_RESEARCH_2025_06_26: &str = "o3-deep-research-2025-06-26";
    pub const O4_MINI_DEEP_RESEARCH: &str = "o4-mini-deep-research";
    pub const O4_MINI_DEEP_RESEARCH_2025_06_26: &str = "o4-mini-deep-research-2025-06-26";
    pub const COMPUTER_USE_PREVIEW: &str = "computer-use-preview";
    pub const COMPUTER_USE_PREVIEW_2025_03_11: &str = "computer-use-preview-2025-03-11";
    pub const GPT_5_CODEX: &str = "gpt-5-codex";
    pub const GPT_5_PRO: &str = "gpt-5-pro";
    pub const GPT_5_PRO_2025_10_06: &str = "gpt-5-pro-2025-10-06";
    pub const GPT_5_1_CODEX_MAX: &str = "gpt-5.1-codex-max";
}

/// `AllModels` ID constants (superset covering both Chat and Responses).
pub mod all_models {
    pub const O1_PRO: &str = "o1-pro";
    pub const O1_PRO_2025_03_19: &str = "o1-pro-2025-03-19";
    pub const O3_PRO: &str = "o3-pro";
    pub const O3_PRO_2025_06_10: &str = "o3-pro-2025-06-10";
    pub const O3_DEEP_RESEARCH: &str = "o3-deep-research";
    pub const O3_DEEP_RESEARCH_2025_06_26: &str = "o3-deep-research-2025-06-26";
    pub const O4_MINI_DEEP_RESEARCH: &str = "o4-mini-deep-research";
    pub const O4_MINI_DEEP_RESEARCH_2025_06_26: &str = "o4-mini-deep-research-2025-06-26";
    pub const COMPUTER_USE_PREVIEW: &str = "computer-use-preview";
    pub const COMPUTER_USE_PREVIEW_2025_03_11: &str = "computer-use-preview-2025-03-11";
    pub const GPT_5_CODEX: &str = "gpt-5-codex";
    pub const GPT_5_PRO: &str = "gpt-5-pro";
    pub const GPT_5_PRO_2025_10_06: &str = "gpt-5-pro-2025-10-06";
    pub const GPT_5_1_CODEX_MAX: &str = "gpt-5.1-codex-max";
}

/// Embedding model ID constants.
pub mod embedding_model {
    pub const TEXT_EMBEDDING_ADA_002: &str = "text-embedding-ada-002";
    pub const TEXT_EMBEDDING_3_SMALL: &str = "text-embedding-3-small";
    pub const TEXT_EMBEDDING_3_LARGE: &str = "text-embedding-3-large";
}

// ---------------------------------------------------------------------------
// ComparisonFilter
// ---------------------------------------------------------------------------

/// Comparison operator for attribute filters.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonFilterType {
    /// Equals.
    Eq,
    /// Not equal.
    Ne,
    /// Greater than.
    Gt,
    /// Greater than or equal.
    Gte,
    /// Less than.
    Lt,
    /// Less than or equal.
    Lte,
    /// In (array membership).
    #[serde(rename = "in")]
    In,
    /// Not in (array non-membership).
    #[serde(rename = "nin")]
    NotIn,
}

/// A filter that compares a specified attribute key to a given value.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ComparisonFilter {
    /// The key to compare against the value.
    pub key: String,
    /// The comparison operator.
    #[serde(rename = "type")]
    pub filter_type: ComparisonFilterType,
    /// The value to compare (string, number, boolean, or array).
    pub value: serde_json::Value,
}

// ---------------------------------------------------------------------------
// CompoundFilter
// ---------------------------------------------------------------------------

/// Operation type for compound filters.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompoundFilterType {
    /// All child filters must match.
    And,
    /// At least one child filter must match.
    Or,
}

/// A union of [`ComparisonFilter`] and [`CompoundFilter`].
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum FilterUnion {
    /// A simple comparison filter.
    Comparison(ComparisonFilter),
    /// A nested compound filter.
    Compound(Box<CompoundFilter>),
}

impl FilterUnion {
    /// Returns the union value as JSON for dynamic matching.
    #[must_use]
    pub fn as_any(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Returns the comparison filter variant.
    #[must_use]
    pub fn as_comparison(&self) -> Option<&ComparisonFilter> {
        match self {
            Self::Comparison(value) => Some(value),
            Self::Compound(_) => None,
        }
    }

    /// Returns the compound filter variant.
    #[must_use]
    pub fn as_compound(&self) -> Option<&CompoundFilter> {
        match self {
            Self::Comparison(_) => None,
            Self::Compound(value) => Some(value),
        }
    }

    /// Creates a comparison filter union variant.
    #[must_use]
    pub fn param_of_comparison(value: ComparisonFilter) -> Self {
        Self::Comparison(value)
    }

    /// Creates a compound filter union variant.
    #[must_use]
    pub fn param_of_compound(value: CompoundFilter) -> Self {
        Self::Compound(Box::new(value))
    }
}

/// Combines multiple filters using `and` or `or`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CompoundFilter {
    /// Array of child filters (comparison or compound).
    pub filters: Vec<FilterUnion>,
    /// The boolean operation.
    #[serde(rename = "type")]
    pub filter_type: CompoundFilterType,
}

// ---------------------------------------------------------------------------
// Reasoning
// ---------------------------------------------------------------------------

/// Constrains effort on reasoning for reasoning models.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningEffort {
    /// No reasoning (supported on gpt-5.1+).
    None,
    /// Minimal reasoning.
    Minimal,
    /// Low reasoning effort.
    Low,
    /// Medium reasoning effort (default for most o-series).
    Medium,
    /// High reasoning effort.
    High,
    /// Extra-high reasoning (supported on gpt-5.1-codex-max+).
    Xhigh,
}

/// Controls whether and how reasoning summaries are generated.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningSummary {
    /// Automatic summary generation.
    Auto,
    /// Concise summary.
    Concise,
    /// Detailed summary.
    Detailed,
}

/// Configuration options for reasoning models.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Reasoning {
    /// Reasoning effort level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,
    /// Reasoning summary mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ReasoningSummary>,
}

// ---------------------------------------------------------------------------
// Response format types (shared)
// ---------------------------------------------------------------------------

/// Default text response format.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResponseFormatText {
    /// Always `"text"`.
    #[serde(rename = "type")]
    pub format_type: ResponseFormatTextType,
}

/// Marker type for [`ResponseFormatText`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResponseFormatTextType {
    #[serde(rename = "text")]
    Text,
}

impl Default for ResponseFormatText {
    fn default() -> Self {
        Self {
            format_type: ResponseFormatTextType::Text,
        }
    }
}

/// JSON object response format.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResponseFormatJsonObject {
    /// Always `"json_object"`.
    #[serde(rename = "type")]
    pub format_type: ResponseFormatJsonObjectType,
}

/// Marker type for [`ResponseFormatJsonObject`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResponseFormatJsonObjectType {
    #[serde(rename = "json_object")]
    JsonObject,
}

impl Default for ResponseFormatJsonObject {
    fn default() -> Self {
        Self {
            format_type: ResponseFormatJsonObjectType::JsonObject,
        }
    }
}

/// JSON Schema response format for structured outputs.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResponseFormatJsonSchema {
    /// Always `"json_schema"`.
    #[serde(rename = "type")]
    pub format_type: ResponseFormatJsonSchemaType,
    /// The JSON Schema configuration.
    pub json_schema: ResponseFormatJsonSchemaConfig,
}

/// Marker type for [`ResponseFormatJsonSchema`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResponseFormatJsonSchemaType {
    #[serde(rename = "json_schema")]
    JsonSchema,
}

/// Configuration for structured JSON Schema outputs.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResponseFormatJsonSchemaConfig {
    /// The name of the response format (a-z, A-Z, 0-9, underscores, dashes; max 64 chars).
    pub name: String,
    /// Optional description of the format purpose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The JSON Schema object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    /// Whether to enforce strict schema adherence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

// ---------------------------------------------------------------------------
// ErrorObject
// ---------------------------------------------------------------------------

/// Structured API error detail returned in error responses.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ErrorObject {
    /// Machine-readable error code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Human-readable error message.
    pub message: String,
    /// The parameter that caused the error, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    /// Error type classification.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
}

// ---------------------------------------------------------------------------
// FunctionDefinition & FunctionParameters
// ---------------------------------------------------------------------------

/// Parameters for a function, described as a JSON Schema object.
pub type FunctionParameters = serde_json::Map<String, serde_json::Value>;

/// A function definition for tool calling.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FunctionDefinition {
    /// The name of the function (a-z, A-Z, 0-9, underscores, dashes; max 64).
    pub name: String,
    /// Description of what the function does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Function parameters as a JSON Schema object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<FunctionParameters>,
    /// Whether to enforce strict schema adherence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

// ---------------------------------------------------------------------------
// CustomToolInputFormat
// ---------------------------------------------------------------------------

/// Specifies the format for custom tool input: plain text or a grammar-based
/// constraint.
///
/// Mirrors the Go SDK's `CustomToolInputFormatUnion` with `Text` and `Grammar`
/// variants.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CustomToolInputFormat {
    /// Unconstrained free-form text.
    Text,
    /// A grammar-based constraint (lark or regex syntax).
    Grammar {
        /// The grammar definition string.
        definition: String,
        /// The syntax of the grammar definition (`lark` or `regex`).
        syntax: String,
    },
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_id_is_a_transparent_string() {
        let model_id = ModelId::new("gpt-4o-mini");
        let json = serde_json::to_string(&model_id).expect("serialize model id");
        assert_eq!(json, "\"gpt-4o-mini\"");

        let decoded: ModelId = serde_json::from_str(&json).expect("deserialize model id");
        assert_eq!(decoded.as_ref(), "gpt-4o-mini");
    }

    #[test]
    fn finish_reason_serializes_snake_case() {
        let json =
            serde_json::to_string(&FinishReason::ToolCalls).expect("serialize finish reason");
        assert_eq!(json, "\"tool_calls\"");

        let decoded: FinishReason =
            serde_json::from_str("\"function_call\"").expect("deserialize finish reason");
        assert!(matches!(decoded, FinishReason::FunctionCall));
    }

    #[test]
    fn chat_model_constants_are_correct() {
        assert_eq!(chat_model::GPT_4O, "gpt-4o");
        assert_eq!(chat_model::GPT_4O_MINI, "gpt-4o-mini");
        assert_eq!(chat_model::GPT_4_TURBO, "gpt-4-turbo");
        assert_eq!(chat_model::O3, "o3");
        assert_eq!(chat_model::GPT_3_5_TURBO, "gpt-3.5-turbo");
        assert_eq!(chat_model::GPT_5, "gpt-5");
        assert_eq!(chat_model::GPT_4_1, "gpt-4.1");
    }

    #[test]
    fn responses_model_constants_are_correct() {
        assert_eq!(responses_model::O1_PRO, "o1-pro");
        assert_eq!(responses_model::GPT_5_CODEX, "gpt-5-codex");
        assert_eq!(
            responses_model::COMPUTER_USE_PREVIEW,
            "computer-use-preview"
        );
    }

    #[test]
    fn embedding_model_constants_are_correct() {
        assert_eq!(
            embedding_model::TEXT_EMBEDDING_3_SMALL,
            "text-embedding-3-small"
        );
        assert_eq!(
            embedding_model::TEXT_EMBEDDING_3_LARGE,
            "text-embedding-3-large"
        );
        assert_eq!(
            embedding_model::TEXT_EMBEDDING_ADA_002,
            "text-embedding-ada-002"
        );
    }

    #[test]
    fn comparison_filter_serializes_correctly() {
        let filter = ComparisonFilter {
            key: "category".to_owned(),
            filter_type: ComparisonFilterType::Eq,
            value: serde_json::Value::String("science".to_owned()),
        };
        let json = serde_json::to_string(&filter).expect("serialize comparison filter");
        assert!(json.contains("\"type\":\"eq\""));
        assert!(json.contains("\"key\":\"category\""));
        assert!(json.contains("\"value\":\"science\""));

        let decoded: ComparisonFilter =
            serde_json::from_str(&json).expect("deserialize comparison filter");
        assert_eq!(decoded.key, "category");
        assert_eq!(decoded.filter_type, ComparisonFilterType::Eq);
    }

    #[test]
    fn comparison_filter_in_nin_variants() {
        let filter_in = ComparisonFilter {
            key: "tags".to_owned(),
            filter_type: ComparisonFilterType::In,
            value: serde_json::json!(["a", "b"]),
        };
        let json = serde_json::to_string(&filter_in).expect("serialize in filter");
        assert!(json.contains("\"type\":\"in\""));

        let filter_nin = ComparisonFilter {
            key: "tags".to_owned(),
            filter_type: ComparisonFilterType::NotIn,
            value: serde_json::json!(["c"]),
        };
        let json = serde_json::to_string(&filter_nin).expect("serialize nin filter");
        assert!(json.contains("\"type\":\"nin\""));
    }

    #[test]
    fn compound_filter_serializes_correctly() {
        let compound = CompoundFilter {
            filters: vec![
                FilterUnion::Comparison(ComparisonFilter {
                    key: "a".to_owned(),
                    filter_type: ComparisonFilterType::Gt,
                    value: serde_json::json!(10),
                }),
                FilterUnion::Comparison(ComparisonFilter {
                    key: "b".to_owned(),
                    filter_type: ComparisonFilterType::Lte,
                    value: serde_json::json!(20),
                }),
            ],
            filter_type: CompoundFilterType::And,
        };
        let json = serde_json::to_string(&compound).expect("serialize compound filter");
        assert!(json.contains("\"type\":\"and\""));
        assert!(json.contains("\"filters\""));

        let decoded: CompoundFilter =
            serde_json::from_str(&json).expect("deserialize compound filter");
        assert_eq!(decoded.filter_type, CompoundFilterType::And);
        assert_eq!(decoded.filters.len(), 2);
    }

    #[test]
    fn filter_union_nested_compound() {
        let nested = CompoundFilter {
            filters: vec![FilterUnion::Compound(Box::new(CompoundFilter {
                filters: vec![FilterUnion::Comparison(ComparisonFilter {
                    key: "x".to_owned(),
                    filter_type: ComparisonFilterType::Ne,
                    value: serde_json::json!(false),
                })],
                filter_type: CompoundFilterType::Or,
            }))],
            filter_type: CompoundFilterType::And,
        };
        let json = serde_json::to_string(&nested).expect("serialize nested");
        let decoded: CompoundFilter = serde_json::from_str(&json).expect("deserialize nested");
        assert_eq!(decoded.filter_type, CompoundFilterType::And);
        assert_eq!(decoded.filters.len(), 1);
        assert!(matches!(&decoded.filters[0], FilterUnion::Compound(_)));
    }

    #[test]
    fn reasoning_serializes_correctly() {
        let reasoning = Reasoning {
            effort: Some(ReasoningEffort::High),
            summary: Some(ReasoningSummary::Concise),
        };
        let json = serde_json::to_string(&reasoning).expect("serialize reasoning");
        assert!(json.contains("\"effort\":\"high\""));
        assert!(json.contains("\"summary\":\"concise\""));

        let decoded: Reasoning = serde_json::from_str(&json).expect("deserialize reasoning");
        assert_eq!(decoded.effort, Some(ReasoningEffort::High));
        assert_eq!(decoded.summary, Some(ReasoningSummary::Concise));
    }

    #[test]
    fn reasoning_effort_all_variants() {
        for (variant, expected) in [
            (ReasoningEffort::None, "\"none\""),
            (ReasoningEffort::Minimal, "\"minimal\""),
            (ReasoningEffort::Low, "\"low\""),
            (ReasoningEffort::Medium, "\"medium\""),
            (ReasoningEffort::High, "\"high\""),
            (ReasoningEffort::Xhigh, "\"xhigh\""),
        ] {
            let json = serde_json::to_string(&variant).expect("serialize");
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn reasoning_summary_all_variants() {
        for (variant, expected) in [
            (ReasoningSummary::Auto, "\"auto\""),
            (ReasoningSummary::Concise, "\"concise\""),
            (ReasoningSummary::Detailed, "\"detailed\""),
        ] {
            let json = serde_json::to_string(&variant).expect("serialize");
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn reasoning_optional_fields() {
        let reasoning = Reasoning {
            effort: None,
            summary: None,
        };
        let json = serde_json::to_string(&reasoning).expect("serialize");
        assert_eq!(json, "{}");
    }

    #[test]
    fn response_format_text_default() {
        let fmt = ResponseFormatText::default();
        let json = serde_json::to_string(&fmt).expect("serialize");
        assert_eq!(json, r#"{"type":"text"}"#);
    }

    #[test]
    fn response_format_json_object_default() {
        let fmt = ResponseFormatJsonObject::default();
        let json = serde_json::to_string(&fmt).expect("serialize");
        assert_eq!(json, r#"{"type":"json_object"}"#);
    }

    #[test]
    fn response_format_json_schema_roundtrip() {
        let fmt = ResponseFormatJsonSchema {
            format_type: ResponseFormatJsonSchemaType::JsonSchema,
            json_schema: ResponseFormatJsonSchemaConfig {
                name: "test_schema".to_owned(),
                description: Some("A test schema".to_owned()),
                schema: Some(serde_json::json!({"type": "object"})),
                strict: Some(true),
            },
        };
        let json = serde_json::to_string(&fmt).expect("serialize");
        assert!(json.contains("\"type\":\"json_schema\""));
        assert!(json.contains("\"name\":\"test_schema\""));

        let decoded: ResponseFormatJsonSchema = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.json_schema.name, "test_schema");
        assert_eq!(decoded.json_schema.strict, Some(true));
    }

    #[test]
    fn error_object_roundtrip() {
        let err = ErrorObject {
            code: Some("invalid_api_key".to_owned()),
            message: "Incorrect API key".to_owned(),
            param: None,
            error_type: Some("invalid_request_error".to_owned()),
        };
        let json = serde_json::to_string(&err).expect("serialize");
        assert!(json.contains("\"code\":\"invalid_api_key\""));
        assert!(json.contains("\"message\":\"Incorrect API key\""));
        assert!(json.contains("\"type\":\"invalid_request_error\""));
        // param should be absent (skip_serializing_if)
        assert!(!json.contains("\"param\""));

        let decoded: ErrorObject = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.code, Some("invalid_api_key".to_owned()));
        assert!(decoded.param.is_none());
    }

    #[test]
    fn metadata_type_is_hashmap() {
        let mut m: Metadata = Metadata::new();
        m.insert("foo".to_owned(), "bar".to_owned());
        let json = serde_json::to_string(&m).expect("serialize");
        assert!(json.contains("\"foo\":\"bar\""));
    }

    #[test]
    fn oauth_error_code_roundtrip() {
        let json = serde_json::to_string(&OAuthErrorCode::InvalidSubjectToken)
            .expect("serialize OAuth error code");
        assert_eq!(json, r#""invalid_subject_token""#);

        let decoded: OAuthErrorCode =
            serde_json::from_str(r#""invalid_grant""#).expect("deserialize OAuth error code");
        assert_eq!(decoded, OAuthErrorCode::InvalidGrant);

        let unknown: OAuthErrorCode =
            serde_json::from_str(r#""temporarily_unavailable""#).expect("deserialize unknown");
        assert_eq!(
            unknown,
            OAuthErrorCode::Other("temporarily_unavailable".to_owned())
        );
    }

    #[test]
    fn function_definition_roundtrip() {
        let def = FunctionDefinition {
            name: "get_weather".to_owned(),
            description: Some("Gets weather for a location".to_owned()),
            parameters: Some(serde_json::Map::from_iter([(
                "type".to_owned(),
                serde_json::json!("object"),
            )])),
            strict: Some(true),
        };
        let json = serde_json::to_string(&def).expect("serialize");
        let decoded: FunctionDefinition = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.name, "get_weather");
        assert_eq!(decoded.strict, Some(true));
    }

    #[test]
    fn custom_tool_input_format_text_roundtrip() {
        let fmt = CustomToolInputFormat::Text;
        let json = serde_json::to_string(&fmt).expect("serialize");
        assert_eq!(json, r#"{"type":"text"}"#);

        let decoded: CustomToolInputFormat = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, CustomToolInputFormat::Text);
    }

    #[test]
    fn custom_tool_input_format_grammar_roundtrip() {
        let fmt = CustomToolInputFormat::Grammar {
            definition: "start: \"hello\"".to_owned(),
            syntax: "lark".to_owned(),
        };
        let json = serde_json::to_string(&fmt).expect("serialize");
        assert!(json.contains("\"type\":\"grammar\""));
        assert!(json.contains("\"definition\":\"start: \\\"hello\\\"\""));
        assert!(json.contains("\"syntax\":\"lark\""));

        let decoded: CustomToolInputFormat = serde_json::from_str(&json).expect("deserialize");
        match decoded {
            CustomToolInputFormat::Grammar { definition, syntax } => {
                assert_eq!(definition, "start: \"hello\"");
                assert_eq!(syntax, "lark");
            }
            _ => panic!("expected Grammar variant"),
        }
    }
}
