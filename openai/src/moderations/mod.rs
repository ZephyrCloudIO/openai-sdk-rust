//! Moderation APIs.

use crate::{shared::ModelId, Client, Result};

// ---------------------------------------------------------------------------
// Moderation model enum
// ---------------------------------------------------------------------------

/// Moderation model identifiers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ModerationModel {
    /// Latest omni moderation model.
    #[serde(rename = "omni-moderation-latest")]
    OmniModerationLatest,
    /// Omni moderation 2024-09-26 snapshot.
    #[serde(rename = "omni-moderation-2024-09-26")]
    OmniModeration20240926,
    /// Latest text moderation model.
    #[serde(rename = "text-moderation-latest")]
    TextModerationLatest,
    /// Stable text moderation model.
    #[serde(rename = "text-moderation-stable")]
    TextModerationStable,
}

impl std::fmt::Display for ModerationModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OmniModerationLatest => write!(f, "omni-moderation-latest"),
            Self::OmniModeration20240926 => write!(f, "omni-moderation-2024-09-26"),
            Self::TextModerationLatest => write!(f, "text-moderation-latest"),
            Self::TextModerationStable => write!(f, "text-moderation-stable"),
        }
    }
}

// ---------------------------------------------------------------------------
// Multi-modal input types
// ---------------------------------------------------------------------------

/// Multi-modal moderation input: text or image URL.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ModerationMultiModalInput {
    /// Text input for classification.
    #[serde(rename = "text")]
    Text {
        /// The text to classify.
        text: String,
    },
    /// Image URL input for classification.
    #[serde(rename = "image_url")]
    ImageUrl {
        /// Contains the image URL or base64 data URL.
        image_url: ModerationImageUrl,
    },
}

/// Image URL reference for moderation input.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModerationImageUrl {
    /// Either a URL of the image or the base64 encoded image data.
    pub url: String,
}

/// Input for the moderation endpoint: single string, array of strings, or multi-modal.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ModerationInput {
    /// Single text input.
    Single(String),
    /// Multiple text inputs.
    StringArray(Vec<String>),
    /// Multi-modal inputs (text and/or image).
    MultiModal(Vec<ModerationMultiModalInput>),
}

impl From<String> for ModerationInput {
    fn from(value: String) -> Self {
        Self::Single(value)
    }
}

impl From<&str> for ModerationInput {
    fn from(value: &str) -> Self {
        Self::Single(value.to_owned())
    }
}

impl From<Vec<String>> for ModerationInput {
    fn from(value: Vec<String>) -> Self {
        Self::StringArray(value)
    }
}

impl From<Vec<ModerationMultiModalInput>> for ModerationInput {
    fn from(value: Vec<ModerationMultiModalInput>) -> Self {
        Self::MultiModal(value)
    }
}

impl ModerationInput {
    /// Returns the union value as JSON for dynamic matching.
    #[must_use]
    pub fn as_any(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Returns the single text input variant.
    #[must_use]
    pub fn as_single(&self) -> Option<&str> {
        match self {
            Self::Single(value) => Some(value),
            Self::StringArray(_) | Self::MultiModal(_) => None,
        }
    }

    /// Returns the string array variant.
    #[must_use]
    pub fn as_string_array(&self) -> Option<&[String]> {
        match self {
            Self::StringArray(value) => Some(value),
            Self::Single(_) | Self::MultiModal(_) => None,
        }
    }

    /// Returns the multimodal input variant.
    #[must_use]
    pub fn as_multi_modal(&self) -> Option<&[ModerationMultiModalInput]> {
        match self {
            Self::MultiModal(value) => Some(value),
            Self::Single(_) | Self::StringArray(_) => None,
        }
    }

    /// Creates a single text input variant.
    #[must_use]
    pub fn param_of_single(value: impl Into<String>) -> Self {
        Self::Single(value.into())
    }

    /// Creates a string array input variant.
    #[must_use]
    pub fn param_of_string_array(value: Vec<String>) -> Self {
        Self::StringArray(value)
    }

    /// Creates a multimodal input variant.
    #[must_use]
    pub fn param_of_multi_modal(value: Vec<ModerationMultiModalInput>) -> Self {
        Self::MultiModal(value)
    }
}

// ---------------------------------------------------------------------------
// Moderation categories (all 13)
// ---------------------------------------------------------------------------

/// Moderation category flags -- all 13 categories as defined by the OpenAI API.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModerationCategories {
    /// Content that expresses, incites, or promotes harassing language.
    pub harassment: bool,
    /// Harassment content that also includes violence or serious harm.
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: bool,
    /// Content that expresses, incites, or promotes hate based on protected attributes.
    pub hate: bool,
    /// Hateful content that also includes violence or serious harm towards the targeted group.
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: bool,
    /// Content that includes instructions or advice facilitating illicit acts.
    #[serde(default)]
    pub illicit: bool,
    /// Illicit content that also includes violence or weapon procurement.
    #[serde(rename = "illicit/violent", default)]
    pub illicit_violent: bool,
    /// Content that promotes, encourages, or depicts acts of self-harm.
    #[serde(rename = "self-harm")]
    pub self_harm: bool,
    /// Content that gives instructions or advice on how to commit self-harm.
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: bool,
    /// Content where the speaker intends to engage in self-harm.
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: bool,
    /// Content meant to arouse sexual excitement.
    pub sexual: bool,
    /// Sexual content involving minors.
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: bool,
    /// Content that depicts death, violence, or physical injury.
    pub violence: bool,
    /// Content that depicts death, violence, or physical injury in graphic detail.
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: bool,
}

/// Moderation category scores -- all 13 categories as `f64`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModerationCategoryScores {
    /// Score for harassment.
    pub harassment: f64,
    /// Score for harassment/threatening.
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: f64,
    /// Score for hate.
    pub hate: f64,
    /// Score for hate/threatening.
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: f64,
    /// Score for illicit.
    #[serde(default)]
    pub illicit: f64,
    /// Score for illicit/violent.
    #[serde(rename = "illicit/violent", default)]
    pub illicit_violent: f64,
    /// Score for self-harm.
    #[serde(rename = "self-harm")]
    pub self_harm: f64,
    /// Score for self-harm/instructions.
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: f64,
    /// Score for self-harm/intent.
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: f64,
    /// Score for sexual.
    pub sexual: f64,
    /// Score for sexual/minors.
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: f64,
    /// Score for violence.
    pub violence: f64,
    /// Score for violence/graphic.
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: f64,
}

/// Applied input types per category -- all 13 categories.
/// Each field contains the input types (e.g. `"text"`, `"image"`) that the score applies to.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModerationCategoryAppliedInputTypes {
    /// Applied input types for harassment.
    #[serde(default)]
    pub harassment: Vec<String>,
    /// Applied input types for harassment/threatening.
    #[serde(rename = "harassment/threatening", default)]
    pub harassment_threatening: Vec<String>,
    /// Applied input types for hate.
    #[serde(default)]
    pub hate: Vec<String>,
    /// Applied input types for hate/threatening.
    #[serde(rename = "hate/threatening", default)]
    pub hate_threatening: Vec<String>,
    /// Applied input types for illicit.
    #[serde(default)]
    pub illicit: Vec<String>,
    /// Applied input types for illicit/violent.
    #[serde(rename = "illicit/violent", default)]
    pub illicit_violent: Vec<String>,
    /// Applied input types for self-harm.
    #[serde(rename = "self-harm", default)]
    pub self_harm: Vec<String>,
    /// Applied input types for self-harm/instructions.
    #[serde(rename = "self-harm/instructions", default)]
    pub self_harm_instructions: Vec<String>,
    /// Applied input types for self-harm/intent.
    #[serde(rename = "self-harm/intent", default)]
    pub self_harm_intent: Vec<String>,
    /// Applied input types for sexual.
    #[serde(default)]
    pub sexual: Vec<String>,
    /// Applied input types for sexual/minors.
    #[serde(rename = "sexual/minors", default)]
    pub sexual_minors: Vec<String>,
    /// Applied input types for violence.
    #[serde(default)]
    pub violence: Vec<String>,
    /// Applied input types for violence/graphic.
    #[serde(rename = "violence/graphic", default)]
    pub violence_graphic: Vec<String>,
}

// ---------------------------------------------------------------------------
// Moderation service
// ---------------------------------------------------------------------------

/// Moderation service.
#[derive(Clone)]
pub struct ModerationService {
    client: Client,
}

impl ModerationService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates moderation results for input text or multi-modal content.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: ModerationCreateParams) -> Result<ModerationResponse> {
        self.client.post_json("/moderations", &params).await
    }
}

// ---------------------------------------------------------------------------
// Request / Response
// ---------------------------------------------------------------------------

/// Moderation request payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModerationCreateParams {
    /// Model ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelId>,
    /// Input text, list of texts, or multi-modal inputs.
    pub input: ModerationInput,
}

/// Moderation API response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModerationResponse {
    /// Response ID.
    pub id: String,
    /// Model used.
    pub model: ModelId,
    /// Result list.
    pub results: Vec<ModerationResult>,
}

/// One moderation result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModerationResult {
    /// Flagged status.
    pub flagged: bool,
    /// Category booleans (all 13 categories).
    pub categories: ModerationCategories,
    /// Category scores (all 13 categories).
    pub category_scores: ModerationCategoryScores,
    /// Applied input types per category (all 13 categories).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_applied_input_types: Option<ModerationCategoryAppliedInputTypes>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ModelId;

    #[test]
    fn create_params_support_multiple_inputs() {
        let params = ModerationCreateParams {
            model: None,
            input: ModerationInput::StringArray(vec!["hello".to_owned(), "world".to_owned()]),
        };

        let value = serde_json::to_value(params).expect("serialize moderation params");
        assert!(value.get("model").is_none());
        assert_eq!(
            value
                .get("input")
                .and_then(serde_json::Value::as_array)
                .map(Vec::len),
            Some(2)
        );
    }

    #[test]
    fn create_params_support_single_input() {
        let params = ModerationCreateParams {
            model: Some(ModelId::from("text-moderation-latest")),
            input: ModerationInput::from("test text"),
        };

        let value = serde_json::to_value(&params).expect("serialize moderation params");
        assert_eq!(
            value.get("input"),
            Some(&serde_json::Value::String("test text".to_owned()))
        );
    }

    #[test]
    fn create_params_support_multi_modal_input() {
        let params = ModerationCreateParams {
            model: Some(ModelId::from("omni-moderation-latest")),
            input: ModerationInput::MultiModal(vec![
                ModerationMultiModalInput::Text {
                    text: "hello".to_owned(),
                },
                ModerationMultiModalInput::ImageUrl {
                    image_url: ModerationImageUrl {
                        url: "https://example.com/image.png".to_owned(),
                    },
                },
            ]),
        };

        let value = serde_json::to_value(&params).expect("serialize moderation params");
        let input_arr = value
            .get("input")
            .and_then(serde_json::Value::as_array)
            .expect("input is array");
        assert_eq!(input_arr.len(), 2);
        assert_eq!(
            input_arr[0].get("type"),
            Some(&serde_json::Value::String("text".to_owned()))
        );
        assert_eq!(
            input_arr[1].get("type"),
            Some(&serde_json::Value::String("image_url".to_owned()))
        );
    }

    #[test]
    fn moderation_response_deserializes_all_13_categories() {
        let json = r#"{
            "id": "modr_123",
            "model": "omni-moderation-latest",
            "results": [{
                "flagged": true,
                "categories": {
                    "harassment": true,
                    "harassment/threatening": false,
                    "hate": false,
                    "hate/threatening": false,
                    "illicit": false,
                    "illicit/violent": false,
                    "self-harm": false,
                    "self-harm/instructions": false,
                    "self-harm/intent": false,
                    "sexual": false,
                    "sexual/minors": false,
                    "violence": true,
                    "violence/graphic": false
                },
                "category_scores": {
                    "harassment": 0.95,
                    "harassment/threatening": 0.1,
                    "hate": 0.05,
                    "hate/threatening": 0.01,
                    "illicit": 0.02,
                    "illicit/violent": 0.01,
                    "self-harm": 0.0,
                    "self-harm/instructions": 0.0,
                    "self-harm/intent": 0.0,
                    "sexual": 0.0,
                    "sexual/minors": 0.0,
                    "violence": 0.8,
                    "violence/graphic": 0.1
                },
                "category_applied_input_types": {
                    "harassment": ["text"],
                    "harassment/threatening": ["text"],
                    "hate": ["text"],
                    "hate/threatening": ["text"],
                    "illicit": ["text"],
                    "illicit/violent": ["text"],
                    "self-harm": ["text", "image"],
                    "self-harm/instructions": ["text", "image"],
                    "self-harm/intent": ["text", "image"],
                    "sexual": ["text", "image"],
                    "sexual/minors": ["text"],
                    "violence": ["text", "image"],
                    "violence/graphic": ["text", "image"]
                }
            }]
        }"#;

        let response: ModerationResponse =
            serde_json::from_str(json).expect("deserialize moderation response");
        assert_eq!(response.model, ModelId::from("omni-moderation-latest"));
        assert_eq!(response.results.len(), 1);

        let result = &response.results[0];
        assert!(result.flagged);
        assert!(result.categories.harassment);
        assert!(!result.categories.harassment_threatening);
        assert!(!result.categories.hate);
        assert!(!result.categories.hate_threatening);
        assert!(!result.categories.illicit);
        assert!(!result.categories.illicit_violent);
        assert!(!result.categories.self_harm);
        assert!(!result.categories.self_harm_instructions);
        assert!(!result.categories.self_harm_intent);
        assert!(!result.categories.sexual);
        assert!(!result.categories.sexual_minors);
        assert!(result.categories.violence);
        assert!(!result.categories.violence_graphic);

        assert!(result.category_scores.harassment > 0.9);
        assert!(result.category_scores.violence > 0.7);
        assert!(result.category_scores.sexual < 0.01);

        let applied = result
            .category_applied_input_types
            .as_ref()
            .expect("applied input types present");
        assert_eq!(applied.harassment, vec!["text"]);
        assert_eq!(applied.self_harm, vec!["text", "image"]);
        assert_eq!(applied.violence, vec!["text", "image"]);
    }

    #[test]
    fn moderation_response_deserializes_without_optional_applied_types() {
        let json = r#"{
            "id": "modr_456",
            "model": "text-moderation-latest",
            "results": [{
                "flagged": false,
                "categories": {
                    "harassment": false,
                    "harassment/threatening": false,
                    "hate": false,
                    "hate/threatening": false,
                    "self-harm": false,
                    "self-harm/instructions": false,
                    "self-harm/intent": false,
                    "sexual": false,
                    "sexual/minors": false,
                    "violence": false,
                    "violence/graphic": false
                },
                "category_scores": {
                    "harassment": 0.01,
                    "harassment/threatening": 0.0,
                    "hate": 0.0,
                    "hate/threatening": 0.0,
                    "self-harm": 0.0,
                    "self-harm/instructions": 0.0,
                    "self-harm/intent": 0.0,
                    "sexual": 0.0,
                    "sexual/minors": 0.0,
                    "violence": 0.0,
                    "violence/graphic": 0.0
                }
            }]
        }"#;

        let response: ModerationResponse =
            serde_json::from_str(json).expect("deserialize moderation response");
        assert_eq!(response.results.len(), 1);
        assert!(!response.results[0].flagged);
        assert!(response.results[0].category_applied_input_types.is_none());
        // illicit defaults to false/0.0 when missing
        assert!(!response.results[0].categories.illicit);
        assert!(response.results[0].category_scores.illicit < f64::EPSILON);
    }

    #[test]
    fn moderation_model_serialization() {
        let model = ModerationModel::OmniModerationLatest;
        let json = serde_json::to_string(&model).expect("serialize moderation model");
        assert_eq!(json, "\"omni-moderation-latest\"");

        let model = ModerationModel::TextModerationStable;
        let json = serde_json::to_string(&model).expect("serialize moderation model");
        assert_eq!(json, "\"text-moderation-stable\"");
    }

    #[test]
    fn multi_modal_input_text_serialization() {
        let input = ModerationMultiModalInput::Text {
            text: "hello world".to_owned(),
        };
        let value = serde_json::to_value(&input).expect("serialize text input");
        assert_eq!(
            value.get("type"),
            Some(&serde_json::Value::String("text".to_owned()))
        );
        assert_eq!(
            value.get("text"),
            Some(&serde_json::Value::String("hello world".to_owned()))
        );
    }

    #[test]
    fn multi_modal_input_image_url_serialization() {
        let input = ModerationMultiModalInput::ImageUrl {
            image_url: ModerationImageUrl {
                url: "https://example.com/img.png".to_owned(),
            },
        };
        let value = serde_json::to_value(&input).expect("serialize image url input");
        assert_eq!(
            value.get("type"),
            Some(&serde_json::Value::String("image_url".to_owned()))
        );
        assert!(value.get("image_url").is_some());
    }
}
