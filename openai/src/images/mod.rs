//! Images APIs.

use futures::Stream;
use reqwest::multipart::{Form, Part};

use crate::{shared::ModelId, ssestream::SseStream, Client, Result};

// ---------------------------------------------------------------------------
// Image model enum
// ---------------------------------------------------------------------------

/// Image generation model identifiers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ImageModel {
    /// GPT Image 1.5 model.
    #[serde(rename = "gpt-image-1.5")]
    GptImage1_5,
    /// DALL-E 2 model.
    #[serde(rename = "dall-e-2")]
    DallE2,
    /// DALL-E 3 model.
    #[serde(rename = "dall-e-3")]
    DallE3,
    /// GPT Image 1 model.
    #[serde(rename = "gpt-image-1")]
    GptImage1,
    /// GPT Image 1 Mini model.
    #[serde(rename = "gpt-image-1-mini")]
    GptImage1Mini,
}

impl std::fmt::Display for ImageModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GptImage1_5 => write!(f, "gpt-image-1.5"),
            Self::DallE2 => write!(f, "dall-e-2"),
            Self::DallE3 => write!(f, "dall-e-3"),
            Self::GptImage1 => write!(f, "gpt-image-1"),
            Self::GptImage1Mini => write!(f, "gpt-image-1-mini"),
        }
    }
}

// ---------------------------------------------------------------------------
// Image enums
// ---------------------------------------------------------------------------

/// Image size options.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ImageSize {
    /// Automatic sizing.
    #[serde(rename = "auto")]
    Auto,
    /// 256x256 pixels (DALL-E 2 only).
    #[serde(rename = "256x256")]
    Size256x256,
    /// 512x512 pixels (DALL-E 2 only).
    #[serde(rename = "512x512")]
    Size512x512,
    /// 1024x1024 pixels.
    #[serde(rename = "1024x1024")]
    Size1024x1024,
    /// 1536x1024 pixels (landscape, GPT image models).
    #[serde(rename = "1536x1024")]
    Size1536x1024,
    /// 1024x1536 pixels (portrait, GPT image models).
    #[serde(rename = "1024x1536")]
    Size1024x1536,
    /// 1792x1024 pixels (DALL-E 3 only).
    #[serde(rename = "1792x1024")]
    Size1792x1024,
    /// 1024x1792 pixels (DALL-E 3 only).
    #[serde(rename = "1024x1792")]
    Size1024x1792,
}

impl std::fmt::Display for ImageSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Size256x256 => write!(f, "256x256"),
            Self::Size512x512 => write!(f, "512x512"),
            Self::Size1024x1024 => write!(f, "1024x1024"),
            Self::Size1536x1024 => write!(f, "1536x1024"),
            Self::Size1024x1536 => write!(f, "1024x1536"),
            Self::Size1792x1024 => write!(f, "1792x1024"),
            Self::Size1024x1792 => write!(f, "1024x1792"),
        }
    }
}

/// Background setting for generated images.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageBackground {
    /// Transparent background.
    Transparent,
    /// Opaque background.
    Opaque,
    /// Automatically determine background.
    Auto,
}

/// Content moderation level for image generation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageModeration {
    /// Less restrictive filtering.
    Low,
    /// Default moderation.
    Auto,
}

/// Output format for GPT image models.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageOutputFormat {
    /// PNG format.
    Png,
    /// JPEG format.
    Jpeg,
    /// WebP format.
    #[serde(rename = "webp")]
    WebP,
}

/// Quality setting for image generation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageQuality {
    /// Standard quality (DALL-E 2/3).
    Standard,
    /// HD quality (DALL-E 3).
    Hd,
    /// Low quality (GPT image models).
    Low,
    /// Medium quality (GPT image models).
    Medium,
    /// High quality (GPT image models).
    High,
    /// Auto quality (default).
    Auto,
}

/// Style for DALL-E 3 image generation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageStyle {
    /// Hyper-real and dramatic style.
    Vivid,
    /// More natural, less hyper-real style.
    Natural,
}

/// Input fidelity for image editing (GPT image models).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageInputFidelity {
    /// High fidelity matching.
    High,
    /// Low fidelity matching (default).
    Low,
}

/// Response format for DALL-E 2/3 images.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageResponseFormat {
    /// URL format (valid for 60 minutes).
    Url,
    /// Base64-encoded JSON format.
    B64Json,
}

impl std::fmt::Display for ImageResponseFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Url => write!(f, "url"),
            Self::B64Json => write!(f, "b64_json"),
        }
    }
}

// ---------------------------------------------------------------------------
// Image usage types
// ---------------------------------------------------------------------------

/// Token usage information for image generation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageUsage {
    /// The number of tokens (images and text) in the input prompt.
    pub input_tokens: i64,
    /// The input tokens detailed information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens_details: Option<ImageUsageTokensDetails>,
    /// The number of output tokens generated.
    pub output_tokens: i64,
    /// The total number of tokens used.
    pub total_tokens: i64,
    /// The output token details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens_details: Option<ImageUsageTokensDetails>,
}

/// Detailed token breakdown for image usage.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageUsageTokensDetails {
    /// The number of image tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_tokens: Option<i64>,
    /// The number of text tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_tokens: Option<i64>,
}

// ---------------------------------------------------------------------------
// Image streaming event types
// ---------------------------------------------------------------------------

/// Streaming event for image generation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ImageGenStreamEvent {
    /// Partial image available during streaming.
    #[serde(rename = "image_generation.partial_image")]
    PartialImage(ImageGenPartialImageEvent),
    /// Final generated image available.
    #[serde(rename = "image_generation.completed")]
    Completed(ImageGenCompletedEvent),
}

/// Emitted when a partial image is available during image generation streaming.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageGenPartialImageEvent {
    /// Base64-encoded partial image data.
    pub b64_json: String,
    /// The background setting for the requested image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    /// The Unix timestamp when the event was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// The output format for the requested image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    /// 0-based index for the partial image (streaming).
    pub partial_image_index: i64,
    /// The quality setting for the requested image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// The size of the requested image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
}

/// Emitted when image generation has completed and the final image is available.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageGenCompletedEvent {
    /// Base64-encoded image data.
    pub b64_json: String,
    /// The background setting for the generated image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    /// The Unix timestamp when the event was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// The output format for the generated image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    /// The quality setting for the generated image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// The size of the generated image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Token usage information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ImageUsage>,
}

/// Streaming event for image editing.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ImageEditStreamEvent {
    /// Partial image available during editing streaming.
    #[serde(rename = "image_edit.partial_image")]
    PartialImage(ImageEditPartialImageEvent),
    /// Final edited image available.
    #[serde(rename = "image_edit.completed")]
    Completed(ImageEditCompletedEvent),
}

/// Emitted when a partial image is available during image editing streaming.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageEditPartialImageEvent {
    /// Base64-encoded partial image data.
    pub b64_json: String,
    /// The background setting for the requested image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    /// The Unix timestamp when the event was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// The output format for the requested image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    /// 0-based index for the partial image (streaming).
    pub partial_image_index: i64,
    /// The quality setting for the requested image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// The size of the requested image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
}

/// Emitted when image editing has completed and the final image is available.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageEditCompletedEvent {
    /// Base64-encoded final edited image data.
    pub b64_json: String,
    /// The background setting for the edited image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    /// The Unix timestamp when the event was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// The output format for the edited image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    /// The quality setting for the edited image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// The size of the edited image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Token usage information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ImageUsage>,
}

// ---------------------------------------------------------------------------
// Image service
// ---------------------------------------------------------------------------

/// Image service.
#[derive(Clone)]
pub struct ImageService {
    client: Client,
}

impl ImageService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates images from text prompt.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn generate(&self, params: ImageGenerateParams) -> Result<ImageResponse> {
        self.client.post_json("/images/generations", &params).await
    }

    /// Alias for [`ImageService::generate`].
    pub async fn generations(&self, params: ImageGenerateParams) -> Result<ImageResponse> {
        self.generate(params).await
    }

    /// Creates a streaming image generation, returning partial images as they arrive.
    ///
    /// The `partial_images` field in `params` controls how many intermediate images
    /// are delivered before the final completed event.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or stream decode failures.
    pub async fn generate_streaming(
        &self,
        params: ImageGenerateParams,
    ) -> Result<impl Stream<Item = Result<ImageGenStreamEvent>>> {
        let mut body = serde_json::to_value(&params)?;
        if let Some(object) = body.as_object_mut() {
            object.insert("stream".to_owned(), serde_json::Value::Bool(true));
        }
        let response = self
            .client
            .post_raw_json("/images/generations", &body)
            .await?;
        Ok(SseStream::new(response))
    }

    /// Creates edited images from prompt and source image.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn edit(&self, params: ImageEditParams) -> Result<ImageResponse> {
        self.client
            .post_multipart_json("/images/edits", || params.to_form())
            .await
    }

    /// Creates a streaming image edit, returning partial images as they arrive.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or stream decode failures.
    pub async fn edit_streaming(
        &self,
        params: ImageEditParams,
    ) -> Result<impl Stream<Item = Result<ImageEditStreamEvent>>> {
        let response = self
            .client
            .post_raw_multipart("/images/edits", || params.to_form().text("stream", "true"))
            .await?;
        Ok(SseStream::new(response))
    }

    /// Alias for [`ImageService::edit`].
    pub async fn edits(&self, params: ImageEditParams) -> Result<ImageResponse> {
        self.edit(params).await
    }

    /// Creates image variations from source image.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create_variation(&self, params: ImageVariationParams) -> Result<ImageResponse> {
        self.client
            .post_multipart_json("/images/variations", || params.to_form())
            .await
    }

    /// Alias for [`ImageService::create_variation`].
    pub async fn variations(&self, params: ImageVariationParams) -> Result<ImageResponse> {
        self.create_variation(params).await
    }
}

// ---------------------------------------------------------------------------
// In-memory image file input
// ---------------------------------------------------------------------------

/// In-memory image file input.
#[derive(Debug, Clone)]
pub struct ImageInputFile {
    /// Image bytes.
    pub bytes: Vec<u8>,
    /// Filename sent to API.
    pub filename: String,
    /// Optional MIME type.
    pub content_type: Option<String>,
}

impl ImageInputFile {
    /// Creates an image input from bytes.
    #[must_use]
    pub fn from_bytes(bytes: impl Into<Vec<u8>>, filename: impl Into<String>) -> Self {
        Self {
            bytes: bytes.into(),
            filename: filename.into(),
            content_type: None,
        }
    }

    /// Sets MIME type.
    #[must_use]
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Request params
// ---------------------------------------------------------------------------

/// Image generation request payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageGenerateParams {
    /// Prompt text.
    pub prompt: String,
    /// Optional model ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelId>,
    /// Optional number of images.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Optional size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ImageSize>,
    /// Optional response format (`url`, `b64_json`) for DALL-E 2/3.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ImageResponseFormat>,
    /// Optional compression level (0-100%) for webp or jpeg output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_compression: Option<i64>,
    /// Optional number of partial images for streaming (0-3).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_images: Option<i64>,
    /// Optional end-user identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Background setting for GPT image models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<ImageBackground>,
    /// Content moderation level for GPT image models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation: Option<ImageModeration>,
    /// Output format for GPT image models (png, jpeg, webp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<ImageOutputFormat>,
    /// Quality setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<ImageQuality>,
    /// Style for DALL-E 3 (vivid, natural).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ImageStyle>,
}

/// Input for the image field in [`ImageEditParams`]: a single file or an array
/// of files.
///
/// Mirrors the Go SDK's `ImageEditParamsImageUnion` which is a union of a single
/// `io.Reader` or `[]io.Reader`.
#[derive(Debug, Clone)]
pub enum ImageEditInput {
    /// A single source image.
    Single(ImageInputFile),
    /// Multiple source images (up to 16 for GPT image models).
    Multiple(Vec<ImageInputFile>),
}

impl From<ImageInputFile> for ImageEditInput {
    fn from(file: ImageInputFile) -> Self {
        Self::Single(file)
    }
}

impl From<Vec<ImageInputFile>> for ImageEditInput {
    fn from(files: Vec<ImageInputFile>) -> Self {
        Self::Multiple(files)
    }
}

impl ImageEditInput {
    /// Returns the single image variant.
    #[must_use]
    pub fn as_single(&self) -> Option<&ImageInputFile> {
        match self {
            Self::Single(value) => Some(value),
            Self::Multiple(_) => None,
        }
    }

    /// Returns the multiple image variant.
    #[must_use]
    pub fn as_multiple(&self) -> Option<&[ImageInputFile]> {
        match self {
            Self::Single(_) => None,
            Self::Multiple(value) => Some(value),
        }
    }

    /// Creates a single image edit input variant.
    #[must_use]
    pub fn param_of_single(value: ImageInputFile) -> Self {
        Self::Single(value)
    }

    /// Creates a multiple image edit input variant.
    #[must_use]
    pub fn param_of_multiple(value: Vec<ImageInputFile>) -> Self {
        Self::Multiple(value)
    }
}

/// Image edit request payload.
#[derive(Debug, Clone)]
pub struct ImageEditParams {
    /// Source image(s). Use [`ImageEditInput::Single`] for one image or
    /// [`ImageEditInput::Multiple`] for up to 16 images (GPT image models).
    pub image: ImageEditInput,
    /// Prompt text.
    pub prompt: String,
    /// Optional mask image.
    pub mask: Option<ImageInputFile>,
    /// Optional model ID.
    pub model: Option<ModelId>,
    /// Optional number of images.
    pub n: Option<u32>,
    /// Optional size.
    pub size: Option<ImageSize>,
    /// Optional response format for DALL-E 2 (`url`, `b64_json`).
    pub response_format: Option<ImageResponseFormat>,
    /// Optional compression level (0-100%) for webp or jpeg output.
    pub output_compression: Option<i64>,
    /// Optional number of partial images for streaming (0-3).
    pub partial_images: Option<i64>,
    /// Optional end-user identifier.
    pub user: Option<String>,
    /// Background setting for GPT image models.
    pub background: Option<ImageBackground>,
    /// Input fidelity control (high or low).
    pub input_fidelity: Option<ImageInputFidelity>,
    /// Output format for GPT image models (png, jpeg, webp).
    pub output_format: Option<ImageOutputFormat>,
    /// Quality setting.
    pub quality: Option<ImageQuality>,
}

impl ImageEditParams {
    /// Build a multipart [`Part`] from an [`ImageInputFile`].
    fn file_to_part(file: &ImageInputFile) -> Part {
        let mut part = Part::bytes(file.bytes.clone()).file_name(file.filename.clone());
        if let Some(content_type) = file.content_type.as_deref() {
            let candidate = Part::bytes(file.bytes.clone()).file_name(file.filename.clone());
            if let Ok(updated) = candidate.mime_str(content_type) {
                part = updated;
            }
        }
        part
    }

    fn to_form(&self) -> Form {
        let mut form = Form::new().text("prompt", self.prompt.clone());

        // Add image(s) — mirrors Go SDK's brackets array format for multiple
        // images: single image uses field name "image", multiple images use
        // "image[]" for each entry.
        match &self.image {
            ImageEditInput::Single(file) => {
                form = form.part("image", Self::file_to_part(file));
            }
            ImageEditInput::Multiple(files) => {
                for file in files {
                    form = form.part("image[]", Self::file_to_part(file));
                }
            }
        }

        if let Some(model) = &self.model {
            form = form.text("model", model.to_string());
        }
        if let Some(n) = self.n {
            form = form.text("n", n.to_string());
        }
        if let Some(size) = &self.size {
            form = form.text("size", size.to_string());
        }
        if let Some(response_format) = &self.response_format {
            form = form.text("response_format", response_format.to_string());
        }
        if let Some(output_compression) = self.output_compression {
            form = form.text("output_compression", output_compression.to_string());
        }
        if let Some(partial_images) = self.partial_images {
            form = form.text("partial_images", partial_images.to_string());
        }
        if let Some(user) = &self.user {
            form = form.text("user", user.clone());
        }
        if let Some(background) = &self.background {
            if let Ok(json) = serde_json::to_value(background) {
                if let Some(s) = json.as_str() {
                    form = form.text("background", s.to_owned());
                }
            }
        }
        if let Some(input_fidelity) = &self.input_fidelity {
            if let Ok(json) = serde_json::to_value(input_fidelity) {
                if let Some(s) = json.as_str() {
                    form = form.text("input_fidelity", s.to_owned());
                }
            }
        }
        if let Some(output_format) = &self.output_format {
            if let Ok(json) = serde_json::to_value(output_format) {
                if let Some(s) = json.as_str() {
                    form = form.text("output_format", s.to_owned());
                }
            }
        }
        if let Some(quality) = &self.quality {
            if let Ok(json) = serde_json::to_value(quality) {
                if let Some(s) = json.as_str() {
                    form = form.text("quality", s.to_owned());
                }
            }
        }
        if let Some(mask) = &self.mask {
            let mut mask_part = Part::bytes(mask.bytes.clone()).file_name(mask.filename.clone());
            if let Some(content_type) = mask.content_type.as_deref() {
                let candidate = Part::bytes(mask.bytes.clone()).file_name(mask.filename.clone());
                if let Ok(updated) = candidate.mime_str(content_type) {
                    mask_part = updated;
                }
            }
            form = form.part("mask", mask_part);
        }

        form
    }
}

/// Image variation request payload.
#[derive(Debug, Clone)]
pub struct ImageVariationParams {
    /// Source image.
    pub image: ImageInputFile,
    /// Optional model ID.
    pub model: Option<ModelId>,
    /// Optional number of images.
    pub n: Option<u32>,
    /// Optional size.
    pub size: Option<ImageSize>,
    /// Optional response format.
    pub response_format: Option<ImageResponseFormat>,
    /// Optional end-user identifier.
    pub user: Option<String>,
}

impl ImageVariationParams {
    fn to_form(&self) -> Form {
        let mut image_part =
            Part::bytes(self.image.bytes.clone()).file_name(self.image.filename.clone());
        if let Some(content_type) = self.image.content_type.as_deref() {
            let candidate =
                Part::bytes(self.image.bytes.clone()).file_name(self.image.filename.clone());
            if let Ok(updated) = candidate.mime_str(content_type) {
                image_part = updated;
            }
        }

        let mut form = Form::new().part("image", image_part);

        if let Some(model) = &self.model {
            form = form.text("model", model.to_string());
        }
        if let Some(n) = self.n {
            form = form.text("n", n.to_string());
        }
        if let Some(size) = &self.size {
            form = form.text("size", size.to_string());
        }
        if let Some(response_format) = &self.response_format {
            form = form.text("response_format", response_format.to_string());
        }
        if let Some(user) = &self.user {
            form = form.text("user", user.clone());
        }

        form
    }
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Image API response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageResponse {
    /// Unix creation timestamp.
    pub created: u64,
    /// Returned image payloads.
    #[serde(default)]
    pub data: Vec<ImageData>,
    /// The background parameter used for the image generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    /// The output format of the image generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    /// The quality of the image generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// The size of the image generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Token usage information (GPT image models only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ImageUsage>,
}

/// One image payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageData {
    /// Remote URL output, when requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Base64 output, when requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b64_json: Option<String>,
    /// Revised prompt text if provided by model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revised_prompt: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_generate_omits_optional_fields() {
        let params = ImageGenerateParams {
            prompt: "A red fox".to_owned(),
            model: None,
            n: None,
            size: None,
            response_format: None,
            output_compression: None,
            partial_images: None,
            user: None,
            background: None,
            moderation: None,
            output_format: None,
            quality: None,
            style: None,
        };

        let value = serde_json::to_value(params).expect("serialize image generate params");
        assert!(value.get("model").is_none());
        assert!(value.get("n").is_none());
        assert!(value.get("background").is_none());
        assert!(value.get("output_format").is_none());
    }

    #[test]
    fn image_generate_includes_all_fields() {
        let params = ImageGenerateParams {
            prompt: "A sunset".to_owned(),
            model: Some(ModelId::from("gpt-image-1")),
            n: Some(2),
            size: Some(ImageSize::Size1024x1024),
            response_format: None,
            output_compression: Some(80),
            partial_images: Some(2),
            user: Some("user_123".to_owned()),
            background: Some(ImageBackground::Transparent),
            moderation: Some(ImageModeration::Low),
            output_format: Some(ImageOutputFormat::Png),
            quality: Some(ImageQuality::High),
            style: Some(ImageStyle::Vivid),
        };

        let value = serde_json::to_value(params).expect("serialize image generate params");
        assert_eq!(
            value.get("model"),
            Some(&serde_json::Value::String("gpt-image-1".to_owned()))
        );
        assert_eq!(value.get("n"), Some(&serde_json::json!(2)));
        assert_eq!(
            value.get("background"),
            Some(&serde_json::Value::String("transparent".to_owned()))
        );
        assert_eq!(
            value.get("moderation"),
            Some(&serde_json::Value::String("low".to_owned()))
        );
        assert_eq!(
            value.get("output_format"),
            Some(&serde_json::Value::String("png".to_owned()))
        );
        assert_eq!(
            value.get("quality"),
            Some(&serde_json::Value::String("high".to_owned()))
        );
        assert_eq!(
            value.get("style"),
            Some(&serde_json::Value::String("vivid".to_owned()))
        );
        assert_eq!(
            value.get("output_compression"),
            Some(&serde_json::json!(80))
        );
        assert_eq!(value.get("partial_images"), Some(&serde_json::json!(2)));
    }

    #[test]
    fn image_input_file_can_set_content_type() {
        let image = ImageInputFile::from_bytes(vec![1_u8, 2, 3], "image.png")
            .with_content_type("image/png");
        assert_eq!(image.content_type.as_deref(), Some("image/png"));
    }

    #[test]
    fn image_response_deserializes_with_new_fields() {
        let json = r#"{
            "created": 1700000000,
            "data": [{"url": "https://example.com/a.png"}],
            "background": "opaque",
            "output_format": "png",
            "quality": "high",
            "size": "1024x1024",
            "usage": {
                "input_tokens": 100,
                "output_tokens": 200,
                "total_tokens": 300,
                "input_tokens_details": {"image_tokens": 50, "text_tokens": 50},
                "output_tokens_details": {"image_tokens": 150, "text_tokens": 50}
            }
        }"#;

        let response: ImageResponse = serde_json::from_str(json).expect("deserialize response");
        assert_eq!(response.data.len(), 1);
        assert_eq!(
            response.data[0].url.as_deref(),
            Some("https://example.com/a.png")
        );
        assert_eq!(response.background.as_deref(), Some("opaque"));
        assert_eq!(response.output_format.as_deref(), Some("png"));
        assert_eq!(response.quality.as_deref(), Some("high"));
        assert_eq!(response.size.as_deref(), Some("1024x1024"));

        let usage = response.usage.expect("usage present");
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 200);
        assert_eq!(usage.total_tokens, 300);
        assert_eq!(
            usage
                .input_tokens_details
                .as_ref()
                .and_then(|d| d.image_tokens),
            Some(50)
        );
        assert_eq!(
            usage
                .output_tokens_details
                .as_ref()
                .and_then(|d| d.text_tokens),
            Some(50)
        );
    }

    #[test]
    fn image_response_deserializes_url_payload_minimal() {
        let json = r#"{
            "created": 1700000000,
            "data": [{"url": "https://example.com/a.png"}]
        }"#;

        let response: ImageResponse = serde_json::from_str(json).expect("deserialize response");
        assert_eq!(response.data.len(), 1);
        assert!(response.background.is_none());
        assert!(response.usage.is_none());
    }

    #[test]
    fn image_model_serialization() {
        let model = ImageModel::GptImage1;
        let json = serde_json::to_string(&model).expect("serialize image model");
        assert_eq!(json, "\"gpt-image-1\"");

        let model = ImageModel::DallE3;
        let json = serde_json::to_string(&model).expect("serialize image model");
        assert_eq!(json, "\"dall-e-3\"");

        let model = ImageModel::GptImage1_5;
        let json = serde_json::to_string(&model).expect("serialize image model");
        assert_eq!(json, "\"gpt-image-1.5\"");

        let model = ImageModel::GptImage1Mini;
        let json = serde_json::to_string(&model).expect("serialize image model");
        assert_eq!(json, "\"gpt-image-1-mini\"");
    }

    #[test]
    fn image_size_all_variants() {
        let sizes = vec![
            ImageSize::Auto,
            ImageSize::Size256x256,
            ImageSize::Size512x512,
            ImageSize::Size1024x1024,
            ImageSize::Size1536x1024,
            ImageSize::Size1024x1536,
            ImageSize::Size1792x1024,
            ImageSize::Size1024x1792,
        ];
        assert_eq!(sizes.len(), 8);

        // Check serialization
        assert_eq!(
            serde_json::to_string(&ImageSize::Size1792x1024).unwrap(),
            "\"1792x1024\""
        );
    }

    #[test]
    fn image_gen_stream_event_deserialization() {
        let partial_json = r#"{
            "type": "image_generation.partial_image",
            "b64_json": "abc123",
            "partial_image_index": 1,
            "background": "transparent",
            "quality": "high"
        }"#;

        let event: ImageGenStreamEvent =
            serde_json::from_str(partial_json).expect("deserialize partial event");
        match event {
            ImageGenStreamEvent::PartialImage(e) => {
                assert_eq!(e.b64_json, "abc123");
                assert_eq!(e.partial_image_index, 1);
            }
            _ => panic!("Expected PartialImage variant"),
        }

        let completed_json = r#"{
            "type": "image_generation.completed",
            "b64_json": "xyz789",
            "background": "opaque",
            "usage": {
                "input_tokens": 10,
                "output_tokens": 20,
                "total_tokens": 30
            }
        }"#;

        let event: ImageGenStreamEvent =
            serde_json::from_str(completed_json).expect("deserialize completed event");
        match event {
            ImageGenStreamEvent::Completed(e) => {
                assert_eq!(e.b64_json, "xyz789");
                assert!(e.usage.is_some());
            }
            _ => panic!("Expected Completed variant"),
        }
    }

    #[test]
    fn image_edit_stream_event_deserialization() {
        let completed_json = r#"{
            "type": "image_edit.completed",
            "b64_json": "edited_data",
            "background": "auto"
        }"#;

        let event: ImageEditStreamEvent =
            serde_json::from_str(completed_json).expect("deserialize edit completed event");
        match event {
            ImageEditStreamEvent::Completed(e) => {
                assert_eq!(e.b64_json, "edited_data");
            }
            _ => panic!("Expected Completed variant"),
        }
    }

    #[test]
    fn image_edit_input_single_from_image_input_file() {
        let file = ImageInputFile::from_bytes(vec![1, 2, 3], "test.png");
        let input: ImageEditInput = file.into();
        match &input {
            ImageEditInput::Single(f) => {
                assert_eq!(f.filename, "test.png");
                assert_eq!(f.bytes, vec![1, 2, 3]);
            }
            ImageEditInput::Multiple(_) => panic!("Expected Single variant"),
        }
    }

    #[test]
    fn image_edit_input_multiple_from_vec() {
        let files = vec![
            ImageInputFile::from_bytes(vec![1], "a.png"),
            ImageInputFile::from_bytes(vec![2], "b.png"),
            ImageInputFile::from_bytes(vec![3], "c.png"),
        ];
        let input: ImageEditInput = files.into();
        match &input {
            ImageEditInput::Multiple(fs) => {
                assert_eq!(fs.len(), 3);
                assert_eq!(fs[0].filename, "a.png");
                assert_eq!(fs[1].filename, "b.png");
                assert_eq!(fs[2].filename, "c.png");
            }
            ImageEditInput::Single(_) => panic!("Expected Multiple variant"),
        }
    }

    #[test]
    fn image_edit_params_to_form_single_image() {
        let params = ImageEditParams {
            image: ImageEditInput::Single(ImageInputFile::from_bytes(vec![0xFF], "single.png")),
            prompt: "Edit this".to_owned(),
            mask: None,
            model: Some(ModelId::from("gpt-image-1")),
            n: None,
            size: None,
            response_format: None,
            output_compression: None,
            partial_images: None,
            user: None,
            background: None,
            input_fidelity: None,
            output_format: None,
            quality: None,
        };
        // Smoke test: form construction should not panic.
        let _form = params.to_form();
    }

    #[test]
    fn image_edit_params_to_form_multiple_images() {
        let params = ImageEditParams {
            image: ImageEditInput::Multiple(vec![
                ImageInputFile::from_bytes(vec![0xAA], "first.png"),
                ImageInputFile::from_bytes(vec![0xBB], "second.png"),
            ]),
            prompt: "Combine these".to_owned(),
            mask: None,
            model: Some(ModelId::from("gpt-image-1")),
            n: Some(1),
            size: None,
            response_format: None,
            output_compression: None,
            partial_images: None,
            user: None,
            background: None,
            input_fidelity: None,
            output_format: None,
            quality: None,
        };
        // Smoke test: form construction should not panic.
        let _form = params.to_form();
    }
}
