//! Images APIs.

use reqwest::multipart::{Form, Part};

use crate::{shared::ModelId, Client, Result};

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

    /// Creates edited images from prompt and source image.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn edit(&self, params: ImageEditParams) -> Result<ImageResponse> {
        self.client
            .post_multipart_json("/images/edits", || params.to_form())
            .await
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
    /// Optional size (`1024x1024`, etc).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Optional response format (`url`, `b64_json`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
}

/// Image edit request payload.
#[derive(Debug, Clone)]
pub struct ImageEditParams {
    /// Source image.
    pub image: ImageInputFile,
    /// Prompt text.
    pub prompt: String,
    /// Optional mask image.
    pub mask: Option<ImageInputFile>,
    /// Optional model ID.
    pub model: Option<ModelId>,
    /// Optional number of images.
    pub n: Option<u32>,
    /// Optional size.
    pub size: Option<String>,
    /// Optional response format.
    pub response_format: Option<String>,
}

impl ImageEditParams {
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

        let mut form = Form::new()
            .text("prompt", self.prompt.clone())
            .part("image", image_part);

        if let Some(model) = &self.model {
            form = form.text("model", model.to_string());
        }
        if let Some(n) = self.n {
            form = form.text("n", n.to_string());
        }
        if let Some(size) = self.size.as_deref() {
            form = form.text("size", size.to_owned());
        }
        if let Some(response_format) = self.response_format.as_deref() {
            form = form.text("response_format", response_format.to_owned());
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
    pub size: Option<String>,
    /// Optional response format.
    pub response_format: Option<String>,
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
        if let Some(size) = self.size.as_deref() {
            form = form.text("size", size.to_owned());
        }
        if let Some(response_format) = self.response_format.as_deref() {
            form = form.text("response_format", response_format.to_owned());
        }

        form
    }
}

/// Image API response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageResponse {
    /// Unix creation timestamp.
    pub created: u64,
    /// Returned image payloads.
    pub data: Vec<ImageData>,
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
    use super::{ImageGenerateParams, ImageInputFile, ImageResponse};

    #[test]
    fn image_generate_omits_optional_fields() {
        let params = ImageGenerateParams {
            prompt: "A red fox".to_owned(),
            model: None,
            n: None,
            size: None,
            response_format: None,
        };

        let value = serde_json::to_value(params).expect("serialize image generate params");
        assert!(value.get("model").is_none());
        assert!(value.get("n").is_none());
    }

    #[test]
    fn image_input_file_can_set_content_type() {
        let image = ImageInputFile::from_bytes(vec![1_u8, 2, 3], "image.png")
            .with_content_type("image/png");
        assert_eq!(image.content_type.as_deref(), Some("image/png"));
    }

    #[test]
    fn image_response_deserializes_url_payload() {
        let json = r#"{
            "created": 1700000000,
            "data": [{"url": "https://example.com/a.png"}]
        }"#;

        let response: ImageResponse = serde_json::from_str(json).expect("deserialize response");
        assert_eq!(response.data.len(), 1);
        assert_eq!(
            response.data[0].url.as_deref(),
            Some("https://example.com/a.png")
        );
    }
}
