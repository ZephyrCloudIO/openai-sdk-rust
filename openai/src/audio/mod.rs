//! Audio APIs.

use reqwest::multipart::{Form, Part};

use crate::{shared::ModelId, Client, Result};

/// Audio service root.
#[derive(Clone)]
pub struct AudioService {
    client: Client,
}

impl AudioService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates a transcription from an audio file.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn transcribe(
        &self,
        params: AudioTranscriptionCreateParams,
    ) -> Result<AudioTranscription> {
        let model = params.model.to_string();
        let filename = params.file.filename;
        let bytes = params.file.bytes;
        let content_type = params.file.content_type;
        let prompt = params.prompt;
        let language = params.language;
        let temperature = params.temperature;

        self.client
            .post_multipart_json("/audio/transcriptions", move || {
                let part = if let Some(content_type) = content_type.as_deref() {
                    Part::bytes(bytes.clone())
                        .file_name(filename.clone())
                        .mime_str(content_type)
                        .unwrap_or_else(|_| Part::bytes(bytes.clone()).file_name(filename.clone()))
                } else {
                    Part::bytes(bytes.clone()).file_name(filename.clone())
                };

                let mut form = Form::new().part("file", part).text("model", model.clone());
                if let Some(prompt) = prompt.as_deref() {
                    form = form.text("prompt", prompt.to_owned());
                }
                if let Some(language) = language.as_deref() {
                    form = form.text("language", language.to_owned());
                }
                if let Some(temperature) = temperature {
                    form = form.text("temperature", temperature.to_string());
                }
                form
            })
            .await
    }

    /// Creates an English translation from an audio file.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn translate(
        &self,
        params: AudioTranslationCreateParams,
    ) -> Result<AudioTranslation> {
        let model = params.model.to_string();
        let filename = params.file.filename;
        let bytes = params.file.bytes;
        let content_type = params.file.content_type;
        let prompt = params.prompt;
        let temperature = params.temperature;

        self.client
            .post_multipart_json("/audio/translations", move || {
                let part = if let Some(content_type) = content_type.as_deref() {
                    Part::bytes(bytes.clone())
                        .file_name(filename.clone())
                        .mime_str(content_type)
                        .unwrap_or_else(|_| Part::bytes(bytes.clone()).file_name(filename.clone()))
                } else {
                    Part::bytes(bytes.clone()).file_name(filename.clone())
                };

                let mut form = Form::new().part("file", part).text("model", model.clone());
                if let Some(prompt) = prompt.as_deref() {
                    form = form.text("prompt", prompt.to_owned());
                }
                if let Some(temperature) = temperature {
                    form = form.text("temperature", temperature.to_string());
                }
                form
            })
            .await
    }

    /// Generates spoken audio bytes from text input.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn speech(&self, params: AudioSpeechCreateParams) -> Result<Vec<u8>> {
        self.client.post_json_bytes("/audio/speech", &params).await
    }
}

/// In-memory audio file input.
#[derive(Debug, Clone)]
pub struct AudioInputFile {
    /// Audio bytes.
    pub bytes: Vec<u8>,
    /// Filename provided to API.
    pub filename: String,
    /// Optional MIME type.
    pub content_type: Option<String>,
}

impl AudioInputFile {
    /// Creates an input file from bytes.
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

/// Audio transcription request.
#[derive(Debug, Clone)]
pub struct AudioTranscriptionCreateParams {
    /// Audio input file.
    pub file: AudioInputFile,
    /// Model ID.
    pub model: ModelId,
    /// Optional prompt.
    pub prompt: Option<String>,
    /// Optional source language hint.
    pub language: Option<String>,
    /// Optional sampling temperature.
    pub temperature: Option<f32>,
}

/// Audio translation request.
#[derive(Debug, Clone)]
pub struct AudioTranslationCreateParams {
    /// Audio input file.
    pub file: AudioInputFile,
    /// Model ID.
    pub model: ModelId,
    /// Optional prompt.
    pub prompt: Option<String>,
    /// Optional sampling temperature.
    pub temperature: Option<f32>,
}

/// Audio speech generation request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioSpeechCreateParams {
    /// Model ID.
    pub model: ModelId,
    /// Input text.
    pub input: String,
    /// Voice name.
    pub voice: String,
    /// Optional output format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Transcription response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioTranscription {
    /// Decoded text.
    pub text: String,
}

/// Translation response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioTranslation {
    /// Decoded text.
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::{AudioInputFile, AudioSpeechCreateParams};
    use crate::shared::ModelId;

    #[test]
    fn speech_params_omit_optional_format() {
        let params = AudioSpeechCreateParams {
            model: ModelId::from("gpt-4o-mini-tts"),
            input: "hello".to_owned(),
            voice: "alloy".to_owned(),
            format: None,
        };

        let value = serde_json::to_value(params).expect("serialize speech params");
        assert_eq!(
            value.get("model"),
            Some(&serde_json::Value::String("gpt-4o-mini-tts".to_owned()))
        );
        assert!(value.get("format").is_none());
    }

    #[test]
    fn audio_input_file_can_set_content_type() {
        let file = AudioInputFile::from_bytes(vec![0_u8, 1, 2], "audio.wav")
            .with_content_type("audio/wav");
        assert_eq!(file.content_type.as_deref(), Some("audio/wav"));
    }
}
